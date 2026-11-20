//! Database-specific parser dialects and bind-placeholder syntax.

use crate::webserver::database::{DbInfo, SupportedDatabase};
use sqlparser::dialect::{
    Dialect, DuckDbDialect, GenericDialect, MsSqlDialect, MySqlDialect, OracleDialect,
    PostgreSqlDialect, SQLiteDialect, SnowflakeDialect,
};

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

impl DbInfo {
    /// A numbered placeholder can refer to the same binding more than once.
    pub(super) fn numbered_placeholder_prefix(self) -> Option<&'static str> {
        match self {
            Self::Native(SupportedDatabase::Sqlite) => Some("?"),
            Self::Native(SupportedDatabase::Postgres) => Some("$"),
            Self::Native(SupportedDatabase::Mssql) => Some("@p"),
            _ => None,
        }
    }

    #[must_use]
    pub fn placeholder(self, number: usize) -> String {
        match self.numbered_placeholder_prefix() {
            Some(prefix) => format!("{prefix}{number}"),
            None => "?".to_owned(),
        }
    }
}
