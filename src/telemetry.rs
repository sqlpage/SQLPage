//! OpenTelemetry initialization and shutdown.
//!
//! When `OTEL_EXPORTER_OTLP_ENDPOINT` is set, sets up a full tracing pipeline
//! with OTLP export. Otherwise, sets up tracing with logfmt output only.
//!
//! In both cases, the same logfmt log format is used, with carefully chosen
//! fields for human readability and machine parseability.

use std::env;
use std::sync::OnceLock;

use opentelemetry_sdk::trace::SdkTracerProvider;

static TRACER_PROVIDER: OnceLock<SdkTracerProvider> = OnceLock::new();

/// Initializes logging / tracing. Returns `true` if OTel was activated.
pub fn init_telemetry() -> bool {
    let otel_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
    let otel_active = otel_endpoint
        .as_deref()
        .is_some_and(|v| !v.is_empty());

    if otel_active {
        init_otel_tracing();
    } else {
        init_tracing();
    }

    otel_active
}

/// Shuts down the OTel tracer provider, flushing pending spans.
pub fn shutdown_telemetry() {
    if let Some(provider) = TRACER_PROVIDER.get() {
        if let Err(e) = provider.shutdown() {
            eprintln!("Error shutting down tracer provider: {e}");
        }
    }
}

/// Tracing subscriber without OTel export — logfmt output only.
fn init_tracing() {
    use tracing_subscriber::layer::SubscriberExt;

    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        "sqlpage=info,actix_web=info,tracing_actix_web=info".into()
    });

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(logfmt::LogfmtLayer::new());

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");
    tracing_log::LogTracer::init().expect("Failed to set log→tracing bridge");
}

fn init_otel_tracing() {
    use opentelemetry::global;
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_sdk::propagation::TraceContextPropagator;
    use tracing_subscriber::layer::SubscriberExt;

    // W3C TraceContext propagation (traceparent header)
    global::set_text_map_propagator(TraceContextPropagator::new());

    // OTLP exporter — reads OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_SERVICE_NAME, etc.
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .build()
        .expect("Failed to build OTLP span exporter");

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();

    let tracer = provider.tracer("sqlpage");
    global::set_tracer_provider(provider.clone());
    let _ = TRACER_PROVIDER.set(provider);

    let otel_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_location(false);

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "sqlpage=info,actix_web=info,tracing_actix_web=info".into());

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(logfmt::LogfmtLayer::new())
        .with(otel_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");
    tracing_log::LogTracer::init().expect("Failed to set log→tracing bridge");
}

/// Custom logfmt logging layer.
///
/// Outputs one line per event in logfmt format with carefully chosen fields:
/// ```text
/// ts=2026-03-08T20:56:15Z level=error target=sqlpage::webserver::error msg="..." method=GET path=/foo client_ip=1.2.3.4 trace_id=abc123
/// ```
///
/// With terminal colors when stderr is a TTY.
mod logfmt {
    use std::collections::HashMap;
    use std::fmt::Write;
    use std::io::{self, IsTerminal};

    use tracing::field::{Field, Visit};
    use tracing::Subscriber;
    use tracing_subscriber::layer::Context;
    use tracing_subscriber::registry::LookupSpan;
    use tracing_subscriber::Layer;

    /// Stores span fields so we can access them when formatting events.
    #[derive(Default)]
    struct SpanFields(HashMap<&'static str, String>);

    /// Visitor that collects fields into a HashMap.
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

    /// Fields we pick from spans, in display order.
    /// (span_field_name, logfmt_key)
    const SPAN_FIELDS: &[(&str, &str)] = &[
        ("http.method", "method"),
        ("http.target", "path"),
        ("http.status_code", "status"),
        ("sqlpage.file", "file"),
        ("http.client_ip", "client_ip"),
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

    pub(super) struct LogfmtLayer {
        use_colors: bool,
    }

    impl LogfmtLayer {
        pub fn new() -> Self {
            Self {
                use_colors: io::stderr().is_terminal(),
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

            // 1. ts
            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
            if colors {
                let _ = write!(buf, "{DIM}ts={now}{RESET}");
            } else {
                let _ = write!(buf, "ts={now}");
            }

            // 2. level (with color)
            let level = event.metadata().level();
            if colors {
                let (color, label) = match *level {
                    tracing::Level::ERROR => (RED, "error"),
                    tracing::Level::WARN => (YELLOW, "warn"),
                    tracing::Level::INFO => (GREEN, "info"),
                    tracing::Level::DEBUG => (BLUE, "debug"),
                    tracing::Level::TRACE => (DIM, "trace"),
                };
                let _ = write!(buf, " {BOLD}{color}level={label}{RESET}");
            } else {
                let _ = write!(buf, " level={}", level.as_str().to_ascii_lowercase());
            }

            // 3. target — for bridged log events, use log.target; otherwise metadata target
            let mut event_fields = HashMap::new();
            event.record(&mut FieldCollector(&mut event_fields));

            let target = event_fields
                .get("log.target")
                .map_or_else(|| event.metadata().target(), String::as_str);
            if colors {
                let _ = write!(buf, " {DIM}target={target}{RESET}");
            } else {
                let _ = write!(buf, " target={target}");
            }

            // 4. msg — for terminal, preserve multi-line formatting (e.g. SQL
            //    error highlighting with arrows); for machine output, flatten.
            let msg = event_fields.get("message");
            let multiline_msg = colors && msg.is_some_and(|m| m.contains('\n'));
            if !multiline_msg {
                if let Some(msg) = msg {
                    write_logfmt_value(&mut buf, "msg", msg);
                }
            }

            // 5. Selected span fields in order
            let mut seen = [false; SPAN_FIELDS.len()];
            if let Some(scope) = ctx.event_scope(event) {
                for span in scope {
                    let ext = span.extensions();
                    if let Some(fields) = ext.get::<SpanFields>() {
                        for (i, &(span_key, logfmt_key)) in SPAN_FIELDS.iter().enumerate() {
                            if !seen[i] {
                                if let Some(val) = fields.0.get(span_key) {
                                    write_logfmt_value(&mut buf, logfmt_key, val);
                                    seen[i] = true;
                                }
                            }
                        }
                    }
                }
            }

            // 6. trace_id from OpenTelemetry span context (only if valid)
            if let Some(scope) = ctx.event_scope(event) {
                'outer: for span in scope {
                    let ext = span.extensions();
                    if let Some(otel_data) = ext.get::<tracing_opentelemetry::OtelData>() {
                        if let Some(trace_id) = otel_data.trace_id() {
                            let id = trace_id.to_string();
                            if id != INVALID_TRACE_ID {
                                if colors {
                                    let _ = write!(buf, " {DIM}trace_id={id}{RESET}");
                                } else {
                                    let _ = write!(buf, " trace_id={id}");
                                }
                                break 'outer;
                            }
                        }
                    }
                }
            }

            buf.push('\n');

            // For multi-line messages on a terminal, print the message below
            // the metadata line with its original formatting preserved.
            if multiline_msg {
                if let Some(msg) = msg {
                    buf.push_str(msg);
                    buf.push('\n');
                }
            }

            // Write atomically to stderr
            let _ = io::Write::write_all(&mut io::stderr().lock(), buf.as_bytes());
        }
    }

    /// Write a logfmt key=value pair, quoting the value if it contains spaces or special chars.
    fn write_logfmt_value(buf: &mut String, key: &str, value: &str) {
        let needs_quoting = value.contains(' ')
            || value.contains('"')
            || value.contains('=')
            || value.contains('\n')
            || value.is_empty();

        if needs_quoting {
            let escaped = value.replace('\n', " ").replace('"', "\\\"");
            let _ = write!(buf, " {key}=\"{escaped}\"");
        } else {
            let _ = write!(buf, " {key}={value}");
        }
    }
}
