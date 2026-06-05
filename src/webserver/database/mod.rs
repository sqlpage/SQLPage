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
use sql::{DB_PLACEHOLDERS, DbPlaceHolder};
// SupportedDatabase is defined in this module

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DbKind {
    Sqlite,
    Postgres,
    MySql,
    Mssql,
    Odbc,
}

impl DbKind {
    #[must_use]
    pub fn from_database_url(database_url: &str) -> Self {
        let lower = database_url.to_ascii_lowercase();
        if lower.starts_with("postgres://") || lower.starts_with("postgresql://") {
            Self::Postgres
        } else if lower.starts_with("mysql://") || lower.starts_with("mariadb://") {
            Self::MySql
        } else if lower.starts_with("sqlite:") {
            Self::Sqlite
        } else if lower.starts_with("mssql://") || lower.starts_with("sqlserver://") {
            Self::Mssql
        } else {
            Self::Odbc
        }
    }

    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Sqlite => "SQLite",
            Self::Postgres => "PostgreSQL",
            Self::MySql => "MySQL",
            Self::Mssql => "Microsoft SQL Server",
            Self::Odbc => "ODBC",
        }
    }
}

impl From<DbKind> for SupportedDatabase {
    fn from(kind: DbKind) -> Self {
        match kind {
            DbKind::Sqlite => Self::Sqlite,
            DbKind::Postgres => Self::Postgres,
            DbKind::MySql => Self::MySql,
            DbKind::Mssql => Self::Mssql,
            DbKind::Odbc => Self::Generic,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DatabasePool {
    Sqlite(sqlx::Pool<sqlx::Sqlite>),
    Postgres(sqlx::Pool<sqlx::Postgres>),
    MySql(sqlx::Pool<sqlx::MySql>),
    Mssql(sqlx::Pool<sqlx_sqlserver::Mssql>),
    Odbc(sqlx::Pool<sqlx_odbc::Odbc>),
}

impl DatabasePool {
    #[must_use]
    pub fn kind(&self) -> DbKind {
        match self {
            Self::Sqlite(_) => DbKind::Sqlite,
            Self::Postgres(_) => DbKind::Postgres,
            Self::MySql(_) => DbKind::MySql,
            Self::Mssql(_) => DbKind::Mssql,
            Self::Odbc(_) => DbKind::Odbc,
        }
    }

    #[must_use]
    pub fn size(&self) -> u32 {
        match self {
            Self::Sqlite(pool) => pool.size(),
            Self::Postgres(pool) => pool.size(),
            Self::MySql(pool) => pool.size(),
            Self::Mssql(pool) => pool.size(),
            Self::Odbc(pool) => pool.size(),
        }
    }

    #[must_use]
    pub fn num_idle(&self) -> usize {
        match self {
            Self::Sqlite(pool) => pool.num_idle(),
            Self::Postgres(pool) => pool.num_idle(),
            Self::MySql(pool) => pool.num_idle(),
            Self::Mssql(pool) => pool.num_idle(),
            Self::Odbc(pool) => pool.num_idle(),
        }
    }

    pub async fn close(&self) {
        match self {
            Self::Sqlite(pool) => pool.close().await,
            Self::Postgres(pool) => pool.close().await,
            Self::MySql(pool) => pool.close().await,
            Self::Mssql(pool) => pool.close().await,
            Self::Odbc(pool) => pool.close().await,
        }
    }

    pub async fn execute(&self, sql: &str) -> sqlx::Result<()> {
        match self {
            Self::Sqlite(pool) => sqlx::query::<sqlx::Sqlite>(sqlx::AssertSqlSafe(sql))
                .execute(pool)
                .await
                .map(|_| ()),
            Self::Postgres(pool) => sqlx::query::<sqlx::Postgres>(sqlx::AssertSqlSafe(sql))
                .execute(pool)
                .await
                .map(|_| ()),
            Self::MySql(pool) => sqlx::query::<sqlx::MySql>(sqlx::AssertSqlSafe(sql))
                .execute(pool)
                .await
                .map(|_| ()),
            Self::Mssql(pool) => sqlx::query::<sqlx_sqlserver::Mssql>(sqlx::AssertSqlSafe(sql))
                .execute(pool)
                .await
                .map(|_| ()),
            Self::Odbc(pool) => sqlx::query::<sqlx_odbc::Odbc>(sqlx::AssertSqlSafe(sql))
                .execute(pool)
                .await
                .map(|_| ()),
        }
    }

    pub async fn acquire(
        &self,
    ) -> sqlx::Result<crate::webserver::database::execute_queries::DbConnection> {
        use crate::webserver::database::execute_queries::DbConnection;
        match self {
            Self::Sqlite(pool) => pool.acquire().await.map(DbConnection::Sqlite),
            Self::Postgres(pool) => pool.acquire().await.map(DbConnection::Postgres),
            Self::MySql(pool) => pool.acquire().await.map(DbConnection::MySql),
            Self::Mssql(pool) => pool.acquire().await.map(DbConnection::Mssql),
            Self::Odbc(pool) => pool.acquire().await.map(DbConnection::Odbc),
        }
    }
}

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

pub struct Database {
    pub connection: DatabasePool,
    pub info: DbInfo,
}

#[derive(Debug, Clone)]
pub struct DbInfo {
    pub dbms_name: String,
    /// The actual database we are connected to. Can be "Generic" when using an unknown ODBC driver
    pub database_type: SupportedDatabase,
    /// The `SQLPage` backend we are using. Can be "Odbc", in which case we need to use `database_type` to know what database we are actually using.
    pub kind: DbKind,
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
        write!(f, "{}", self.info.database_type.display_name())
    }
}

#[inline]
#[must_use]
pub fn make_placeholder(dbms: DbKind, arg_number: usize) -> String {
    if let Some((_, placeholder)) = DB_PLACEHOLDERS.iter().find(|(kind, _)| *kind == dbms) {
        match *placeholder {
            DbPlaceHolder::PrefixedNumber { prefix } => format!("{prefix}{arg_number}"),
            DbPlaceHolder::Positional { placeholder } => placeholder.to_string(),
        }
    } else {
        unreachable!("missing dbms: {dbms:?} in DB_PLACEHOLDERS ({DB_PLACEHOLDERS:?})")
    }
}
