//! Plans static simple selects without a database query.

use sqlparser::ast::{GroupByExpr, SelectItem, SetExpr, Statement as SqlStatement};

use super::super::statement::{OutputColumn, StaticSimpleSelect};
use super::sqlpage_expression::{
    StandaloneContext, build_sqlpage_expr, is_static_simple_select_expression,
};
use crate::webserver::database::DbInfo;

/// Plans a documented static simple select for execution without the database.
pub(super) fn try_plan(
    statement: &mut SqlStatement,
    database: &DbInfo,
) -> anyhow::Result<Option<StaticSimpleSelect>> {
    if !has_static_simple_select_shape(statement) {
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
        if !is_static_simple_select_expression(expr)? {
            return Ok(None);
        }
    }

    let columns = std::mem::take(&mut select.projection)
        .into_iter()
        .map(|item| {
            let SelectItem::ExprWithAlias { expr, alias } = item else {
                unreachable!("projection shape was checked")
            };
            Ok(OutputColumn {
                name: alias.value,
                value: build_sqlpage_expr(database, &mut StandaloneContext, expr)?,
            })
        })
        .collect::<anyhow::Result<Box<_>>>()?;
    Ok(Some(StaticSimpleSelect { columns }))
}

fn has_static_simple_select_shape(statement: &SqlStatement) -> bool {
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
