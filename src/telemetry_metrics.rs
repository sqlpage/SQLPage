use opentelemetry::global;
use opentelemetry::metrics::{Histogram, ObservableGauge};
use opentelemetry_semantic_conventions::attribute as otel;
use opentelemetry_semantic_conventions::metric as otel_metric;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

struct PoolConnectionSnapshot {
    used: AtomicI64,
    idle: AtomicI64,
}

pub struct TelemetryMetrics {
    pub http_request_duration: Histogram<f64>,
    pub db_query_duration: Histogram<f64>,
    _pool_connection_count: ObservableGauge<i64>,
    pool_snapshot: Arc<PoolConnectionSnapshot>,
    db_system_name: &'static str,
}

impl Default for TelemetryMetrics {
    fn default() -> Self {
        Self::new("other_sql")
    }
}

impl TelemetryMetrics {
    #[must_use]
    pub fn new(db_system_name: &'static str) -> Self {
        let meter = global::meter("sqlpage");
        let pool_snapshot = Arc::new(PoolConnectionSnapshot {
            used: AtomicI64::new(0),
            idle: AtomicI64::new(0),
        });
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
        let snapshot_ref = Arc::clone(&pool_snapshot);
        let pool_connection_count = meter
            .i64_observable_gauge(otel_metric::DB_CLIENT_CONNECTION_COUNT)
            .with_unit("{connection}")
            .with_description("Number of connections in the database pool.")
            .with_callback(move |observer| {
                let used = snapshot_ref.used.load(Ordering::Relaxed);
                let idle = snapshot_ref.idle.load(Ordering::Relaxed);
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
            pool_snapshot,
            db_system_name,
        }
    }

    pub fn record_pool_snapshot(&self, db_system_name: &'static str, size: u32, idle: usize) {
        if db_system_name != self.db_system_name {
            return;
        }
        let idle = i64::try_from(idle).unwrap_or(i64::MAX);
        let used = i64::from(size).saturating_sub(idle);
        self.pool_snapshot.used.store(used, Ordering::Relaxed);
        self.pool_snapshot.idle.store(idle, Ordering::Relaxed);
    }
}
