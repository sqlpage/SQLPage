use opentelemetry::global;
use opentelemetry::metrics::{Histogram, ObservableGauge};
use opentelemetry_semantic_conventions::attribute as otel;
use opentelemetry_semantic_conventions::metric as otel_metric;
use sqlx::AnyPool;

pub struct TelemetryMetrics {
    pub http_request_duration: Histogram<f64>,
    pub db_query_duration: Histogram<f64>,
    _pool_connection_count: ObservableGauge<i64>,
}

impl Default for TelemetryMetrics {
    fn default() -> Self {
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
        // This default is only used in tests that don't touch pool metrics.
        let pool_connection_count = meter
            .i64_observable_gauge(otel_metric::DB_CLIENT_CONNECTION_COUNT)
            .with_unit("{connection}")
            .with_description("Number of connections in the database pool.")
            .with_callback(|_| {})
            .build();

        Self {
            http_request_duration,
            db_query_duration,
            _pool_connection_count: pool_connection_count,
        }
    }
}

impl TelemetryMetrics {
    #[must_use]
    pub fn new(pool: &AnyPool, db_system_name: &'static str) -> Self {
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
        let pool_ref = pool.clone();
        let pool_connection_count = meter
            .i64_observable_gauge(otel_metric::DB_CLIENT_CONNECTION_COUNT)
            .with_unit("{connection}")
            .with_description("Number of connections in the database pool.")
            .with_callback(move |observer| {
                let size = pool_ref.size();
                let idle_u32 = u32::try_from(pool_ref.num_idle()).unwrap_or(u32::MAX);
                let used = i64::from(size.saturating_sub(idle_u32));
                let idle = i64::from(idle_u32);
                observer.observe(
                    used,
                    &[
                        opentelemetry::KeyValue::new(otel::DB_SYSTEM_NAME, db_system_name),
                        opentelemetry::KeyValue::new(
                            otel::DB_CLIENT_CONNECTION_POOL_NAME,
                            "sqlpage",
                        ),
                        opentelemetry::KeyValue::new(otel::DB_CLIENT_CONNECTION_STATE, "used"),
                    ],
                );
                observer.observe(
                    idle,
                    &[
                        opentelemetry::KeyValue::new(otel::DB_SYSTEM_NAME, db_system_name),
                        opentelemetry::KeyValue::new(
                            otel::DB_CLIENT_CONNECTION_POOL_NAME,
                            "sqlpage",
                        ),
                        opentelemetry::KeyValue::new(otel::DB_CLIENT_CONNECTION_STATE, "idle"),
                    ],
                );
            })
            .build();

        Self {
            http_request_duration,
            db_query_duration,
            _pool_connection_count: pool_connection_count,
        }
    }
}
