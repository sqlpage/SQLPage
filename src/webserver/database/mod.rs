pub mod blob_to_data_url;
mod connect;
mod csv_import;
pub mod execute_queries;
pub mod migrations;
mod sql;
mod sqlpage_functions;
mod syntax_tree;

mod error_highlighting;
mod sql_to_json;

pub use sql::ParsedSqlFile;
use sql::{DbPlaceHolder, DB_PLACEHOLDERS};
use sqlx::any::AnyKind;
// SupportedDatabase is defined in this module

/// Supported database types in `SQLPage`. Represents an actual DBMS, not a sqlx backend kind (like "Odbc")
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedDatabase {
    Sqlite,
    Duckdb,
    Oracle,
    Postgres,
    MySql,
    Mssql,
    Snowflake,
    Generic,
}

impl SupportedDatabase {
    /// Detect the database type from a connection's `dbms_name`
    #[must_use]
    pub fn from_dbms_name(dbms_name: &str) -> Self {
        match dbms_name.to_lowercase().as_str() {
            "sqlite" | "sqlite3" => Self::Sqlite,
            "duckdb" | "d\0\0\0\0\0" => Self::Duckdb, // ducksdb incorrectly truncates the db name: https://github.com/duckdb/duckdb-odbc/issues/350
            "oracle" => Self::Oracle,
            "postgres" | "postgresql" => Self::Postgres,
            "mysql" | "mariadb" => Self::MySql,
            "mssql" | "sql server" | "microsoft sql server" => Self::Mssql,
            "snowflake" => Self::Snowflake,
            _ => Self::Generic,
        }
    }

    /// Get the display name for the database
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Sqlite => "SQLite",
            Self::Duckdb => "DuckDB",
            Self::Oracle => "Oracle",
            Self::Postgres => "PostgreSQL",
            Self::MySql => "MySQL",
            Self::Mssql => "Microsoft SQL Server",
            Self::Snowflake => "Snowflake",
            Self::Generic => "Generic",
        }
    }

    /// Returns the `OTel` `db.system.name` well-known value.
    /// See <https://opentelemetry.io/docs/specs/semconv/registry/attributes/db/#db-system-name>
    #[must_use]
    pub fn otel_name(self) -> &'static str {
        Self::otel_name_from_kind(self)
    }

    #[must_use]
    pub fn otel_name_from_kind(kind: impl Into<SupportedDatabase>) -> &'static str {
        match kind.into() {
            Self::Sqlite => "sqlite",
            Self::Duckdb => "duckdb",
            Self::Oracle => "oracle.db",
            Self::Postgres => "postgresql",
            Self::MySql => "mysql",
            Self::Mssql => "microsoft.sql_server",
            Self::Snowflake => "snowflake",
            Self::Generic => "other_sql",
        }
    }
}

impl From<AnyKind> for SupportedDatabase {
    fn from(kind: AnyKind) -> Self {
        match kind {
            AnyKind::Postgres => Self::Postgres,
            AnyKind::MySql => Self::MySql,
            AnyKind::Sqlite => Self::Sqlite,
            AnyKind::Mssql => Self::Mssql,
            AnyKind::Odbc => Self::Generic,
        }
    }
}

pub struct Database {
    pub connection: sqlx::AnyPool,
    pub info: DbInfo,
}

#[derive(Debug, Clone)]
pub struct DbInfo {
    pub dbms_name: String,
    /// The actual database we are connected to. Can be "Generic" when using an unknown ODBC driver
    pub database_type: SupportedDatabase,
    /// The sqlx database backend we are using. Can be "Odbc", in which case we need to use `database_type` to know what database we are actually using.
    pub kind: AnyKind,
}

impl Database {
    pub async fn close(&self) -> anyhow::Result<()> {
        log::info!("Closing all database connections...");
        self.connection.close().await;
        Ok(())
    }
}

#[derive(Debug)]
pub enum DbItem {
    Row(serde_json::Value),
    FinishedQuery,
    Error(anyhow::Error),
}

impl std::fmt::Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.connection.any_kind())
    }
}

#[inline]
#[must_use]
pub fn make_placeholder(dbms: AnyKind, arg_number: usize) -> String {
    if let Some((_, placeholder)) = DB_PLACEHOLDERS.iter().find(|(kind, _)| *kind == dbms) {
        match *placeholder {
            DbPlaceHolder::PrefixedNumber { prefix } => format!("{prefix}{arg_number}"),
            DbPlaceHolder::Positional { placeholder } => placeholder.to_string(),
        }
    } else {
        unreachable!("missing dbms: {dbms:?} in DB_PLACEHOLDERS ({DB_PLACEHOLDERS:?})")
    }
}
