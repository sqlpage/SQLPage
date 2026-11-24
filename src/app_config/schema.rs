use super::{CERTIFICATE_CACHE_DIRECTORY, Duration};
use crate::webserver::database::default_max_connections;
use schemars::Schema;
use serde_json::Value;
use sqlx::any::AnyKind;

pub(super) fn working_directory() -> &'static str {
    "."
}

pub(super) fn certificate_cache_directory() -> &'static str {
    CERTIFICATE_CACHE_DIRECTORY
}

pub(super) fn timeout(duration: Duration) -> String {
    format!(
        "Disabled for SQLite, {} seconds for other databases.",
        duration.as_secs()
    )
}

pub(super) fn pool_sizes() -> String {
    [
        ("PostgreSQL", AnyKind::Postgres, false),
        ("MySQL", AnyKind::MySql, false),
        ("SQLite on disk", AnyKind::Sqlite, false),
        ("SQLite in memory", AnyKind::Sqlite, true),
        ("MSSQL", AnyKind::Mssql, false),
        ("ODBC", AnyKind::Odbc, false),
    ]
    .map(|(label, kind, memory)| format!("{label} {}", default_max_connections(kind, memory)))
    .join(", ")
}

/// JSON Schema defaults represent a single value supplied when the property is omitted.
/// Conditional defaults have a human-readable description, and optional settings can be unset.
pub(super) fn remove_unset_defaults(schema: &mut Schema) {
    if let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) {
        for field in properties.values_mut().filter_map(Value::as_object_mut) {
            if field.contains_key("x-default-description")
                || field.get("default") == Some(&Value::Null)
            {
                field.remove("default");
            }
        }
    }
}
