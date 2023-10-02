mod sql;
mod sql_pseudofunctions;
mod sql_to_json;
pub mod migrations;
mod connect;
pub mod execute_queries;

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

struct PreparedStatement {
    statement: sqlx::any::AnyStatement<'static>,
    parameters: Vec<sql_pseudofunctions::StmtParam>,
}

impl std::fmt::Display for PreparedStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use sqlx::Statement;
        write!(f, "{}", self.statement.sql())
    }
}
