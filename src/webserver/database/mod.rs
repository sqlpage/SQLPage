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
// SupportedDatabase is defined in this module

/// Supported database types in `SQLPage`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedDatabase {
    Sqlite,
    Postgres,
    MySql,
    Mssql,
    Generic,
}

impl SupportedDatabase {
    /// Detect the database type from a connection's `dbms_name`
    #[must_use]
    pub fn from_dbms_name(dbms_name: &str) -> Self {
        match dbms_name.to_lowercase().as_str() {
            "sqlite" | "sqlite3" => Self::Sqlite,
            "postgres" | "postgresql" => Self::Postgres,
            "mysql" | "mariadb" => Self::MySql,
            "mssql" | "sql server" | "microsoft sql server" => Self::Mssql,
            _ => Self::Generic,
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
            Self::Generic => "Generic",
        }
    }
}

pub struct Database {
    pub connection: sqlx::AnyPool,
    pub database_type: SupportedDatabase,
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
pub fn make_placeholder(dbms: SupportedDatabase, arg_number: usize) -> String {
    if let Some((_, placeholder)) = DB_PLACEHOLDERS.iter().find(|(kind, _)| *kind == dbms) {
        match *placeholder {
            DbPlaceHolder::PrefixedNumber { prefix } => format!("{prefix}{arg_number}"),
            DbPlaceHolder::Positional { placeholder } => placeholder.to_string(),
        }
    } else {
        unreachable!("missing dbms: {dbms:?} in DB_PLACEHOLDERS ({DB_PLACEHOLDERS:?})")
    }
}
