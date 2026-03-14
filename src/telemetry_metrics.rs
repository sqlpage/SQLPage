use opentelemetry::global;
use opentelemetry::metrics::Histogram;
use opentelemetry_semantic_conventions::metric as otel_metric;

#[derive(Clone)]
pub struct TelemetryMetrics {
    pub http_request_duration: Histogram<f64>,
    pub db_query_duration: Histogram<f64>,
}

impl Default for TelemetryMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryMetrics {
    #[must_use]
    pub fn new() -> Self {
        let meter = global::meter("sqlpage");
        let http_request_duration = meter
            .f64_histogram(otel_metric::HTTP_SERVER_REQUEST_DURATION)
            .with_unit("s")
            .with_description("Duration of HTTP requests processed by the server.")
            .build();
        let db_query_duration = meter
            .f64_histogram(otel_metric::DB_CLIENT_OPERATION_DURATION)
            .with_unit("s")
            .with_description("Duration of executing SQL queries.")
            .build();

        Self {
            http_request_duration,
            db_query_duration,
        }
    }
}
