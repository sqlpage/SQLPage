pub mod blob_to_data_url;
mod connect;
mod csv_import;
pub mod driver;
pub mod execute_queries;
pub mod migrations;
mod sql;
mod sqlpage_expr;
mod sqlpage_functions;

mod error_highlighting;
mod sql_to_json;
#[cfg(feature = "unstable-sqlx-next")]
mod sqlx_next;

pub use sql::SqlFile;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScalarSubqueryBehavior {
    FirstRow,
    ErrorOnMultipleRows,
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

    /// Mirrors how the backend handles a scalar subquery that returns multiple rows.
    fn scalar_subquery_behavior(self) -> ScalarSubqueryBehavior {
        match self {
            Self::Sqlite => ScalarSubqueryBehavior::FirstRow,
            _ => ScalarSubqueryBehavior::ErrorOnMultipleRows,
        }
    }

    fn concat_function_null_behavior(self) -> sqlpage_expr::ConcatNullBehavior {
        use sqlpage_expr::ConcatNullBehavior::{IgnoreNull, PropagateNull};

        match self {
            Self::Sqlite | Self::Duckdb | Self::Oracle | Self::Postgres | Self::Mssql => IgnoreNull,
            Self::MySql | Self::Snowflake | Self::Generic => PropagateNull,
        }
    }

    fn concat_operator_null_behavior(self) -> sqlpage_expr::ConcatNullBehavior {
        use sqlpage_expr::ConcatNullBehavior::{IgnoreNull, PropagateNull};

        match self {
            Self::Oracle | Self::Mssql => IgnoreNull,
            _ => PropagateNull,
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
    pub connection: driver::DatabasePool,
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

impl DbInfo {
    fn new(dbms_name: String, kind: AnyKind) -> Self {
        Self {
            database_type: SupportedDatabase::from_dbms_name(&dbms_name),
            dbms_name,
            kind,
        }
    }
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
        write!(f, "{:?}", self.info.kind)
    }
}

#[inline]
#[must_use]
pub fn make_placeholder(dbms: AnyKind, arg_number: usize) -> String {
    match dbms {
        AnyKind::Sqlite => format!("?{arg_number}"),
        AnyKind::Postgres => format!("${arg_number}"),
        AnyKind::Mssql => format!("@p{arg_number}"),
        AnyKind::MySql | AnyKind::Odbc => "?".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::sqlpage_expr::ConcatNullBehavior::{IgnoreNull, PropagateNull};
    use super::{ScalarSubqueryBehavior, SupportedDatabase};

    #[test]
    fn scalar_subquery_behavior_matches_backends() {
        assert_eq!(
            SupportedDatabase::Sqlite.scalar_subquery_behavior(),
            ScalarSubqueryBehavior::FirstRow
        );
        for database in [
            SupportedDatabase::Duckdb,
            SupportedDatabase::Oracle,
            SupportedDatabase::Postgres,
            SupportedDatabase::MySql,
            SupportedDatabase::Mssql,
            SupportedDatabase::Snowflake,
            SupportedDatabase::Generic,
        ] {
            assert_eq!(
                database.scalar_subquery_behavior(),
                ScalarSubqueryBehavior::ErrorOnMultipleRows
            );
        }
    }

    #[test]
    fn concat_null_behavior_matches_backends() {
        for database in [
            SupportedDatabase::Sqlite,
            SupportedDatabase::Duckdb,
            SupportedDatabase::Oracle,
            SupportedDatabase::Postgres,
            SupportedDatabase::Mssql,
        ] {
            assert_eq!(database.concat_function_null_behavior(), IgnoreNull);
        }
        for database in [
            SupportedDatabase::MySql,
            SupportedDatabase::Snowflake,
            SupportedDatabase::Generic,
        ] {
            assert_eq!(database.concat_function_null_behavior(), PropagateNull);
        }
    }

    #[test]
    fn concat_operator_null_behavior_matches_backends() {
        for database in [SupportedDatabase::Oracle, SupportedDatabase::Mssql] {
            assert_eq!(database.concat_operator_null_behavior(), IgnoreNull);
        }
        for database in [
            SupportedDatabase::Sqlite,
            SupportedDatabase::Duckdb,
            SupportedDatabase::Postgres,
            SupportedDatabase::MySql,
            SupportedDatabase::Snowflake,
            SupportedDatabase::Generic,
        ] {
            assert_eq!(database.concat_operator_null_behavior(), PropagateNull);
        }
    }
}
