use opentelemetry::{global, KeyValue};
use opentelemetry::metrics::UpDownCounter;

fn get_counter() -> UpDownCounter<i64> {
    global::meter("sqlpage")
        .i64_up_down_counter("db.client.connection.count")
        .with_unit("{connection}")
        .with_description("Number of connections in the database pool.")
        .build()
}

pub fn on_acquire() {
    let counter = get_counter();
    let db_system_name = super::get_discovered_db_type().otel_name();
    counter.add(1, &[
        KeyValue::new("db.system.name", db_system_name),
        KeyValue::new("db.client.connection.pool.name", "sqlpage"),
        KeyValue::new("db.client.connection.state", "used"),
    ]);
    counter.add(-1, &[
        KeyValue::new("db.system.name", db_system_name),
        KeyValue::new("db.client.connection.pool.name", "sqlpage"),
        KeyValue::new("db.client.connection.state", "idle"),
    ]);
}

pub fn on_release() {
    let counter = get_counter();
    let db_system_name = super::get_discovered_db_type().otel_name();
    counter.add(-1, &[
        KeyValue::new("db.system.name", db_system_name),
        KeyValue::new("db.client.connection.pool.name", "sqlpage"),
        KeyValue::new("db.client.connection.state", "used"),
    ]);
    counter.add(1, &[
        KeyValue::new("db.system.name", db_system_name),
        KeyValue::new("db.client.connection.pool.name", "sqlpage"),
        KeyValue::new("db.client.connection.state", "idle"),
    ]);
}

pub fn on_connect() {
    let counter = get_counter();
    let db_system_name = super::get_discovered_db_type().otel_name();
    counter.add(1, &[
        KeyValue::new("db.system.name", db_system_name),
        KeyValue::new("db.client.connection.pool.name", "sqlpage"),
        KeyValue::new("db.client.connection.state", "idle"),
    ]);
}
