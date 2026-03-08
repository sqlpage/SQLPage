//! OpenTelemetry initialization and shutdown.
//!
//! When `OTEL_EXPORTER_OTLP_ENDPOINT` is set, sets up a full tracing pipeline
//! with OTLP export. Otherwise, falls back to `env_logger` as before.

use std::env;
use std::sync::OnceLock;

use opentelemetry_sdk::trace::SdkTracerProvider;

static TRACER_PROVIDER: OnceLock<SdkTracerProvider> = OnceLock::new();

/// Initializes logging / tracing. Returns `true` if `OTel` was activated.
#[must_use]
pub fn init_telemetry() -> bool {
    let otel_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
    let otel_active = otel_endpoint.as_deref().is_some_and(|v| !v.is_empty());

    if otel_active {
        init_otel_tracing();
    } else {
        init_env_logger();
    }

    otel_active
}

/// Shuts down the `OTel` tracer provider, flushing pending spans.
pub fn shutdown_telemetry() {
    if let Some(provider) = TRACER_PROVIDER.get() {
        if let Err(e) = provider.shutdown() {
            eprintln!("Error shutting down tracer provider: {e}");
        }
    }
}

fn init_env_logger() {
    let env =
        env_logger::Env::new().default_filter_or("sqlpage=info,actix_web::middleware::logger=info");
    let mut logging = env_logger::Builder::from_env(env);
    logging.format_timestamp_millis();
    logging.init();
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

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "sqlpage=info,actix_web=info,tracing_actix_web=info".into());

    let fmt_layer = json_subscriber::layer()
        .with_current_span(true)
        .with_span_list(false)
        .with_opentelemetry_ids(true);

    // Build the subscriber and set it as global default.
    // We use tracing_log::LogTracer to bridge log::* → tracing,
    // and set_global_default (not .init()) to avoid double-setting the log logger.
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");
    tracing_log::LogTracer::init().expect("Failed to set log→tracing bridge");
}
