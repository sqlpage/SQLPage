//! OpenTelemetry initialization and shutdown.
//!
//! When `OTEL_EXPORTER_OTLP_ENDPOINT` is set, sets up a full tracing pipeline
//! with OTLP export. Otherwise, sets up tracing with logfmt output only.
//!
//! In both cases, the same logfmt log format is used, with carefully chosen
//! fields for human readability and machine parseability.

use std::env;
use std::sync::{Once, OnceLock};

use anyhow::Context as _;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;

static TRACER_PROVIDER: OnceLock<SdkTracerProvider> = OnceLock::new();
static METER_PROVIDER: OnceLock<SdkMeterProvider> = OnceLock::new();
static TEST_LOGGING_INIT: Once = Once::new();
static OTLP_HTTP_WORKER_SENDER: OnceLock<
    Result<tokio::sync::mpsc::UnboundedSender<OtlpHttpJob>, String>,
> = OnceLock::new();
const DEFAULT_ENV_FILTER_DIRECTIVES: &str = "sqlpage=info,actix_web=info,tracing_actix_web=info,opentelemetry=warn,opentelemetry_sdk=warn,opentelemetry_otlp=warn";

type OtlpHttpResponse = Result<opentelemetry_http::Response<opentelemetry_http::Bytes>, String>;

struct OtlpHttpJob {
    request: opentelemetry_http::Request<opentelemetry_http::Bytes>,
    response_sender: tokio::sync::oneshot::Sender<OtlpHttpResponse>,
}

#[derive(Clone)]
struct AwcOtlpHttpClient {
    sender: tokio::sync::mpsc::UnboundedSender<OtlpHttpJob>,
}

impl AwcOtlpHttpClient {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            sender: otlp_http_worker_sender()?,
        })
    }
}

impl std::fmt::Debug for AwcOtlpHttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AwcOtlpHttpClient").finish_non_exhaustive()
    }
}

fn otlp_http_worker_sender() -> anyhow::Result<tokio::sync::mpsc::UnboundedSender<OtlpHttpJob>> {
    let sender_result = OTLP_HTTP_WORKER_SENDER
        .get_or_init(|| init_otlp_http_worker_sender().map_err(|error| error.to_string()));

    match sender_result {
        Ok(sender) => Ok(sender.clone()),
        Err(error) => {
            anyhow::bail!("Failed to initialize OTLP AWC worker thread: {error}");
        }
    }
}

fn init_otlp_http_worker_sender() -> anyhow::Result<tokio::sync::mpsc::UnboundedSender<OtlpHttpJob>>
{
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<OtlpHttpJob>();
    let (init_result_sender, init_result_receiver) = std::sync::mpsc::sync_channel(1);
    let use_system_root_ca_certificates =
        crate::webserver::http_client::default_system_root_ca_certificates_from_env();

    std::thread::Builder::new()
        .name("sqlpage-otlp-http".to_owned())
        .spawn(move || {
            actix_web::rt::System::new().block_on(async move {
                let awc_client =
                    match crate::webserver::http_client::make_http_client_with_system_roots(
                        use_system_root_ca_certificates,
                    ) {
                        Ok(client) => {
                            let _ = init_result_sender.send(Ok(()));
                            client
                        }
                        Err(error) => {
                            let error = format!("Failed to initialize OTLP AWC client: {error:#}");
                            let _ = init_result_sender.send(Err(error.clone()));
                            while let Some(job) = receiver.recv().await {
                                let _ = job.response_sender.send(Err(error.clone()));
                            }
                            return;
                        }
                    };

                while let Some(job) = receiver.recv().await {
                    let response = execute_otlp_http_request_with_awc(&awc_client, job.request)
                        .await
                        .map_err(|error| error.to_string());
                    let _ = job.response_sender.send(response);
                }
            });
        })
        .context("Failed to spawn OTLP AWC worker thread")?;

    match init_result_receiver.recv() {
        Ok(Ok(())) => {}
        Ok(Err(error)) => anyhow::bail!("{error}"),
        Err(error) => anyhow::bail!("OTLP AWC worker initialization channel closed: {error}"),
    }

    Ok(sender)
}

async fn execute_otlp_http_request_with_awc(
    awc_client: &awc::Client,
    request: opentelemetry_http::Request<opentelemetry_http::Bytes>,
) -> anyhow::Result<opentelemetry_http::Response<opentelemetry_http::Bytes>> {
    let (request_parts, request_body) = request.into_parts();

    let awc_method = awc::http::Method::from_bytes(request_parts.method.as_str().as_bytes())
        .with_context(|| format!("Invalid OTLP HTTP method: {}", request_parts.method))?;
    let awc_uri: awc::http::Uri = request_parts
        .uri
        .to_string()
        .parse()
        .with_context(|| format!("Invalid OTLP collector URI: {}", request_parts.uri))?;

    let mut awc_request = awc_client.request(awc_method, awc_uri.clone());
    for (header_name, header_value) in &request_parts.headers {
        let header_name_str = header_name.as_str();
        let awc_header_name = awc::http::header::HeaderName::from_bytes(header_name_str.as_bytes())
            .with_context(|| format!("Invalid OTLP header name: {header_name_str}"))?;
        let awc_header_value = awc::http::header::HeaderValue::from_bytes(header_value.as_bytes())
            .with_context(|| format!("Invalid OTLP header value for {header_name_str}"))?;
        awc_request = awc_request.insert_header((awc_header_name, awc_header_value));
    }

    let mut awc_response = awc_request.send_body(request_body).await.map_err(|error| {
        anyhow::anyhow!("Failed to send OTLP HTTP request to {awc_uri}: {error}")
    })?;

    let mut response_builder =
        opentelemetry_http::Response::builder().status(awc_response.status().as_u16());
    for (header_name, header_value) in awc_response.headers() {
        let header_value = header_value.to_str().map_err(|error| {
            anyhow::anyhow!(
                "Invalid OTLP response header value for {}: {error}",
                header_name.as_str()
            )
        })?;
        response_builder = response_builder.header(header_name.as_str(), header_value);
    }

    let response_body = awc_response.body().await.map_err(|error| {
        anyhow::anyhow!("Failed to read OTLP HTTP response body from {awc_uri}: {error}")
    })?;

    response_builder
        .body(response_body)
        .context("Failed to build OTLP HTTP response")
}

#[async_trait::async_trait]
impl opentelemetry_http::HttpClient for AwcOtlpHttpClient {
    async fn send_bytes(
        &self,
        request: opentelemetry_http::Request<opentelemetry_http::Bytes>,
    ) -> Result<
        opentelemetry_http::Response<opentelemetry_http::Bytes>,
        opentelemetry_http::HttpError,
    > {
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();
        self.sender
            .send(OtlpHttpJob {
                request,
                response_sender,
            })
            .map_err(|_| anyhow::anyhow!("OTLP AWC worker thread is unavailable"))?;

        response_receiver
            .await
            .map_err(|_| anyhow::anyhow!("OTLP AWC worker dropped response channel"))?
            .map_err(Into::into)
    }
}

/// Initializes logging / tracing. Returns whether `OTel` was activated.
pub fn init_telemetry() -> anyhow::Result<bool> {
    init_telemetry_with_log_layer(logfmt::LogfmtLayer::new())
}

fn init_telemetry_with_log_layer(logfmt_layer: logfmt::LogfmtLayer) -> anyhow::Result<bool> {
    let otel_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
    let otel_active = otel_endpoint.as_deref().is_some_and(|v| !v.is_empty());

    if otel_active {
        init_otel_tracing(logfmt_layer)?;
    } else {
        init_tracing(logfmt_layer)?;
    }

    Ok(otel_active)
}

/// Initializes logging once for tests using the same formatter as production.
///
/// Unlike `init_telemetry`, this does not initialize OTEL exporters and does
/// not panic on invalid `LOG_LEVEL` / `RUST_LOG` values.
pub fn init_test_logging() {
    TEST_LOGGING_INIT.call_once(|| {
        init_test_tracing();
    });
}

/// Shuts down the `OTel` tracer provider, flushing pending spans.
pub fn shutdown_telemetry() {
    if let Some(provider) = TRACER_PROVIDER.get() {
        if let Err(e) = provider.shutdown() {
            eprintln!("Error shutting down tracer provider: {e}");
        }
    }
    if let Some(provider) = METER_PROVIDER.get() {
        if let Err(e) = provider.shutdown() {
            eprintln!("Error shutting down meter provider: {e}");
        }
    }
}

/// Tracing subscriber without `OTel` export — logfmt output only.
fn init_tracing(logfmt_layer: logfmt::LogfmtLayer) -> anyhow::Result<()> {
    use tracing_subscriber::layer::SubscriberExt;

    let subscriber = tracing_subscriber::registry()
        .with(default_env_filter())
        .with(logfmt_layer);

    set_global_subscriber(subscriber)
}

fn init_test_tracing() {
    use tracing_subscriber::layer::SubscriberExt;

    let subscriber = tracing_subscriber::registry()
        .with(test_env_filter())
        .with(logfmt::LogfmtLayer::test_writer());

    set_global_subscriber(subscriber).expect("Failed to initialize test tracing subscriber");
}

fn init_otel_tracing(logfmt_layer: logfmt::LogfmtLayer) -> anyhow::Result<()> {
    use opentelemetry::global;
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_otlp::WithExportConfig as _;
    use opentelemetry_otlp::WithHttpConfig as _;
    use opentelemetry_sdk::propagation::TraceContextPropagator;
    use tracing_subscriber::layer::SubscriberExt;

    let http_client = AwcOtlpHttpClient::new().context("Failed to initialize OTLP AWC client")?;

    // W3C TraceContext propagation (traceparent header)
    global::set_text_map_propagator(TraceContextPropagator::new());
    // OTLP exporter — reads OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_SERVICE_NAME, etc.
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_http_client(http_client.clone())
        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
        .build()
        .context("Failed to build OTLP span exporter")?;

    let span_processor =
        opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor::builder(
            exporter,
            opentelemetry_sdk::runtime::Tokio,
        )
        .build();
    let provider = SdkTracerProvider::builder()
        .with_span_processor(span_processor)
        .build();

    let tracer = provider.tracer("sqlpage");
    global::set_tracer_provider(provider.clone());
    let _ = TRACER_PROVIDER.set(provider);

    // OTLP Metric exporter
    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_http_client(http_client)
        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
        .build()
        .context("Failed to build OTLP metric exporter")?;

    let reader =
        opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader::builder(
            metric_exporter,
            opentelemetry_sdk::runtime::Tokio,
        )
        .build();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_view(|instrument: &opentelemetry_sdk::metrics::Instrument| {
            if instrument.kind() == opentelemetry_sdk::metrics::InstrumentKind::Histogram {
                opentelemetry_sdk::metrics::Stream::builder()
                    .with_aggregation(
                        opentelemetry_sdk::metrics::Aggregation::ExplicitBucketHistogram {
                            boundaries: vec![
                                0.001, 0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0,
                                2.5, 5.0, 7.5, 10.0,
                            ],
                            record_min_max: true,
                        },
                    )
                    .build()
                    .ok()
            } else {
                None
            }
        })
        .build();
    global::set_meter_provider(meter_provider.clone());
    let _ = METER_PROVIDER.set(meter_provider.clone());

    let otel_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_location(false);

    let subscriber = tracing_subscriber::registry()
        .with(default_env_filter())
        .with(logfmt_layer)
        .with(otel_layer)
        .with(tracing_opentelemetry::MetricsLayer::new(meter_provider));

    set_global_subscriber(subscriber)
}

fn default_env_filter() -> tracing_subscriber::EnvFilter {
    env_filter_directives(
        env::var("LOG_LEVEL").ok().as_deref(),
        env::var("RUST_LOG").ok().as_deref(),
    )
    .parse()
    .expect("Invalid log filter value in LOG_LEVEL or RUST_LOG")
}

fn test_env_filter() -> tracing_subscriber::EnvFilter {
    env_filter_directives(
        env::var("LOG_LEVEL").ok().as_deref(),
        env::var("RUST_LOG").ok().as_deref(),
    )
    .parse()
    .unwrap_or_else(|_| {
        DEFAULT_ENV_FILTER_DIRECTIVES
            .parse()
            .expect("Default filter directives should always be valid")
    })
}

fn env_filter_directives(log_level: Option<&str>, rust_log: Option<&str>) -> String {
    match (
        log_level.filter(|value| !value.is_empty()),
        rust_log.filter(|value| !value.is_empty()),
    ) {
        (Some(value), _) | (None, Some(value)) => value.to_owned(),
        (None, None) => DEFAULT_ENV_FILTER_DIRECTIVES.to_owned(),
    }
}

fn set_global_subscriber(subscriber: impl tracing::Subscriber + Send + Sync) -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set global tracing subscriber")?;
    tracing_log::LogTracer::init().context("Failed to set log→tracing bridge")
}

/// Custom logfmt logging layer.
///
/// Outputs one line per event in logfmt format with carefully chosen fields:
/// ```text
/// ts=2026-03-08T20:56:15Z level=error target=sqlpage::webserver::error msg="..." http.request.method=GET url.path=/foo client.address=1.2.3.4 trace_id=abc123
/// ```
///
/// With terminal colors when stderr is a TTY.
mod logfmt {
    use std::collections::BTreeMap;
    use std::collections::HashMap;
    use std::fmt::Write;
    use std::io::{self, IsTerminal};

    use tracing::Subscriber;
    use tracing::field::{Field, Visit};
    use tracing_subscriber::Layer;
    use tracing_subscriber::layer::Context;
    use tracing_subscriber::registry::LookupSpan;

    /// Stores span fields so we can access them when formatting events.
    #[derive(Default)]
    struct SpanFields(HashMap<&'static str, String>);

    /// Visitor that collects fields into a `HashMap`.
    struct FieldCollector<'a>(&'a mut HashMap<&'static str, String>);

    impl Visit for FieldCollector<'_> {
        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            self.0.insert(field.name(), format!("{value:?}"));
        }
        fn record_str(&mut self, field: &Field, value: &str) {
            self.0.insert(field.name(), value.to_owned());
        }
        fn record_i64(&mut self, field: &Field, value: i64) {
            self.0.insert(field.name(), value.to_string());
        }
        fn record_u64(&mut self, field: &Field, value: u64) {
            self.0.insert(field.name(), value.to_string());
        }
        fn record_bool(&mut self, field: &Field, value: bool) {
            self.0.insert(field.name(), value.to_string());
        }
    }

    use opentelemetry_semantic_conventions::attribute as otel;
    /// Fields we pick from spans, in display order.
    /// (`span_field_name`, `logfmt_key`)
    const SPAN_FIELDS: &[(&str, &str)] = &[
        (otel::HTTP_REQUEST_METHOD, otel::HTTP_REQUEST_METHOD),
        (otel::URL_PATH, otel::URL_PATH),
        (
            otel::HTTP_RESPONSE_STATUS_CODE,
            otel::HTTP_RESPONSE_STATUS_CODE,
        ),
        (otel::CODE_FILE_PATH, otel::CODE_FILE_PATH),
        (otel::CLIENT_ADDRESS, otel::CLIENT_ADDRESS),
    ];

    /// All-zeros trace ID means no real trace context.
    const INVALID_TRACE_ID: &str = "00000000000000000000000000000000";

    // ANSI color codes
    const RED: &str = "\x1b[31m";
    const YELLOW: &str = "\x1b[33m";
    const GREEN: &str = "\x1b[32m";
    const BLUE: &str = "\x1b[34m";
    const DIM: &str = "\x1b[2m";
    const BOLD: &str = "\x1b[1m";
    const RESET: &str = "\x1b[0m";

    #[derive(Copy, Clone)]
    enum OutputMode {
        Stderr,
        TestWriter,
    }

    pub(super) struct LogfmtLayer {
        use_colors: bool,
        output_mode: OutputMode,
    }

    impl LogfmtLayer {
        pub fn new() -> Self {
            Self {
                use_colors: io::stderr().is_terminal(),
                output_mode: OutputMode::Stderr,
            }
        }

        pub fn test_writer() -> Self {
            Self {
                use_colors: false,
                output_mode: OutputMode::TestWriter,
            }
        }
    }

    impl<S> Layer<S> for LogfmtLayer
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_new_span(
            &self,
            attrs: &tracing::span::Attributes<'_>,
            id: &tracing::span::Id,
            ctx: Context<'_, S>,
        ) {
            let mut fields = SpanFields::default();
            attrs.record(&mut FieldCollector(&mut fields.0));
            if let Some(span) = ctx.span(id) {
                span.extensions_mut().insert(fields);
            }
        }

        fn on_record(
            &self,
            id: &tracing::span::Id,
            values: &tracing::span::Record<'_>,
            ctx: Context<'_, S>,
        ) {
            if let Some(span) = ctx.span(id) {
                let mut ext = span.extensions_mut();
                if let Some(fields) = ext.get_mut::<SpanFields>() {
                    values.record(&mut FieldCollector(&mut fields.0));
                }
            }
        }

        fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
            let mut buf = String::with_capacity(256);
            let colors = self.use_colors;
            let level = *event.metadata().level();
            let include_all_span_fields = includes_all_span_fields();
            let mut event_fields = HashMap::new();
            event.record(&mut FieldCollector(&mut event_fields));
            let target = event_target(event, &event_fields);
            let msg = event_fields.get("message");
            let multiline_msg = is_multiline_terminal_message(colors, msg);
            let include_all_event_fields =
                include_all_span_fields || msg.is_none_or(String::is_empty);

            write_timestamp(&mut buf, colors);
            write_level(&mut buf, level, colors);
            write_message(&mut buf, msg, multiline_msg);
            write_dimmed_field(&mut buf, "target", target, colors);
            write_event_fields(&mut buf, &event_fields, include_all_event_fields);
            write_span_fields(&mut buf, ctx.event_scope(event), include_all_span_fields);
            write_trace_id(&mut buf, ctx.event_scope(event), colors);

            buf.push('\n');
            write_multiline_message(&mut buf, msg, multiline_msg);
            match self.output_mode {
                OutputMode::Stderr => {
                    let _ = io::Write::write_all(&mut io::stderr().lock(), buf.as_bytes());
                }
                OutputMode::TestWriter => {
                    eprint!("{buf}");
                }
            }
        }
    }

    fn event_target<'a>(
        event: &'a tracing::Event<'_>,
        event_fields: &'a HashMap<&'static str, String>,
    ) -> &'a str {
        event_fields
            .get("log.target")
            .map_or_else(|| event.metadata().target(), String::as_str)
    }

    fn is_multiline_terminal_message(colors: bool, msg: Option<&String>) -> bool {
        colors && msg.is_some_and(|message| message.contains('\n'))
    }

    fn write_timestamp(buf: &mut String, colors: bool) {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
        if colors {
            let _ = write!(buf, "{DIM}ts={now}{RESET}");
        } else {
            let _ = write!(buf, "ts={now}");
        }
    }

    fn write_level(buf: &mut String, level: tracing::Level, colors: bool) {
        if colors {
            let (color, label) = level_style(level);
            let _ = write!(buf, " {DIM}level={RESET}{BOLD}{color}{label}{RESET}");
        } else {
            let _ = write!(buf, " level={}", level.as_str().to_ascii_lowercase());
        }
    }

    fn level_style(level: tracing::Level) -> (&'static str, &'static str) {
        match level {
            tracing::Level::ERROR => (RED, "error"),
            tracing::Level::WARN => (YELLOW, "warn"),
            tracing::Level::INFO => (GREEN, "info"),
            tracing::Level::DEBUG => (BLUE, "debug"),
            tracing::Level::TRACE => (DIM, "trace"),
        }
    }

    fn write_dimmed_field(buf: &mut String, key: &str, value: &str, colors: bool) {
        if colors {
            let _ = write!(buf, " {DIM}{key}={value}{RESET}");
        } else {
            let _ = write!(buf, " {key}={value}");
        }
    }

    fn write_message(buf: &mut String, msg: Option<&String>, multiline_msg: bool) {
        if !multiline_msg {
            if let Some(msg) = msg.filter(|msg| !msg.is_empty()) {
                write_logfmt_value(buf, "msg", msg);
            }
        }
    }

    fn write_event_fields(
        buf: &mut String,
        event_fields: &HashMap<&'static str, String>,
        include_all_event_fields: bool,
    ) {
        if !include_all_event_fields {
            return;
        }

        let mut extra_fields = BTreeMap::new();
        for (&key, value) in event_fields {
            if key == "message" || key == "log.target" {
                continue;
            }
            extra_fields.insert(key, value);
        }

        for (key, value) in extra_fields {
            write_logfmt_value(buf, key, value);
        }
    }

    fn write_span_fields<S>(
        buf: &mut String,
        scope: Option<tracing_subscriber::registry::Scope<'_, S>>,
        include_all_span_fields: bool,
    ) where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        if let Some(scope) = scope {
            let mut seen_mapped_fields = [false; SPAN_FIELDS.len()];
            let mut extra_fields = BTreeMap::new();

            for span in scope {
                let ext = span.extensions();
                if let Some(fields) = ext.get::<SpanFields>() {
                    for (i, &(span_key, logfmt_key)) in SPAN_FIELDS.iter().enumerate() {
                        if seen_mapped_fields[i] {
                            continue;
                        }
                        if let Some(val) = fields.0.get(span_key) {
                            write_logfmt_value(buf, logfmt_key, val);
                            seen_mapped_fields[i] = true;
                        }
                    }
                    if include_all_span_fields {
                        for (&key, val) in &fields.0 {
                            if SPAN_FIELDS.iter().any(|(span_key, _)| key == *span_key) {
                                continue;
                            }
                            extra_fields.entry(key).or_insert_with(|| val.clone());
                        }
                    }
                }
            }

            if include_all_span_fields {
                for (key, val) in extra_fields {
                    write_logfmt_value(buf, key, &val);
                }
            }
        }
    }

    fn includes_all_span_fields() -> bool {
        tracing::level_filters::LevelFilter::current() >= tracing::level_filters::LevelFilter::DEBUG
    }

    #[cfg(test)]
    fn write_span_field_maps<'a>(
        buf: &mut String,
        span_fields: impl IntoIterator<Item = &'a HashMap<&'static str, String>>,
        include_all_span_fields: bool,
    ) {
        let mut seen_mapped_fields = [false; SPAN_FIELDS.len()];
        let mut extra_fields = BTreeMap::new();

        for fields in span_fields {
            for (i, &(span_key, logfmt_key)) in SPAN_FIELDS.iter().enumerate() {
                if seen_mapped_fields[i] {
                    continue;
                }
                if let Some(val) = fields.get(span_key) {
                    write_logfmt_value(buf, logfmt_key, val);
                    seen_mapped_fields[i] = true;
                }
            }
            if include_all_span_fields {
                for (&key, val) in fields {
                    if SPAN_FIELDS.iter().any(|(span_key, _)| key == *span_key) {
                        continue;
                    }
                    extra_fields.entry(key).or_insert_with(|| val.clone());
                }
            }
        }

        if include_all_span_fields {
            for (key, val) in extra_fields {
                write_logfmt_value(buf, key, &val);
            }
        }
    }

    fn write_trace_id<S>(
        buf: &mut String,
        scope: Option<tracing_subscriber::registry::Scope<'_, S>>,
        colors: bool,
    ) where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        if let Some(trace_id) = first_valid_trace_id(scope) {
            write_dimmed_field(buf, "trace_id", &trace_id, colors);
        }
    }

    fn first_valid_trace_id<S>(
        scope: Option<tracing_subscriber::registry::Scope<'_, S>>,
    ) -> Option<String>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        for span in scope? {
            let ext = span.extensions();
            if let Some(otel_data) = ext.get::<tracing_opentelemetry::OtelData>() {
                if let Some(trace_id) = otel_data.trace_id() {
                    let trace_id = trace_id.to_string();
                    if trace_id != INVALID_TRACE_ID {
                        return Some(trace_id);
                    }
                }
            }
        }
        None
    }

    fn write_multiline_message(buf: &mut String, msg: Option<&String>, multiline_msg: bool) {
        if multiline_msg {
            if let Some(msg) = msg {
                buf.push_str(msg);
                buf.push('\n');
            }
        }
    }

    /// Write a logfmt key=value pair, quoting the value if it contains spaces or special chars.
    fn write_logfmt_value(buf: &mut String, key: &str, value: &str) {
        let needs_quoting = value.contains([' ', '"', '=', '\n', '\t']) || value.is_empty();

        if needs_quoting {
            let escaped = value.replace('\n', " ").replace('"', "\\\"");
            let _ = write!(buf, " {key}=\"{escaped}\"");
        } else {
            let _ = write!(buf, " {key}={value}");
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::telemetry::env_filter_directives;

        #[test]
        fn log_level_takes_precedence_over_rust_log() {
            assert_eq!(
                env_filter_directives(Some("sqlpage=debug"), Some("sqlpage=trace")),
                "sqlpage=debug"
            );
        }

        #[test]
        fn rust_log_is_used_as_alias_when_log_level_is_missing() {
            assert_eq!(
                env_filter_directives(None, Some("sqlpage=trace")),
                "sqlpage=trace"
            );
        }

        #[test]
        fn empty_values_fall_back_to_default_filter() {
            assert_eq!(
                env_filter_directives(Some(""), Some("")),
                "sqlpage=info,actix_web=info,tracing_actix_web=info,opentelemetry=warn,opentelemetry_sdk=warn,opentelemetry_otlp=warn"
            );
        }

        #[test]
        fn debug_logs_include_unmapped_span_fields() {
            let mut buf = String::new();
            let span_fields = HashMap::from([
                (otel::HTTP_REQUEST_METHOD, "GET".to_string()),
                (otel::HTTP_ROUTE, "/users/:id".to_string()),
                ("otel.kind", "server".to_string()),
            ]);

            write_span_field_maps(&mut buf, [&span_fields], true);

            assert_eq!(
                buf,
                " http.request.method=GET http.route=/users/:id otel.kind=server"
            );
        }

        #[test]
        fn info_logs_keep_only_mapped_span_fields_when_not_in_debug_mode() {
            let mut buf = String::new();
            let span_fields = HashMap::from([
                (otel::HTTP_REQUEST_METHOD, "GET".to_string()),
                (otel::HTTP_ROUTE, "/users/:id".to_string()),
                ("otel.kind", "server".to_string()),
            ]);

            write_span_field_maps(&mut buf, [&span_fields], false);

            assert_eq!(buf, " http.request.method=GET");
        }

        #[test]
        fn event_fields_are_rendered_when_message_is_missing() {
            let mut buf = String::new();
            let event_fields = HashMap::from([
                ("message", String::new()),
                ("log.target", "opentelemetry_sdk".to_string()),
                ("name", "BatchSpanProcessor.ExportFailed".to_string()),
                ("reason", "connection error".to_string()),
            ]);

            write_event_fields(&mut buf, &event_fields, true);

            assert_eq!(
                buf,
                " name=BatchSpanProcessor.ExportFailed reason=\"connection error\""
            );
        }
    }
}
