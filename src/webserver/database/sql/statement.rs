//! Immutable SQL-file statements consumed by the executor.

use std::path::PathBuf;

use super::super::csv_import::CsvImport;
use super::super::sqlpage_expr::{RowExpr, StandaloneExpr};
use super::super::sqlpage_functions::functions::SqlPageFunctionName;

/// A parsed and rewritten SQL file ready for repeated execution.
#[derive(Default)]
pub struct SqlFile {
    pub(in crate::webserver::database) statements: Box<[FileStatement]>,
    pub source_path: PathBuf,
}

/// One statement in a SQL file.
#[derive(Debug)]
pub(in crate::webserver::database) enum FileStatement {
    Query(Query),
    SetVariable { target: VariableName, value: Query },
    CsvImport(CsvImport),
    Error(anyhow::Error),
}

/// A query and its original source location.
#[derive(Debug, PartialEq)]
pub(in crate::webserver::database) struct Query {
    pub body: QueryBody,
    pub source_span: SourceSpan,
}

/// The legal ways `SQLPage` obtains rows.
///
/// Keeping output expressions inside each variant prevents a synthetic row
/// from containing an expression that requires a database row input.
#[derive(Debug, PartialEq)]
pub(in crate::webserver::database) enum QueryBody {
    Database(DatabaseQuery),
    StaticSimpleSelect(StaticSimpleSelect),
}

/// A statement executed by the configured database.
#[derive(Debug, PartialEq)]
pub(in crate::webserver::database) struct DatabaseQuery {
    pub sql: String,
    /// Evaluated once, in placeholder order, before executing `sql`.
    pub bindings: Box<[StandaloneExpr]>,
    /// JSON decoding flags for the trailing private input columns.
    pub row_input_json: Box<[bool]>,
    /// Evaluated once for every returned database row.
    pub computed_columns: Box<[OutputColumn<RowExpr>]>,
    pub json_columns: Box<[String]>,
}

impl DatabaseQuery {
    /// Whether row evaluation needs the request's existing connection and
    /// must therefore wait until the database stream is closed.
    pub(crate) fn must_buffer_rows(&self) -> bool {
        self.computed_columns
            .iter()
            .any(|column| column.value.contains_function(SqlPageFunctionName::run_sql))
    }
}

/// Execution plan for a documented static simple select.
#[derive(Debug, PartialEq)]
pub(in crate::webserver::database) struct StaticSimpleSelect {
    pub columns: Box<[OutputColumn<StandaloneExpr>]>,
}

/// A named SQLPage-owned output expression.
#[derive(Debug, PartialEq, Eq)]
pub(in crate::webserver::database) struct OutputColumn<Expr> {
    pub name: String,
    pub value: Expr,
}

/// A validated variable name used as the target of a `SET` statement.
#[derive(Debug, PartialEq, Eq)]
pub(in crate::webserver::database) struct VariableName(pub String);

/// A location in the SQL source.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(in crate::webserver::database) struct SourceSpan {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

/// A line and column in the SQL source.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(in crate::webserver::database) struct SourceLocation {
    pub line: usize,
    pub column: usize,
}
