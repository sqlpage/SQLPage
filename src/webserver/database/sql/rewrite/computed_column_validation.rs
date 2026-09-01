//! Rejects database clauses that depend on columns computed later by `SQLPage`.

use std::ops::ControlFlow;

use sqlparser::ast::{
    Expr as SqlExpr, GroupByExpr, OrderByKind, Query, Select, SelectItem, SetExpr, Value,
    ValueWithSpan,
};

use super::super::statement::OutputColumn;
use super::sqlpage_expression::is_sqlpage_func;
use crate::webserver::database::sqlpage_expr::RowExpr;

pub(super) fn reject_distinct_sqlpage_projection(select: &Select) -> anyhow::Result<()> {
    if select.distinct.is_some() && select.projection.iter().any(select_item_contains_sqlpage) {
        anyhow::bail!(
            "SQLPage-computed projections cannot be used with SELECT DISTINCT because DISTINCT must be evaluated by the database"
        );
    }
    Ok(())
}

pub(super) fn reject_references(
    query: &Query,
    computed_columns: &[OutputColumn<RowExpr>],
) -> anyhow::Result<()> {
    let SetExpr::Select(select) = query.body.as_ref() else {
        return Ok(());
    };
    let validator = ComputedColumnValidator(computed_columns);
    validator.reject("WHERE", select.selection.iter(), false)?;
    if let GroupByExpr::Expressions(expressions, _) = &select.group_by {
        validator.reject("GROUP BY", expressions, true)?;
    }
    for (clause, expression) in [
        ("HAVING", select.having.as_ref()),
        ("QUALIFY", select.qualify.as_ref()),
    ] {
        validator.reject(clause, expression, false)?;
    }
    for (clause, expressions) in [
        ("CLUSTER BY", select.cluster_by.as_slice()),
        ("DISTRIBUTE BY", select.distribute_by.as_slice()),
    ] {
        validator.reject(clause, expressions, false)?;
    }
    validator.reject(
        "SORT BY",
        select.sort_by.iter().map(|ordering| &ordering.expr),
        true,
    )?;
    if let Some(order_by) = &query.order_by
        && let OrderByKind::Expressions(expressions) = &order_by.kind
    {
        validator.reject(
            "ORDER BY",
            expressions.iter().map(|ordering| &ordering.expr),
            true,
        )?;
    }
    Ok(())
}

struct ComputedColumnValidator<'a>(&'a [OutputColumn<RowExpr>]);

impl ComputedColumnValidator<'_> {
    fn reject<'a>(
        &self,
        clause: &str,
        expressions: impl IntoIterator<Item = &'a SqlExpr>,
        reject_ordinal: bool,
    ) -> anyhow::Result<()> {
        if self.0.is_empty() {
            return Ok(());
        }
        let references_computed_column = |expression: &SqlExpr| {
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
            let mut finder = ComputedAliasFinder(self.0);
            sqlparser::ast::Visit::visit(expression, &mut finder).is_break()
        };
        if expressions.into_iter().any(references_computed_column) {
            let reason = match clause {
                "GROUP BY" => "grouping is performed by the database",
                "SORT BY" | "ORDER BY" => "ordering is performed by the database",
                _ => "it is evaluated by the database",
            };
            anyhow::bail!("{clause} cannot reference a SQLPage-computed column because {reason}");
        }
        Ok(())
    }
}

struct ComputedAliasFinder<'a>(&'a [OutputColumn<RowExpr>]);

impl sqlparser::ast::Visitor for ComputedAliasFinder<'_> {
    type Break = ();

    fn pre_visit_expr(&mut self, expression: &SqlExpr) -> ControlFlow<Self::Break> {
        if let SqlExpr::Identifier(identifier) = expression
            && self
                .0
                .iter()
                .any(|column| identifier.value.eq_ignore_ascii_case(&column.name))
        {
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    }
}

fn select_item_contains_sqlpage(item: &SelectItem) -> bool {
    struct SqlPageFunctionFinder;
    impl sqlparser::ast::Visitor for SqlPageFunctionFinder {
        type Break = ();

        fn pre_visit_expr(&mut self, expression: &SqlExpr) -> ControlFlow<Self::Break> {
            if let SqlExpr::Function(function) = expression
                && is_sqlpage_func(&function.name.0)
            {
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        }
    }
    sqlparser::ast::Visit::visit(item, &mut SqlPageFunctionFinder).is_break()
}
