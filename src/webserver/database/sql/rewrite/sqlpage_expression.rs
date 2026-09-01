//! Recognition and construction of expressions evaluated by `SQLPage`.

use std::str::FromStr as _;

use anyhow::{Context as _, anyhow};
use serde_json::Value as JsonValue;
use sqlparser::ast::{
    BinaryOperator, Expr as SqlExpr, Function, FunctionArg, FunctionArgExpr, FunctionArgumentList,
    FunctionArguments, Ident, ObjectName, ObjectNamePart, Value, ValueWithSpan,
};

use crate::webserver::database::sqlpage_expr::{
    NoRowInput, SqlPageExpr, StandaloneExpr, VariableRef, VariableSource,
};
use crate::webserver::database::sqlpage_functions::functions::SqlPageFunctionName;
use crate::webserver::database::{DbInfo, SupportedDatabase};

const SQLPAGE_FUNCTION_NAMESPACE: &str = "sqlpage";

/// Defines how an opaque database expression is represented when it crosses
/// into a SQLPage-owned expression.
pub(super) trait SqlPageExpressionContext {
    type Input;

    fn use_database_expr(
        &mut self,
        expression: SqlExpr,
    ) -> anyhow::Result<SqlPageExpr<Self::Input>>;
}

/// Rejects database inputs because no returned row is available.
pub(super) struct StandaloneContext;

impl SqlPageExpressionContext for StandaloneContext {
    type Input = NoRowInput;

    fn use_database_expr(&mut self, expression: SqlExpr) -> anyhow::Result<StandaloneExpr> {
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

#[derive(Clone, Copy)]
pub(super) enum EmulatedFunction {
    Concat,
    Coalesce,
    JsonObject,
    JsonArray,
}

/// Checks whether `SQLPage` can evaluate a selected value without a database row.
pub(super) fn is_static_simple_select_expression(expression: &SqlExpr) -> anyhow::Result<bool> {
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
                if !is_static_simple_select_expression(expression)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        SqlExpr::BinaryOp {
            left,
            op: BinaryOperator::StringConcat,
            right,
        } => {
            Ok(is_static_simple_select_expression(left)?
                && is_static_simple_select_expression(right)?)
        }
        SqlExpr::Nested(expression) => is_static_simple_select_expression(expression),
        _ => Ok(false),
    }
}

pub(super) fn build_sqlpage_expr<Context: SqlPageExpressionContext>(
    database: &DbInfo,
    context: &mut Context,
    expression: SqlExpr,
) -> anyhow::Result<SqlPageExpr<Context::Input>> {
    match expression {
        SqlExpr::Value(ValueWithSpan { value, .. }) => match value {
            Value::Placeholder(name) => Ok(SqlPageExpr::Variable(variable_from_placeholder(name))),
            Value::SingleQuotedString(text) => Ok(SqlPageExpr::Literal(JsonValue::String(text))),
            Value::Number(number, _) => Ok(SqlPageExpr::Literal(JsonValue::Number(
                number.parse().context("Invalid numeric SQL literal")?,
            ))),
            Value::Boolean(value) => Ok(SqlPageExpr::Literal(JsonValue::Bool(value))),
            Value::Null => Ok(SqlPageExpr::Literal(JsonValue::Null)),
            _ => context.use_database_expr(SqlExpr::Value(ValueWithSpan::from(value))),
        },
        SqlExpr::Identifier(identifier) => variable_from_ident(&identifier)
            .map(SqlPageExpr::Variable)
            .map_or_else(
                || context.use_database_expr(SqlExpr::Identifier(identifier)),
                Ok,
            ),
        SqlExpr::Function(function) => {
            if let Some(function_name) = recognize_sqlpage_function(&function)? {
                let arguments = take_expression_arguments(function)?
                    .into_iter()
                    .map(|argument| build_sqlpage_expr(database, context, argument))
                    .collect::<anyhow::Result<Vec<_>>>()?;
                Ok(SqlPageExpr::Call {
                    function: function_name,
                    arguments: arguments.into_boxed_slice(),
                })
            } else if let Some(kind) = emulated_function(&function) {
                let arguments = take_expression_arguments(function)?
                    .into_iter()
                    .map(|argument| build_sqlpage_expr(database, context, argument))
                    .collect::<anyhow::Result<Vec<_>>>()?;
                build_emulated(kind, arguments, database.database_type)
            } else {
                context.use_database_expr(SqlExpr::Function(function))
            }
        }
        SqlExpr::BinaryOp {
            left,
            op: BinaryOperator::StringConcat,
            right,
        } => Ok(SqlPageExpr::Concat {
            arguments: vec![
                build_sqlpage_expr(database, context, *left)?,
                build_sqlpage_expr(database, context, *right)?,
            ]
            .into_boxed_slice(),
            null_behavior: database.database_type.concat_operator_null_behavior(),
        }),
        SqlExpr::Nested(expression) => build_sqlpage_expr(database, context, *expression),
        expression => context.use_database_expr(expression),
    }
}

pub(super) fn build_emulated<Input>(
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
                entries.push((key, arguments.next().expect("argument count was checked")));
            }
            SqlPageExpr::JsonObject(entries.into_boxed_slice())
        }
    })
}

/// Recognizes and validates an unquoted `sqlpage.<name>` call.
pub(super) fn recognize_sqlpage_function(
    function: &Function,
) -> anyhow::Result<Option<SqlPageFunctionName>> {
    let ObjectName(parts) = &function.name;
    if !is_sqlpage_func(parts) {
        return Ok(None);
    }
    let ObjectNamePart::Identifier(name) = &parts[1] else {
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

pub(super) fn is_sqlpage_func(parts: &[ObjectNamePart]) -> bool {
    let [
        ObjectNamePart::Identifier(namespace),
        ObjectNamePart::Identifier(name),
    ] = parts
    else {
        return false;
    };
    namespace.quote_style.is_none()
        && name.quote_style.is_none()
        && namespace
            .value
            .eq_ignore_ascii_case(SQLPAGE_FUNCTION_NAMESPACE)
}

pub(super) fn emulated_function(function: &Function) -> Option<EmulatedFunction> {
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

pub(super) fn take_expression_arguments(function: Function) -> anyhow::Result<Vec<SqlExpr>> {
    let FunctionArguments::List(arguments) = function.args else {
        anyhow::bail!("Unsupported arguments to {}", function.name);
    };
    if arguments.duplicate_treatment.is_some() || !arguments.clauses.is_empty() {
        anyhow::bail!("Unsupported arguments to {}", function.name);
    }
    arguments
        .args
        .into_iter()
        .map(|argument| match argument {
            FunctionArg::Unnamed(FunctionArgExpr::Expr(expression)) => Ok(expression),
            _ => Err(anyhow!(
                "Named and wildcard function arguments are not supported"
            )),
        })
        .collect()
}

pub(super) fn variable_from_expr(expression: &SqlExpr) -> Option<VariableRef> {
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
    let source = variable_source(name.remove(0));
    VariableRef { name, source }
}

fn variable_source(prefix: char) -> VariableSource {
    match prefix {
        '$' => VariableSource::SetOrUrl,
        ':' => VariableSource::SetOrForm,
        _ => VariableSource::Url,
    }
}
