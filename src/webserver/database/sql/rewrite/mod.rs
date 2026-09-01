//! Turns a parsed SQL statement into an immutable execution plan.
//!
//! The pipeline uses the same terms as `SQLPage`'s SQL documentation:
//! 1. A **static simple select** is a restricted `SELECT` that `SQLPage` can
//!    evaluate without querying the database. Its selected values are
//!    **standalone expressions**: expressions that need no returned row.
//! 2. **Projection partitioning** assigns each selected expression either to
//!    the database or to per-row `SQLPage` evaluation. Database values needed by
//!    a `SQLPage` expression become private trailing columns called row inputs.
//! 3. **Database lowering** replaces variables and `sqlpage.*` calls needed by
//!    database SQL with typed bindings and backend-specific placeholders.
//! 4. **Computed-column validation** rejects relational clauses that refer to a
//!    value computed by `SQLPage` after the database has returned its row.
//!
//! Partitioning a source projection must finish before lowering it. This
//! one-way handoff prevents generated database placeholders from returning to
//! `SQLPage` expression parsing. Numbered bindings retain source order;
//! positional bindings are finalized later in rendered SQL order.

mod computed_column_validation;
mod database_lowering;
mod projection_partitioning;
mod sqlpage_expression;
mod static_simple_select;

use sqlparser::ast::{
    Expr as SqlExpr, Ident, SelectItem, SetExpr, Statement as SqlStatement, Value,
};

use self::database_lowering::DatabaseLowerer;
use self::projection_partitioning::{
    PRIVATE_ROW_INPUT_PREFIX, PartitionedProjection, ProjectionPartitioner,
    detect_public_json_columns,
};
use super::statement::{DatabaseQuery, OutputColumn, Query, QueryBody, SourceLocation, SourceSpan};
use crate::webserver::database::DbInfo;
use crate::webserver::database::sqlpage_expr::RowExpr;

/// Rewrites one parsed statement into database SQL plus the `SQLPage` expressions
/// evaluated around it.
pub(super) fn rewrite_query(
    mut statement: SqlStatement,
    database: &DbInfo,
    semicolon: bool,
) -> anyhow::Result<Query> {
    let source_span = source_span(&statement);
    if let Some(static_select) = static_simple_select::try_plan(&mut statement, database)? {
        return Ok(Query {
            body: QueryBody::StaticSimpleSelect(static_select),
            source_span,
        });
    }

    let mut partitioner = ProjectionPartitioner::new(database);
    let mut lowerer = DatabaseLowerer::new(database);
    let computed_columns =
        partition_and_lower_top_level_projection(&mut statement, &mut partitioner, &mut lowerer)?;
    if let SqlStatement::Query(query) = &mut statement
        && let SetExpr::Select(select) = query.body.as_mut()
        && select.projection.is_empty()
    {
        select.projection.push(SelectItem::ExprWithAlias {
            expr: SqlExpr::value(Value::Null),
            alias: Ident::with_quote('"', format!("{PRIVATE_ROW_INPUT_PREFIX}anchor")),
        });
        partitioner.row_input_json.push(false);
    }

    let json_columns = detect_public_json_columns(&statement, database.database_type);
    lowerer.lower_ast(&mut statement)?;
    let bindings = lowerer.finish_bindings(&mut statement)?;
    let sql = format!("{statement}{}", if semicolon { ";" } else { "" });

    Ok(Query {
        body: QueryBody::Database(DatabaseQuery {
            sql,
            bindings,
            row_input_json: partitioner.row_input_json.into_boxed_slice(),
            computed_columns: computed_columns.into_boxed_slice(),
            json_columns,
        }),
        source_span,
    })
}

fn partition_and_lower_top_level_projection(
    statement: &mut SqlStatement,
    partitioner: &mut ProjectionPartitioner<'_>,
    lowerer: &mut DatabaseLowerer<'_>,
) -> anyhow::Result<Vec<OutputColumn<RowExpr>>> {
    let SqlStatement::Query(query) = statement else {
        return Ok(Vec::new());
    };
    let computed_columns = {
        let SetExpr::Select(select) = query.body.as_mut() else {
            return Ok(Vec::new());
        };
        computed_column_validation::reject_distinct_sqlpage_projection(select)?;

        let mut database_projection = Vec::with_capacity(select.projection.len());
        let mut computed_columns = Vec::new();
        for item in std::mem::take(&mut select.projection) {
            let (expression, alias) = match item {
                SelectItem::ExprWithAlias { expr, alias } => (expr, Some(alias)),
                SelectItem::UnnamedExpr(expr) => (expr, None),
                item => {
                    database_projection.push(item);
                    continue;
                }
            };
            let name = alias
                .as_ref()
                .map_or_else(|| expression.to_string(), |alias| alias.value.clone());
            let first_private_input = partitioner.private_projection.len();
            match partitioner.partition_projection(expression)? {
                PartitionedProjection::Database(mut expression) => {
                    lowerer.lower_ast(&mut expression)?;
                    database_projection.push(match alias {
                        Some(alias) => SelectItem::ExprWithAlias {
                            expr: expression,
                            alias,
                        },
                        None => SelectItem::UnnamedExpr(expression),
                    });
                }
                PartitionedProjection::PerRow(value) => {
                    for input in &mut partitioner.private_projection[first_private_input..] {
                        lowerer.lower_ast(input)?;
                    }
                    computed_columns.push(OutputColumn { name, value });
                }
            }
        }
        database_projection.append(&mut partitioner.private_projection);
        select.projection = database_projection;
        computed_columns
    };
    computed_column_validation::reject_references(query, &computed_columns)?;
    Ok(computed_columns)
}

fn source_span(value: &impl sqlparser::ast::Spanned) -> SourceSpan {
    let span = value.span();
    let location = |location: sqlparser::tokenizer::Location| SourceLocation {
        line: usize::try_from(location.line).unwrap_or(0),
        column: usize::try_from(location.column).unwrap_or(0),
    };
    SourceSpan {
        start: location(span.start),
        end: location(span.end),
    }
}
