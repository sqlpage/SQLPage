//! Assigns selected expressions to the database or per-row `SQLPage` evaluation.

use sqlparser::ast::{
    BinaryOperator, DataType, Expr as SqlExpr, FunctionArg, FunctionArgExpr, FunctionArguments,
    Ident, ObjectNamePart, SelectItem, SetExpr, Statement as SqlStatement,
};

use super::sqlpage_expression::{
    SqlPageExpressionContext, build_emulated, build_sqlpage_expr, emulated_function,
    recognize_sqlpage_function, take_expression_arguments,
};
use crate::webserver::database::sqlpage_expr::{RowExpr, RowInputId, SqlPageExpr};
use crate::webserver::database::{DbInfo, SupportedDatabase};

pub(super) const PRIVATE_ROW_INPUT_PREFIX: &str = "__sqlpage_input_";

/// Ownership assigned to one selected expression before database lowering.
#[allow(clippy::large_enum_variant)]
pub(super) enum PartitionedProjection {
    /// The database evaluates this expression as part of its `SELECT`.
    Database(SqlExpr),
    /// `SQLPage` evaluates this expression for every row returned by the database.
    PerRow(RowExpr),
}

/// Partitions selected expressions and records their private database row inputs.
pub(super) struct ProjectionPartitioner<'a> {
    database: &'a DbInfo,
    pub(super) row_input_json: Vec<bool>,
    pub(super) private_projection: Vec<SelectItem>,
}

struct PerRowContext<'a> {
    row_input_json: &'a mut Vec<bool>,
    private_projection: &'a mut Vec<SelectItem>,
}

impl SqlPageExpressionContext for PerRowContext<'_> {
    type Input = RowInputId;

    fn use_database_expr(&mut self, expression: SqlExpr) -> anyhow::Result<RowExpr> {
        let index = self.row_input_json.len();
        let decode_as_json = is_json_expression(&expression);
        self.private_projection.push(SelectItem::ExprWithAlias {
            expr: expression,
            alias: Ident::with_quote('"', format!("{PRIVATE_ROW_INPUT_PREFIX}{index}")),
        });
        self.row_input_json.push(decode_as_json);
        Ok(SqlPageExpr::Input(RowInputId::new(index)))
    }
}

impl<'a> ProjectionPartitioner<'a> {
    pub(super) fn new(database: &'a DbInfo) -> Self {
        Self {
            database,
            row_input_json: Vec::new(),
            private_projection: Vec::new(),
        }
    }

    /// Splits a projection at SQLPage-supported operations while leaving opaque
    /// database operations in the SQL AST.
    pub(super) fn partition_projection(
        &mut self,
        expression: SqlExpr,
    ) -> anyhow::Result<PartitionedProjection> {
        if !projection_is_per_row(&expression)? {
            return Ok(PartitionedProjection::Database(expression));
        }
        match expression {
            SqlExpr::Function(function) => {
                if recognize_sqlpage_function(&function)?.is_some() {
                    return self
                        .build_row_expr(SqlExpr::Function(function))
                        .map(PartitionedProjection::PerRow);
                }
                let kind = emulated_function(&function)
                    .expect("per-row function ownership was already classified");
                let arguments = take_expression_arguments(function)?
                    .into_iter()
                    .map(|argument| {
                        let projection = self.partition_projection(argument)?;
                        self.projection_into_row_expr(projection)
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;
                Ok(PartitionedProjection::PerRow(build_emulated(
                    kind,
                    arguments,
                    self.database.database_type,
                )?))
            }
            SqlExpr::BinaryOp {
                left,
                op: BinaryOperator::StringConcat,
                right,
            } => {
                let left = self.partition_projection(*left)?;
                let left = self.projection_into_row_expr(left)?;
                let right = self.partition_projection(*right)?;
                let right = self.projection_into_row_expr(right)?;
                Ok(PartitionedProjection::PerRow(SqlPageExpr::Concat {
                    arguments: vec![left, right].into_boxed_slice(),
                    null_behavior: self.database.database_type.concat_operator_null_behavior(),
                }))
            }
            SqlExpr::Nested(expression) => match self.partition_projection(*expression)? {
                PartitionedProjection::Database(expression) => Ok(PartitionedProjection::Database(
                    SqlExpr::Nested(Box::new(expression)),
                )),
                PartitionedProjection::PerRow(expression) => {
                    Ok(PartitionedProjection::PerRow(expression))
                }
            },
            _ => unreachable!("per-row expression ownership was already classified"),
        }
    }

    fn projection_into_row_expr(
        &mut self,
        projection: PartitionedProjection,
    ) -> anyhow::Result<RowExpr> {
        match projection {
            PartitionedProjection::Database(expression) => self.build_row_expr(expression),
            PartitionedProjection::PerRow(expression) => Ok(expression),
        }
    }

    fn build_row_expr(&mut self, expression: SqlExpr) -> anyhow::Result<RowExpr> {
        build_sqlpage_expr(
            self.database,
            &mut PerRowContext {
                row_input_json: &mut self.row_input_json,
                private_projection: &mut self.private_projection,
            },
            expression,
        )
    }
}

/// Determines ownership before mutation so promoted row inputs retain source order.
fn projection_is_per_row(expression: &SqlExpr) -> anyhow::Result<bool> {
    match expression {
        SqlExpr::Function(function) => {
            if recognize_sqlpage_function(function)?.is_some() {
                return Ok(true);
            }
            if emulated_function(function).is_none() {
                return Ok(false);
            }
            let FunctionArguments::List(arguments) = &function.args else {
                anyhow::bail!("Unsupported arguments to {}", function.name);
            };
            if arguments.duplicate_treatment.is_some() || !arguments.clauses.is_empty() {
                anyhow::bail!("Unsupported arguments to {}", function.name);
            }
            for argument in &arguments.args {
                let FunctionArg::Unnamed(FunctionArgExpr::Expr(expression)) = argument else {
                    anyhow::bail!("Named and wildcard function arguments are not supported");
                };
                if projection_is_per_row(expression)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        SqlExpr::BinaryOp {
            left,
            op: BinaryOperator::StringConcat,
            right,
        } => Ok(projection_is_per_row(left)? || projection_is_per_row(right)?),
        SqlExpr::Nested(expression) => projection_is_per_row(expression),
        _ => Ok(false),
    }
}

pub(super) fn detect_public_json_columns(
    statement: &SqlStatement,
    database: SupportedDatabase,
) -> Box<[String]> {
    if matches!(
        database,
        SupportedDatabase::Postgres | SupportedDatabase::Mssql
    ) {
        return Box::new([]);
    }
    let SqlStatement::Query(query) = statement else {
        return Box::new([]);
    };
    let SetExpr::Select(select) = query.body.as_ref() else {
        return Box::new([]);
    };
    select
        .projection
        .iter()
        .filter_map(|item| match item {
            SelectItem::ExprWithAlias { expr, alias } if is_json_expression(expr) => {
                (!alias.value.starts_with(PRIVATE_ROW_INPUT_PREFIX)).then(|| alias.value.clone())
            }
            _ => None,
        })
        .collect()
}

fn is_json_expression(expression: &SqlExpr) -> bool {
    match expression {
        SqlExpr::Function(function) => {
            let [ObjectNamePart::Identifier(name)] = function.name.0.as_slice() else {
                return false;
            };
            matches!(
                name.value.to_ascii_lowercase().as_str(),
                "json_object"
                    | "json_array"
                    | "json_build_object"
                    | "json_build_array"
                    | "to_json"
                    | "to_jsonb"
                    | "json_agg"
                    | "jsonb_agg"
                    | "json_arrayagg"
                    | "json_objectagg"
                    | "json_group_array"
                    | "json_group_object"
                    | "json"
                    | "jsonb"
            )
        }
        SqlExpr::Cast { data_type, .. } => matches!(data_type, DataType::JSON | DataType::JSONB),
        _ => false,
    }
}
