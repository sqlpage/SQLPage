//! Database-specific parser dialects and bind-placeholder syntax.

use crate::webserver::database::{DbInfo, SupportedDatabase};
use sqlparser::dialect::{
    Dialect, DuckDbDialect, GenericDialect, MsSqlDialect, MySqlDialect, OracleDialect,
    PostgreSqlDialect, SQLiteDialect, SnowflakeDialect,
};

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

impl DbInfo {
    pub(super) fn placeholder_style(self) -> PlaceholderStyle {
        match self {
            Self::Native(SupportedDatabase::Sqlite) => PlaceholderStyle::Numbered { prefix: "?" },
            Self::Native(SupportedDatabase::Postgres) => PlaceholderStyle::Numbered { prefix: "$" },
            Self::Native(SupportedDatabase::Mssql) => PlaceholderStyle::Numbered { prefix: "@p" },
            _ => PlaceholderStyle::Positional { token: "?" },
        }
    }

    #[must_use]
    pub fn placeholder(self, number: usize) -> String {
        match self.placeholder_style() {
            PlaceholderStyle::Numbered { prefix } => format!("{prefix}{number}"),
            PlaceholderStyle::Positional { token } => token.to_owned(),
        }
    }
}
