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

/// Supported database types in `SQLPage`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedDatabase {
    Sqlite,
    Postgres,
    MySql,
    Mssql,
    Odbc,
}

impl SupportedDatabase {
    /// Detect the database type from a connection's `dbms_name`
    #[must_use]
    pub fn from_dbms_name(dbms_name: &str) -> Option<Self> {
        match dbms_name.to_lowercase().as_str() {
            "sqlite" | "sqlite3" => Some(Self::Sqlite),
            "postgres" | "postgresql" => Some(Self::Postgres),
            "mysql" | "mariadb" => Some(Self::MySql),
            "mssql" | "sql server" | "microsoft sql server" => Some(Self::Mssql),
            "odbc" => Some(Self::Odbc),
            _ => None,
        }
    }

    /// Convert from sqlx `AnyKind` to our enum
    #[must_use]
    pub fn from_any_kind(kind: AnyKind) -> Self {
        match kind {
            AnyKind::Sqlite => Self::Sqlite,
            AnyKind::Postgres => Self::Postgres,
            AnyKind::MySql => Self::MySql,
            AnyKind::Mssql => Self::Mssql,
            AnyKind::Odbc => Self::Odbc,
        }
    }

    /// Get the display name for the database
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

pub struct Database {
    pub connection: sqlx::AnyPool,
}

impl Database {
    pub async fn close(&self) -> anyhow::Result<()> {
        log::info!("Closing all database connections...");
        self.connection.close().await;
        Ok(())
    }

    /// Detect the database type using the connection's `dbms_name`
    pub async fn detect_database_type(&self) -> anyhow::Result<SupportedDatabase> {
        let mut conn = self.connection.acquire().await?;
        let dbms_name = conn.dbms_name().await?;

        if let Some(db_type) = SupportedDatabase::from_dbms_name(&dbms_name) {
            log::debug!(
                "Detected database type: {} from dbms_name: {}",
                db_type.display_name(),
                dbms_name
            );
            Ok(db_type)
        } else {
            log::warn!("Unknown database type from dbms_name: {dbms_name}");
            // Fallback to AnyKind detection
            Ok(SupportedDatabase::from_any_kind(self.connection.any_kind()))
        }
    }

    /// Get the database type using the fallback method (`AnyKind`)
    #[must_use]
    pub fn get_database_type(&self) -> SupportedDatabase {
        SupportedDatabase::from_any_kind(self.connection.any_kind())
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
pub fn make_placeholder(db_kind: AnyKind, arg_number: usize) -> String {
    if let Some((_, placeholder)) = DB_PLACEHOLDERS.iter().find(|(kind, _)| *kind == db_kind) {
        match *placeholder {
            DbPlaceHolder::PrefixedNumber { prefix } => format!("{prefix}{arg_number}"),
            DbPlaceHolder::Positional { placeholder } => placeholder.to_string(),
        }
    } else {
        unreachable!("missing db_kind: {db_kind:?} in DB_PLACEHOLDERS ({DB_PLACEHOLDERS:?})")
    }
}
