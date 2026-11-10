//! Database-specific parser dialects and bind-placeholder syntax.

use sqlparser::dialect::{
    Dialect, DuckDbDialect, GenericDialect, MsSqlDialect, MySqlDialect, OracleDialect,
    PostgreSqlDialect, SQLiteDialect, SnowflakeDialect,
};
use sqlx::any::AnyKind;

use crate::webserver::database::SupportedDatabase;

/// Native bind-placeholder forms supported by `sqlx` database backends.
#[derive(Clone, Copy)]
pub(super) enum PlaceholderStyle {
    Numbered { prefix: &'static str },
    Positional { token: &'static str },
}

/// Returns the `sqlparser` dialect matching the configured database.
pub(super) fn parser_dialect(database: SupportedDatabase) -> Box<dyn Dialect> {
    match database {
        SupportedDatabase::Duckdb => Box::new(DuckDbDialect {}),
        SupportedDatabase::Oracle => Box::new(OracleDialect {}),
        SupportedDatabase::Postgres => Box::new(PostgreSqlDialect {}),
        SupportedDatabase::Generic => Box::new(GenericDialect {}),
        SupportedDatabase::Mssql => Box::new(MsSqlDialect {}),
        SupportedDatabase::MySql => Box::new(MySqlDialect {}),
        SupportedDatabase::Sqlite => Box::new(SQLiteDialect {}),
        SupportedDatabase::Snowflake => Box::new(SnowflakeDialect {}),
    }
}

/// Returns the native placeholder syntax emitted into rewritten SQL.
pub(super) fn placeholder_style(kind: AnyKind) -> PlaceholderStyle {
    match kind {
        AnyKind::Sqlite => PlaceholderStyle::Numbered { prefix: "?" },
        AnyKind::Postgres => PlaceholderStyle::Numbered { prefix: "$" },
        AnyKind::Mssql => PlaceholderStyle::Numbered { prefix: "@p" },
        AnyKind::MySql | AnyKind::Odbc => PlaceholderStyle::Positional { token: "?" },
    }
}
