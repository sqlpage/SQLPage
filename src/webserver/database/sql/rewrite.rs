//! Compiler pass that partitions parsed SQL between the database and `SQLPage`.
//!
//! `SQLPage` files contain expressions owned by two runtimes: the configured
//! database handles ordinary SQL, while variables and `sqlpage.*` functions are
//! evaluated by `SQLPage`. This module makes that boundary explicit. Values
//! needed before a database statement become typed bindings; computed
//! projections run once per returned row, with any database-owned dependencies
//! appended to the SQL projection as a private trailing column suffix. Queries
//! that require no database work become single-row `SQLPage` plans instead. The
//! distinct standalone and row expression types prevent pre-query work from
//! depending on a row that does not exist yet.
//!
//! Rewriting also renders backend-appropriate placeholders and casts, preserves
//! selected DBMS semantics for operations `SQLPage` emulates, records which
//! returned values need JSON decoding, and rejects grouping, ordering, or other
//! relational clauses that would incorrectly depend on post-database computed
//! columns. Its output is the immutable query representation consumed by
//! `execute_queries`; no request values are resolved and no functions are
//! executed here.

use std::ops::ControlFlow;
use std::str::FromStr as _;

use anyhow::{Context as _, anyhow};
use serde_json::Value as JsonValue;
use sqlparser::ast::{
    BinaryOperator, CastKind, CharacterLength, DataType, Expr as SqlExpr, Function, FunctionArg,
    FunctionArgExpr, FunctionArgumentList, FunctionArguments, GroupByExpr, Ident, ObjectName,
    ObjectNamePart, OrderByKind, SelectItem, SetExpr, Statement as SqlStatement, Value,
    ValueWithSpan, VisitMut, VisitorMut,
};
use sqlparser::tokenizer::Span;

use super::dialect::{PlaceholderStyle, placeholder_style};
use super::statement::{
    DatabaseQuery, OutputColumn, Query, QueryBody, SingleRowQuery, SourceLocation, SourceSpan,
};
use super::{extract_json_columns, is_json_expression, is_sqlpage_func};
use crate::webserver::database::sqlpage_expr::{
    NoRowInput, RowExpr, RowInputId, SqlPageExpr, StandaloneExpr, VariableRef, VariableSource,
};
use crate::webserver::database::sqlpage_functions::functions::SqlPageFunctionName;
use crate::webserver::database::{DbInfo, SupportedDatabase};

const SQLPAGE_INPUT_PREFIX: &str = "__sqlpage_input_";

/// Mutable state used while rewriting one database query.
struct QueryRewriter<'a> {
    database: &'a DbInfo,
    bindings: Vec<StandaloneExpr>,
    row_input_json: Vec<bool>,
    private_projection: Vec<SelectItem>,
    error: Option<anyhow::Error>,
}

struct ComputedAliasFinder<'a> {
    computed_columns: &'a [OutputColumn<RowExpr>],
}

impl sqlparser::ast::Visitor for ComputedAliasFinder<'_> {
    type Break = ();

    fn pre_visit_expr(&mut self, expression: &SqlExpr) -> ControlFlow<Self::Break> {
        let SqlExpr::Identifier(identifier) = expression else {
            return ControlFlow::Continue(());
        };
        if self
            .computed_columns
            .iter()
            .any(|column| identifier.value.eq_ignore_ascii_case(&column.name))
        {
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    }
}

/// Result of partitioning one projected expression.
// Keeping the owned AST inline avoids one heap allocation for every ordinary
// projected expression. The enum is short-lived inside the rewriter.
#[allow(clippy::large_enum_variant)]
enum RewrittenProjection {
    Database(SqlExpr),
    PerRow(RowExpr),
}

/// Defines how a SQL expression crossing into a SQLPage-owned expression is
/// represented at a particular evaluation site.
trait ExprEnvironment {
    type Input;

    fn use_database_expr(
        rewriter: &mut QueryRewriter<'_>,
        expression: SqlExpr,
    ) -> anyhow::Result<SqlPageExpr<Self::Input>>;
}

/// Rejects database-owned inputs because no returned row is available.
struct StandaloneEnvironment;
/// Projects database-owned inputs into the current returned row.
struct RowEnvironment;

impl ExprEnvironment for StandaloneEnvironment {
    type Input = NoRowInput;

    fn use_database_expr(
        _rewriter: &mut QueryRewriter<'_>,
        expression: SqlExpr,
    ) -> anyhow::Result<StandaloneExpr> {
        if let SqlExpr::Function(function) = &expression
            && let [ObjectNamePart::Identifier(name)] = function.name.0.as_slice()
        {
            return Err(anyhow!(
                "{} is not a supported sqlpage function and cannot be evaluated before the query",
                name.value
            ));
        }
        Err(anyhow!(
            "{expression} is a database expression, but its value is required before the query can run"
        ))
    }
}

impl ExprEnvironment for RowEnvironment {
    type Input = RowInputId;

    fn use_database_expr(
        rewriter: &mut QueryRewriter<'_>,
        expression: SqlExpr,
    ) -> anyhow::Result<RowExpr> {
        let id = rewriter.add_row_input(expression)?;
        Ok(SqlPageExpr::Input(id))
    }
}

#[derive(Clone, Copy)]
/// SQL operations whose semantics are implemented by the shared `SQLPage`
/// expression evaluator.
enum EmulatedFunction {
    Concat,
    Coalesce,
    JsonObject,
    JsonArray,
}

/// Rewrites one parsed statement into database SQL plus the `SQLPage`
/// expressions evaluated around it.
pub(super) fn rewrite_query(
    mut statement: SqlStatement,
    database: &DbInfo,
    semicolon: bool,
) -> anyhow::Result<Query> {
    let source_span = source_span(&statement);
    let mut rewriter = QueryRewriter {
        database,
        bindings: Vec::new(),
        row_input_json: Vec::new(),
        private_projection: Vec::new(),
        error: None,
    };
    if let Some(single_row) = rewrite_single_row(&mut statement, &mut rewriter)? {
        return Ok(Query {
            body: QueryBody::SingleRow(single_row),
            source_span,
        });
    }
    let computed_columns = rewrite_top_level_projection(&mut statement, &mut rewriter)?;

    let _ = statement.visit(&mut rewriter);
    if let Some(error) = rewriter.error {
        return Err(error);
    }

    if let SqlStatement::Query(query) = &mut statement
        && let SetExpr::Select(select) = query.body.as_mut()
        && select.projection.is_empty()
    {
        select.projection.push(SelectItem::ExprWithAlias {
            expr: SqlExpr::value(Value::Null),
            alias: Ident::with_quote('"', format!("{SQLPAGE_INPUT_PREFIX}anchor")),
        });
        rewriter.row_input_json.push(false);
    }

    let json_columns = extract_json_columns(&statement, database.database_type)
        .into_iter()
        .filter(|name| !name.starts_with(SQLPAGE_INPUT_PREFIX))
        .collect();
    let bindings = rewriter.finish_bindings(&mut statement)?;
    let sql = format!(
        "{statement}{semicolon}",
        semicolon = if semicolon { ";" } else { "" }
    );

    Ok(Query {
        body: QueryBody::Database(DatabaseQuery {
            sql,
            bindings,
            row_input_json: rewriter.row_input_json.into_boxed_slice(),
            computed_columns: computed_columns.into_boxed_slice(),
            json_columns,
        }),
        source_span,
    })
}

/// Removes SQLPage-owned projection expressions from the database projection
/// and appends their private database inputs as a trailing suffix.
fn rewrite_top_level_projection(
    statement: &mut SqlStatement,
    rewriter: &mut QueryRewriter<'_>,
) -> anyhow::Result<Vec<OutputColumn<RowExpr>>> {
    let SqlStatement::Query(query) = statement else {
        return Ok(Vec::new());
    };
    let SetExpr::Select(select) = query.body.as_mut() else {
        return Ok(Vec::new());
    };
    if select.distinct.is_some() && select.projection.iter().any(select_item_contains_sqlpage) {
        anyhow::bail!(
            "SQLPage-computed projections cannot be used with SELECT DISTINCT because DISTINCT must be evaluated by the database"
        );
    }

    let (mut database_projection, computed_columns) =
        rewrite_projection_items(std::mem::take(&mut select.projection), rewriter)?;
    database_projection.append(&mut rewriter.private_projection);
    select.projection = database_projection;

    reject_computed_alias_references("WHERE", select.selection.as_ref(), &computed_columns)?;
    reject_computed_group_by_references(&select.group_by, &computed_columns)?;
    reject_computed_alias_references("HAVING", select.having.as_ref(), &computed_columns)?;
    reject_computed_alias_references("QUALIFY", select.qualify.as_ref(), &computed_columns)?;
    reject_computed_aliases_in_expressions("CLUSTER BY", &select.cluster_by, &computed_columns)?;
    reject_computed_aliases_in_expressions(
        "DISTRIBUTE BY",
        &select.distribute_by,
        &computed_columns,
    )?;
    reject_computed_ordering_references("SORT BY", &select.sort_by, &computed_columns)?;
    reject_computed_order_by_references(query.order_by.as_ref(), &computed_columns)?;
    Ok(computed_columns)
}

fn rewrite_projection_items(
    projection: Vec<SelectItem>,
    rewriter: &mut QueryRewriter<'_>,
) -> anyhow::Result<(Vec<SelectItem>, Vec<OutputColumn<RowExpr>>)> {
    let mut database_projection = Vec::with_capacity(projection.len());
    let mut computed_columns = Vec::new();
    for item in projection {
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
        match rewriter.rewrite_projection(expression)? {
            RewrittenProjection::Database(expression) => {
                database_projection.push(match alias {
                    Some(alias) => SelectItem::ExprWithAlias {
                        expr: expression,
                        alias,
                    },
                    None => SelectItem::UnnamedExpr(expression),
                });
            }
            RewrittenProjection::PerRow(value) => {
                computed_columns.push(OutputColumn { name, value });
            }
        }
    }
    Ok((database_projection, computed_columns))
}

fn reject_computed_order_by_references(
    order_by: Option<&sqlparser::ast::OrderBy>,
    computed_columns: &[OutputColumn<RowExpr>],
) -> anyhow::Result<()> {
    if let Some(order_by) = order_by
        && let OrderByKind::Expressions(expressions) = &order_by.kind
    {
        reject_computed_ordering_references("ORDER BY", expressions, computed_columns)?;
    }
    Ok(())
}

fn reject_computed_ordering_references(
    clause: &str,
    expressions: &[sqlparser::ast::OrderByExpr],
    computed_columns: &[OutputColumn<RowExpr>],
) -> anyhow::Result<()> {
    if expressions
        .iter()
        .any(|ordering| references_computed_projection(&ordering.expr, computed_columns, true))
    {
        anyhow::bail!(
            "{clause} cannot reference a SQLPage-computed column because ordering is performed by the database"
        );
    }
    Ok(())
}

fn reject_computed_group_by_references(
    group_by: &GroupByExpr,
    computed_columns: &[OutputColumn<RowExpr>],
) -> anyhow::Result<()> {
    let GroupByExpr::Expressions(expressions, _) = group_by else {
        return Ok(());
    };
    if expressions
        .iter()
        .any(|expression| references_computed_projection(expression, computed_columns, true))
    {
        anyhow::bail!(
            "GROUP BY cannot reference a SQLPage-computed column because grouping is performed by the database"
        );
    }
    Ok(())
}

fn reject_computed_alias_references(
    clause: &str,
    expression: Option<&SqlExpr>,
    computed_columns: &[OutputColumn<RowExpr>],
) -> anyhow::Result<()> {
    if expression.is_some_and(|expression| {
        references_computed_projection(expression, computed_columns, false)
    }) {
        anyhow::bail!(
            "{clause} cannot reference a SQLPage-computed column because it is evaluated by the database"
        );
    }
    Ok(())
}

fn reject_computed_aliases_in_expressions(
    clause: &str,
    expressions: &[SqlExpr],
    computed_columns: &[OutputColumn<RowExpr>],
) -> anyhow::Result<()> {
    if expressions
        .iter()
        .any(|expression| references_computed_projection(expression, computed_columns, false))
    {
        anyhow::bail!(
            "{clause} cannot reference a SQLPage-computed column because it is evaluated by the database"
        );
    }
    Ok(())
}

fn references_computed_projection(
    expression: &SqlExpr,
    computed_columns: &[OutputColumn<RowExpr>],
    reject_ordinal: bool,
) -> bool {
    if computed_columns.is_empty() {
        return false;
    }
    if reject_ordinal
        && matches!(
            expression,
            SqlExpr::Value(ValueWithSpan {
                value: Value::Number(_, _),
                ..
            })
        )
    {
        return true;
    }

    let mut finder = ComputedAliasFinder { computed_columns };
    sqlparser::ast::Visit::visit(expression, &mut finder).is_break()
}

/// Rewrites a guaranteed one-row query directly as standalone expressions,
/// avoiding both a database round trip and an intermediate row-expression tree.
fn rewrite_single_row(
    statement: &mut SqlStatement,
    rewriter: &mut QueryRewriter<'_>,
) -> anyhow::Result<Option<SingleRowQuery>> {
    if !has_single_row_shape(statement) {
        return Ok(None);
    }
    let SqlStatement::Query(query) = statement else {
        return Ok(None);
    };
    let SetExpr::Select(select) = query.body.as_mut() else {
        return Ok(None);
    };
    for item in &select.projection {
        let SelectItem::ExprWithAlias { expr, .. } = item else {
            return Ok(None);
        };
        if !can_build_standalone(expr)? {
            return Ok(None);
        }
    }

    let mut columns = Vec::with_capacity(select.projection.len());
    for item in std::mem::take(&mut select.projection) {
        let SelectItem::ExprWithAlias { expr, alias } = item else {
            unreachable!("projection shape was checked")
        };
        columns.push(OutputColumn {
            name: alias.value,
            value: build_sqlpage_expr::<StandaloneEnvironment>(rewriter, expr)?,
        });
    }
    Ok(Some(SingleRowQuery {
        columns: columns.into_boxed_slice(),
    }))
}

fn has_single_row_shape(statement: &SqlStatement) -> bool {
    let SqlStatement::Query(query) = statement else {
        return false;
    };
    if query.with.is_some()
        || query.order_by.is_some()
        || query.limit_clause.is_some()
        || query.fetch.is_some()
        || !query.locks.is_empty()
        || query.for_clause.is_some()
        || query.settings.is_some()
        || query.format_clause.is_some()
        || !query.pipe_operators.is_empty()
    {
        return false;
    }
    let SetExpr::Select(select) = query.body.as_ref() else {
        return false;
    };
    select.distinct.is_none()
        && select.top.is_none()
        && select.into.is_none()
        && select.from.is_empty()
        && select.lateral_views.is_empty()
        && select.selection.is_none()
        && select.group_by == GroupByExpr::Expressions(vec![], vec![])
        && select.cluster_by.is_empty()
        && select.distribute_by.is_empty()
        && select.sort_by.is_empty()
        && select.having.is_none()
        && select.named_window.is_empty()
        && select.qualify.is_none()
        && select.prewhere.is_none()
        && select.connect_by.is_empty()
        && select.optimizer_hints.is_empty()
        && select.select_modifiers.is_none()
        && select.exclude.is_none()
}

/// Checks standalone support without consuming or cloning the AST, allowing
/// callers to select a rewrite path before moving any nodes.
fn can_build_standalone(expression: &SqlExpr) -> anyhow::Result<bool> {
    match expression {
        SqlExpr::Value(ValueWithSpan {
            value:
                Value::Boolean(_)
                | Value::Number(_, _)
                | Value::SingleQuotedString(_)
                | Value::Null
                | Value::Placeholder(_),
            ..
        }) => Ok(true),
        SqlExpr::Identifier(identifier) => Ok(variable_from_ident(identifier).is_some()),
        SqlExpr::Function(function) => {
            if recognize_sqlpage_function(function)?.is_none()
                && emulated_function(function).is_none()
            {
                return Ok(false);
            }
            let FunctionArguments::List(arguments) = &function.args else {
                return Ok(false);
            };
            for argument in &arguments.args {
                let FunctionArg::Unnamed(FunctionArgExpr::Expr(expression)) = argument else {
                    return Ok(false);
                };
                if !can_build_standalone(expression)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        SqlExpr::BinaryOp {
            left,
            op: BinaryOperator::StringConcat,
            right,
        } => Ok(can_build_standalone(left)? && can_build_standalone(right)?),
        SqlExpr::Nested(expression) => can_build_standalone(expression),
        _ => Ok(false),
    }
}

impl QueryRewriter<'_> {
    /// Splits a projected expression at SQLPage-supported operations while
    /// leaving opaque database operations in the SQL AST.
    fn rewrite_projection(&mut self, expression: SqlExpr) -> anyhow::Result<RewrittenProjection> {
        match expression {
            SqlExpr::Function(function) => {
                if recognize_sqlpage_function(&function)?.is_some() {
                    return build_sqlpage_expr::<RowEnvironment>(self, SqlExpr::Function(function))
                        .map(RewrittenProjection::PerRow);
                }
                if let Some(kind) = emulated_function(&function) {
                    return self.rewrite_emulated_projection(function, kind);
                }
                let mut expression = SqlExpr::Function(function);
                self.rewrite_database_expression(&mut expression)?;
                Ok(RewrittenProjection::Database(expression))
            }
            SqlExpr::BinaryOp {
                left,
                op: BinaryOperator::StringConcat,
                right,
            } => {
                let left = self.rewrite_projection(*left)?;
                let right = self.rewrite_projection(*right)?;
                match (left, right) {
                    (RewrittenProjection::Database(left), RewrittenProjection::Database(right)) => {
                        Ok(RewrittenProjection::Database(SqlExpr::BinaryOp {
                            left: Box::new(left),
                            op: BinaryOperator::StringConcat,
                            right: Box::new(right),
                        }))
                    }
                    (left, right) => Ok(RewrittenProjection::PerRow(SqlPageExpr::Concat {
                        arguments: vec![
                            self.projection_into_row_expr(left)?,
                            self.projection_into_row_expr(right)?,
                        ]
                        .into_boxed_slice(),
                        null_behavior: self.database.database_type.concat_operator_null_behavior(),
                    })),
                }
            }
            SqlExpr::Nested(expression) => match self.rewrite_projection(*expression)? {
                RewrittenProjection::Database(expression) => Ok(RewrittenProjection::Database(
                    SqlExpr::Nested(Box::new(expression)),
                )),
                RewrittenProjection::PerRow(expression) => {
                    Ok(RewrittenProjection::PerRow(expression))
                }
            },
            mut expression => {
                self.rewrite_database_expression(&mut expression)?;
                Ok(RewrittenProjection::Database(expression))
            }
        }
    }

    fn rewrite_emulated_projection(
        &mut self,
        function: Function,
        kind: EmulatedFunction,
    ) -> anyhow::Result<RewrittenProjection> {
        let (arguments, original) = take_expression_arguments(function)?;
        let mut rewritten = Vec::with_capacity(arguments.len());
        let mut has_per_row = false;
        for argument in arguments {
            let argument = self.rewrite_projection(argument)?;
            has_per_row |= matches!(argument, RewrittenProjection::PerRow(_));
            rewritten.push(argument);
        }
        if !has_per_row {
            let arguments = rewritten
                .into_iter()
                .map(|argument| match argument {
                    RewrittenProjection::Database(expression) => expression,
                    RewrittenProjection::PerRow(_) => unreachable!(),
                })
                .collect();
            return Ok(RewrittenProjection::Database(rebuild_function(
                original, arguments,
            )));
        }

        let arguments = rewritten
            .into_iter()
            .map(|argument| self.projection_into_row_expr(argument))
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(RewrittenProjection::PerRow(build_emulated(
            kind,
            arguments,
            self.database.database_type,
        )?))
    }

    /// Converts a database-owned projection fragment into a typed row input,
    /// while preserving an already per-row expression unchanged.
    fn projection_into_row_expr(
        &mut self,
        projection: RewrittenProjection,
    ) -> anyhow::Result<RowExpr> {
        match projection {
            RewrittenProjection::Database(expression) => {
                build_sqlpage_expr::<RowEnvironment>(self, expression)
            }
            RewrittenProjection::PerRow(expression) => Ok(expression),
        }
    }

    fn rewrite_database_expression(&mut self, expression: &mut SqlExpr) -> anyhow::Result<()> {
        let _ = expression.visit(self);
        self.error.take().map_or(Ok(()), Err)
    }

    fn add_binding(&mut self, value: StandaloneExpr) -> SqlExpr {
        let sequence = self.bindings.len();
        self.bindings.push(value);
        let placeholder = match placeholder_style(self.database.kind) {
            PlaceholderStyle::Numbered { prefix } => format!("{prefix}{}", sequence + 1),
            PlaceholderStyle::Positional { .. } => format!("${}", sequence + 1),
        };
        cast_placeholder(placeholder, self.database.database_type)
    }

    fn add_row_input(&mut self, mut expression: SqlExpr) -> anyhow::Result<RowInputId> {
        let decode_as_json = is_json_expression(&expression);
        self.rewrite_database_expression(&mut expression)?;
        let index = self.row_input_json.len();
        let name = format!("{SQLPAGE_INPUT_PREFIX}{index}");
        self.private_projection.push(SelectItem::ExprWithAlias {
            expr: expression,
            alias: Ident::with_quote('"', name),
        });
        self.row_input_json.push(decode_as_json);
        Ok(RowInputId::new(index))
    }

    fn finish_bindings(
        &mut self,
        statement: &mut SqlStatement,
    ) -> anyhow::Result<Box<[StandaloneExpr]>> {
        let PlaceholderStyle::Positional { token } = placeholder_style(self.database.kind) else {
            return Ok(std::mem::take(&mut self.bindings).into_boxed_slice());
        };
        let mut finalizer = PositionalBindingFinalizer {
            token,
            binding_count: self.bindings.len(),
            order: Vec::with_capacity(self.bindings.len()),
            error: None,
        };
        let _ = statement.visit(&mut finalizer);
        if let Some(error) = finalizer.error {
            return Err(error);
        }
        let mut bindings = std::mem::take(&mut self.bindings)
            .into_iter()
            .map(Some)
            .collect::<Vec<_>>();
        finalizer
            .order
            .into_iter()
            .map(|index| {
                bindings[index].take().ok_or_else(|| {
                    anyhow!("Generated binding placeholder ${} was repeated", index + 1)
                })
            })
            .collect::<anyhow::Result<Box<_>>>()
    }
}

struct PositionalBindingFinalizer {
    token: &'static str,
    binding_count: usize,
    order: Vec<usize>,
    error: Option<anyhow::Error>,
}

impl VisitorMut for PositionalBindingFinalizer {
    type Break = ();

    fn pre_visit_expr(&mut self, expression: &mut SqlExpr) -> ControlFlow<Self::Break> {
        let SqlExpr::Value(ValueWithSpan {
            value: Value::Placeholder(name),
            span,
        }) = expression
        else {
            return ControlFlow::Continue(());
        };
        if *span != Span::empty() {
            return ControlFlow::Continue(());
        }
        let Some(index) = name
            .strip_prefix('$')
            .and_then(|number| number.parse::<usize>().ok())
            .and_then(|number| number.checked_sub(1))
        else {
            return ControlFlow::Continue(());
        };
        if index >= self.binding_count {
            self.error = Some(anyhow!("Invalid generated binding placeholder {name}"));
            return ControlFlow::Break(());
        }
        self.order.push(index);
        self.token.clone_into(name);
        ControlFlow::Continue(())
    }
}

impl VisitorMut for QueryRewriter<'_> {
    type Break = ();

    fn pre_visit_expr(&mut self, expression: &mut SqlExpr) -> ControlFlow<Self::Break> {
        if self.error.is_some() {
            return ControlFlow::Break(());
        }

        let replacement = match expression {
            SqlExpr::Value(ValueWithSpan {
                value: Value::Placeholder(_),
                span,
            }) if *span == Span::empty() => None,
            SqlExpr::Value(ValueWithSpan {
                value: Value::Placeholder(_),
                ..
            })
            | SqlExpr::Identifier(_) => variable_from_expr(expression)
                .map(|variable| self.add_binding(SqlPageExpr::Variable(variable))),
            SqlExpr::Function(function) => match recognize_sqlpage_function(function) {
                Ok(Some(_)) => {
                    let owned = std::mem::replace(expression, SqlExpr::value(Value::Null));
                    match build_sqlpage_expr::<StandaloneEnvironment>(self, owned) {
                        Ok(value) => Some(self.add_binding(value)),
                        Err(error) => {
                            self.error = Some(error.context(
                                "A SQLPage function used by the database could not be evaluated before the query",
                            ));
                            None
                        }
                    }
                }
                Ok(None) => None,
                Err(error) => {
                    self.error = Some(error);
                    None
                }
            },
            SqlExpr::BinaryOp {
                left,
                op: BinaryOperator::StringConcat,
                right,
            } if self.database.database_type == SupportedDatabase::Mssql => {
                let left = std::mem::replace(left.as_mut(), SqlExpr::value(Value::Null));
                let right = std::mem::replace(right.as_mut(), SqlExpr::value(Value::Null));
                Some(make_function("CONCAT", vec![left, right]))
            }
            SqlExpr::Cast {
                kind: kind @ CastKind::DoubleColon,
                ..
            } if ![
                SupportedDatabase::Postgres,
                SupportedDatabase::Duckdb,
                SupportedDatabase::Snowflake,
                SupportedDatabase::Generic,
            ]
            .contains(&self.database.database_type) =>
            {
                *kind = CastKind::Cast;
                None
            }
            _ => None,
        };

        if let Some(replacement) = replacement {
            *expression = replacement;
        }
        ControlFlow::Continue(())
    }
}

/// Consumes an AST expression into the shared `SQLPage` expression type. The
/// environment determines whether opaque database fragments are illegal or
/// become private row inputs.
fn build_sqlpage_expr<Environment: ExprEnvironment>(
    rewriter: &mut QueryRewriter<'_>,
    expression: SqlExpr,
) -> anyhow::Result<SqlPageExpr<Environment::Input>> {
    match expression {
        SqlExpr::Value(ValueWithSpan { value, .. }) => match value {
            Value::Placeholder(name) => Ok(SqlPageExpr::Variable(variable_from_placeholder(name))),
            Value::SingleQuotedString(text) => Ok(SqlPageExpr::Literal(JsonValue::String(text))),
            Value::Number(number, _) => Ok(SqlPageExpr::Literal(JsonValue::Number(
                number.parse().context("Invalid numeric SQL literal")?,
            ))),
            Value::Boolean(value) => Ok(SqlPageExpr::Literal(JsonValue::Bool(value))),
            Value::Null => Ok(SqlPageExpr::Literal(JsonValue::Null)),
            _ => {
                Environment::use_database_expr(rewriter, SqlExpr::Value(ValueWithSpan::from(value)))
            }
        },
        SqlExpr::Identifier(identifier) => {
            if let Some(variable) = variable_from_ident(&identifier) {
                Ok(SqlPageExpr::Variable(variable))
            } else {
                Environment::use_database_expr(rewriter, SqlExpr::Identifier(identifier))
            }
        }
        SqlExpr::Function(function) => {
            if let Some(function_name) = recognize_sqlpage_function(&function)? {
                let (arguments, _) = take_expression_arguments(function)?;
                let arguments = arguments
                    .into_iter()
                    .map(|argument| build_sqlpage_expr::<Environment>(rewriter, argument))
                    .collect::<anyhow::Result<Vec<_>>>()?;
                Ok(SqlPageExpr::Call {
                    function: function_name,
                    arguments: arguments.into_boxed_slice(),
                })
            } else if let Some(kind) = emulated_function(&function) {
                let (arguments, _) = take_expression_arguments(function)?;
                let arguments = arguments
                    .into_iter()
                    .map(|argument| build_sqlpage_expr::<Environment>(rewriter, argument))
                    .collect::<anyhow::Result<Vec<_>>>()?;
                build_emulated(kind, arguments, rewriter.database.database_type)
            } else {
                Environment::use_database_expr(rewriter, SqlExpr::Function(function))
            }
        }
        SqlExpr::BinaryOp {
            left,
            op: BinaryOperator::StringConcat,
            right,
        } => Ok(SqlPageExpr::Concat {
            arguments: vec![
                build_sqlpage_expr::<Environment>(rewriter, *left)?,
                build_sqlpage_expr::<Environment>(rewriter, *right)?,
            ]
            .into_boxed_slice(),
            null_behavior: rewriter
                .database
                .database_type
                .concat_operator_null_behavior(),
        }),
        SqlExpr::Nested(expression) => build_sqlpage_expr::<Environment>(rewriter, *expression),
        expression => Environment::use_database_expr(rewriter, expression),
    }
}

fn build_emulated<Input>(
    kind: EmulatedFunction,
    arguments: Vec<SqlPageExpr<Input>>,
    database: SupportedDatabase,
) -> anyhow::Result<SqlPageExpr<Input>> {
    Ok(match kind {
        EmulatedFunction::Concat => SqlPageExpr::Concat {
            arguments: arguments.into_boxed_slice(),
            null_behavior: database.concat_function_null_behavior(),
        },
        EmulatedFunction::Coalesce => SqlPageExpr::Coalesce(arguments.into_boxed_slice()),
        EmulatedFunction::JsonArray => SqlPageExpr::JsonArray(arguments.into_boxed_slice()),
        EmulatedFunction::JsonObject => {
            if !arguments.len().is_multiple_of(2) {
                anyhow::bail!("JSON_OBJECT requires an even number of arguments");
            }
            let mut arguments = arguments.into_iter();
            let mut entries = Vec::with_capacity(arguments.len() / 2);
            while let Some(key) = arguments.next() {
                let value = arguments.next().expect("argument count was checked");
                entries.push((key, value));
            }
            SqlPageExpr::JsonObject(entries.into_boxed_slice())
        }
    })
}

/// Recognizes and validates an unquoted `sqlpage.<name>` call. A recognized
/// call is either rewritten or rejected and can never reach database SQL.
fn recognize_sqlpage_function(function: &Function) -> anyhow::Result<Option<SqlPageFunctionName>> {
    let ObjectName(parts) = &function.name;
    if !is_sqlpage_func(parts) {
        return Ok(None);
    }
    let [
        ObjectNamePart::Identifier(_),
        ObjectNamePart::Identifier(name),
    ] = parts.as_slice()
    else {
        unreachable!("is_sqlpage_func checked the name")
    };
    if function.uses_odbc_syntax
        || !matches!(function.parameters, FunctionArguments::None)
        || function.filter.is_some()
        || function.null_treatment.is_some()
        || function.over.is_some()
        || !function.within_group.is_empty()
    {
        anyhow::bail!(
            "Modifiers are not supported on SQLPage function {}",
            function.name
        );
    }
    let FunctionArguments::List(FunctionArgumentList {
        duplicate_treatment: None,
        clauses,
        ..
    }) = &function.args
    else {
        anyhow::bail!(
            "Unsupported argument syntax for SQLPage function {}",
            function.name
        );
    };
    if !clauses.is_empty() {
        anyhow::bail!(
            "Argument clauses are not supported on SQLPage function {}",
            function.name
        );
    }
    Ok(Some(SqlPageFunctionName::from_str(&name.value)?))
}

fn emulated_function(function: &Function) -> Option<EmulatedFunction> {
    let [ObjectNamePart::Identifier(name)] = function.name.0.as_slice() else {
        return None;
    };
    if !matches!(function.parameters, FunctionArguments::None)
        || function.filter.is_some()
        || function.null_treatment.is_some()
        || function.over.is_some()
        || !function.within_group.is_empty()
    {
        return None;
    }
    match name.value.to_ascii_lowercase().as_str() {
        "concat" => Some(EmulatedFunction::Concat),
        "coalesce" => Some(EmulatedFunction::Coalesce),
        "json_object" | "jsonb_object" | "json_build_object" | "jsonb_build_object" => {
            Some(EmulatedFunction::JsonObject)
        }
        "json_array" | "jsonb_array" | "json_build_array" | "jsonb_build_array" => {
            Some(EmulatedFunction::JsonArray)
        }
        _ => None,
    }
}

/// Moves expression arguments out of a function while retaining its emptied
/// AST shell so database-owned functions can be rebuilt without cloning.
fn take_expression_arguments(mut function: Function) -> anyhow::Result<(Vec<SqlExpr>, Function)> {
    let FunctionArguments::List(arguments) = &mut function.args else {
        anyhow::bail!("Unsupported arguments to {}", function.name);
    };
    if arguments.duplicate_treatment.is_some() || !arguments.clauses.is_empty() {
        anyhow::bail!("Unsupported arguments to {}", function.name);
    }
    let arguments = std::mem::take(&mut arguments.args)
        .into_iter()
        .map(|argument| match argument {
            FunctionArg::Unnamed(FunctionArgExpr::Expr(expression)) => Ok(expression),
            _ => Err(anyhow!(
                "Named and wildcard function arguments are not supported"
            )),
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok((arguments, function))
}

fn rebuild_function(mut function: Function, expressions: Vec<SqlExpr>) -> SqlExpr {
    let FunctionArguments::List(arguments) = &mut function.args else {
        unreachable!()
    };
    arguments.args = expressions
        .into_iter()
        .map(|expression| FunctionArg::Unnamed(FunctionArgExpr::Expr(expression)))
        .collect();
    SqlExpr::Function(function)
}

fn make_function(name: &str, expressions: Vec<SqlExpr>) -> SqlExpr {
    SqlExpr::Function(Function {
        name: ObjectName(vec![ObjectNamePart::Identifier(Ident::new(name))]),
        args: FunctionArguments::List(FunctionArgumentList {
            args: expressions
                .into_iter()
                .map(|expression| FunctionArg::Unnamed(FunctionArgExpr::Expr(expression)))
                .collect(),
            duplicate_treatment: None,
            clauses: Vec::new(),
        }),
        parameters: FunctionArguments::None,
        over: None,
        filter: None,
        null_treatment: None,
        within_group: Vec::new(),
        uses_odbc_syntax: false,
    })
}

fn variable_from_expr(expression: &SqlExpr) -> Option<VariableRef> {
    match expression {
        SqlExpr::Value(ValueWithSpan {
            value: Value::Placeholder(name),
            ..
        }) => Some(variable_from_placeholder(name.clone())),
        SqlExpr::Identifier(identifier) => variable_from_ident(identifier),
        _ => None,
    }
}

fn variable_from_ident(identifier: &Ident) -> Option<VariableRef> {
    if identifier.quote_style.is_some() {
        return None;
    }
    let prefix = identifier.value.chars().next()?;
    matches!(prefix, '$' | ':' | '?').then(|| VariableRef {
        name: identifier.value[prefix.len_utf8()..].to_owned(),
        source: variable_source(prefix),
    })
}

fn variable_from_placeholder(mut name: String) -> VariableRef {
    let prefix = name.remove(0);
    VariableRef {
        name,
        source: variable_source(prefix),
    }
}

fn variable_source(prefix: char) -> VariableSource {
    match prefix {
        '$' => VariableSource::SetOrUrl,
        ':' => VariableSource::SetOrForm,
        _ => VariableSource::Url,
    }
}

/// Wraps a generated placeholder in the backend-specific text cast when the
/// database cannot reliably infer that the parameter is a string.
///
/// `SQLPage` always binds parameters as strings. `PostgreSQL` (which pins the
/// parameter type to `TEXT` when preparing the statement), `MySQL` and `SQL
/// Server` (which convert the bound string to the type expected by the
/// surrounding expression) do not need the cast, and it can even be harmful:
/// on `SQL Server` the parameter is bound as `NVARCHAR(MAX)`, and casting it
/// to a narrow `VARCHAR` mangles non-ASCII values. `SQLite` and ODBC-backed
/// databases, whose parameter type inference is unpredictable, keep the
/// explicit cast.
fn cast_placeholder(placeholder: String, database: SupportedDatabase) -> SqlExpr {
    let data_type = match database {
        SupportedDatabase::Sqlite => DataType::Text,
        SupportedDatabase::Oracle => DataType::Varchar(Some(CharacterLength::IntegerLength {
            length: 4000,
            unit: None,
        })),
        SupportedDatabase::Postgres | SupportedDatabase::MySql | SupportedDatabase::Mssql => {
            return SqlExpr::value(Value::Placeholder(placeholder));
        }
        _ => DataType::Varchar(None),
    };
    SqlExpr::Cast {
        expr: Box::new(SqlExpr::value(Value::Placeholder(placeholder))),
        data_type,
        format: None,
        kind: CastKind::Cast,
        array: false,
    }
}

fn source_span(value: &impl sqlparser::ast::Spanned) -> SourceSpan {
    let span = value.span();
    SourceSpan {
        start: SourceLocation {
            line: usize::try_from(span.start.line).unwrap_or(0),
            column: usize::try_from(span.start.column).unwrap_or(0),
        },
        end: SourceLocation {
            line: usize::try_from(span.end.line).unwrap_or(0),
            column: usize::try_from(span.end.column).unwrap_or(0),
        },
    }
}

fn select_item_contains_sqlpage(item: &SelectItem) -> bool {
    struct Finder(bool);
    impl sqlparser::ast::Visitor for Finder {
        type Break = ();

        fn pre_visit_expr(&mut self, expression: &SqlExpr) -> ControlFlow<Self::Break> {
            if let SqlExpr::Function(function) = expression
                && is_sqlpage_func(&function.name.0)
            {
                self.0 = true;
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        }
    }
    let mut finder = Finder(false);
    let _ = sqlparser::ast::Visit::visit(item, &mut finder);
    finder.0
}
