use opentelemetry::metrics::UpDownCounter;
use opentelemetry::{global, KeyValue};
use opentelemetry_semantic_conventions::attribute as otel;
use opentelemetry_semantic_conventions::metric as otel_metric;

fn get_counter() -> UpDownCounter<i64> {
    global::meter("sqlpage")
        .i64_up_down_counter(otel_metric::DB_CLIENT_CONNECTION_COUNT)
        .with_unit("{connection}")
        .with_description("Number of connections in the database pool.")
        .build()
}

pub fn on_acquire(db_system_name: &'static str) {
    let counter = get_counter();
    counter.add(
        1,
        &[
            KeyValue::new(otel::DB_SYSTEM_NAME, db_system_name),
            KeyValue::new(otel::DB_CLIENT_CONNECTION_POOL_NAME, "sqlpage"),
            KeyValue::new(otel::DB_CLIENT_CONNECTION_STATE, "used"),
        ],
    );
    counter.add(
        -1,
        &[
            KeyValue::new(otel::DB_SYSTEM_NAME, db_system_name),
            KeyValue::new(otel::DB_CLIENT_CONNECTION_POOL_NAME, "sqlpage"),
            KeyValue::new(otel::DB_CLIENT_CONNECTION_STATE, "idle"),
        ],
    );
}

pub fn on_release(db_system_name: &'static str) {
    let counter = get_counter();
    counter.add(
        -1,
        &[
            KeyValue::new(otel::DB_SYSTEM_NAME, db_system_name),
            KeyValue::new(otel::DB_CLIENT_CONNECTION_POOL_NAME, "sqlpage"),
            KeyValue::new(otel::DB_CLIENT_CONNECTION_STATE, "used"),
        ],
    );
    counter.add(
        1,
        &[
            KeyValue::new(otel::DB_SYSTEM_NAME, db_system_name),
            KeyValue::new(otel::DB_CLIENT_CONNECTION_POOL_NAME, "sqlpage"),
            KeyValue::new(otel::DB_CLIENT_CONNECTION_STATE, "idle"),
        ],
    );
}

pub fn on_connect(db_system_name: &'static str) {
    let counter = get_counter();
    counter.add(
        1,
        &[
            KeyValue::new(otel::DB_SYSTEM_NAME, db_system_name),
            KeyValue::new(otel::DB_CLIENT_CONNECTION_POOL_NAME, "sqlpage"),
            KeyValue::new(otel::DB_CLIENT_CONNECTION_STATE, "idle"),
        ],
    );
}
