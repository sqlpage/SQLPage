mod connect;
mod csv_import;
pub mod execute_queries;
pub mod migrations;
mod sql;
mod sqlpage_functions;
mod syntax_tree;

mod error_highlighting;
mod sql_to_json;

pub use sql::{make_placeholder, ParsedSqlFile};

pub struct Database {
    pub(crate) connection: sqlx::AnyPool,
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
