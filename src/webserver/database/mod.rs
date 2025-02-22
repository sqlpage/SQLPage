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

pub struct Database {
    pub connection: sqlx::AnyPool,
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
