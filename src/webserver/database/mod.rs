mod connect;
mod csv_import;
pub mod execute_queries;
pub mod migrations;
mod sql;
mod sqlpage_functions;
mod syntax_tree;

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

#[must_use]
pub fn highlight_sql_error(
    context: &str,
    query: &str,
    db_err: sqlx::error::Error,
) -> anyhow::Error {
    use std::fmt::Write;
    let mut msg = format!("{context}:\n");
    let offset = if let sqlx::error::Error::Database(db_err) = &db_err {
        db_err.offset()
    } else {
        None
    };
    if let Some(mut offset) = offset {
        for (line_no, line) in query.lines().enumerate() {
            if offset > line.len() {
                offset -= line.len() + 1;
            } else {
                writeln!(msg, "{line}").unwrap();
                writeln!(msg, "{}⬆️", " ".repeat(offset)).unwrap();
                write!(msg, "line {}, character {offset}", line_no + 1).unwrap();
                break;
            }
        }
    } else {
        write!(msg, "{}", query.lines().next().unwrap_or_default()).unwrap();
    }
    anyhow::Error::new(db_err).context(msg)
}
