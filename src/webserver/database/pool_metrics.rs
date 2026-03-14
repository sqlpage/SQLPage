use opentelemetry::{global, KeyValue};
use opentelemetry::metrics::UpDownCounter;

fn get_counter() -> UpDownCounter<i64> {
    global::meter("sqlpage")
        .i64_up_down_counter("db.client.connection.count")
        .with_unit("{connection}")
        .with_description("Number of connections in the database pool.")
        .build()
}

pub fn on_acquire(db_system_name: &'static str) {
    let counter = get_counter();
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

pub fn on_release(db_system_name: &'static str) {
    let counter = get_counter();
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

pub fn on_connect(db_system_name: &'static str) {
    let counter = get_counter();
    counter.add(1, &[
        KeyValue::new("db.system.name", db_system_name),
        KeyValue::new("db.client.connection.pool.name", "sqlpage"),
        KeyValue::new("db.client.connection.state", "idle"),
    ]);
}
