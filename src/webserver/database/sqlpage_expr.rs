//! Expressions evaluated by `SQLPage` instead of the database.
//!
//! The input type records whether an expression may read values from a
//! returned database row. The expression implementation and evaluator remain
//! shared between standalone and per-row expressions.

use std::borrow::Cow;

use anyhow::Context as _;
use serde_json::Value;

use super::execute_queries::DbConn;
use super::sqlpage_functions::functions::SqlPageFunctionName;
use crate::webserver::http_request_info::ExecutionContext;

/// An expression evaluated by `SQLPage`.
///
/// Nodes are not shared automatically because function calls may be
/// effectful. `Input` identifies values supplied by the evaluation site.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SqlPageExpr<Input> {
    Literal(Value),
    Variable(VariableRef),
    Input(Input),
    Call {
        function: SqlPageFunctionName,
        arguments: Box<[Self]>,
    },
    Concat {
        arguments: Box<[Self]>,
        null_behavior: ConcatNullBehavior,
    },
    Coalesce(Box<[Self]>),
    JsonObject(Box<[(Self, Self)]>),
    JsonArray(Box<[Self]>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConcatNullBehavior {
    IgnoreNull,
    PropagateNull,
}

/// Uninhabited input type for expressions that cannot read a database row.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum NoRowInput {}

/// Identifies one private value projected by the database for `SQLPage`.
///
/// The index is private and this type is intentionally not `Clone` or `Copy`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RowInputId(usize);

impl RowInputId {
    pub(super) fn new(index: usize) -> Self {
        Self(index)
    }

    pub(super) fn index(&self) -> usize {
        self.0
    }
}

/// An expression that can be evaluated without a returned database row.
pub(crate) type StandaloneExpr = SqlPageExpr<NoRowInput>;

/// An expression evaluated once for each returned database row.
pub(crate) type RowExpr = SqlPageExpr<RowInputId>;

#[derive(Debug, PartialEq, Eq)]
/// A reference to request or previously assigned `SQLPage` state.
pub(crate) struct VariableRef {
    pub name: String,
    pub source: VariableSource,
}

#[derive(Debug, PartialEq, Eq)]
/// Lookup precedence implied by `SQLPage`'s three variable syntaxes.
pub(crate) enum VariableSource {
    /// Read only from URL parameters (`?name`).
    Url,
    /// Prefer a `SET` value, then read a URL parameter (`$name`).
    SetOrUrl,
    /// Prefer a `SET` value, then read a form field (`:name`).
    SetOrForm,
}

/// A possibly borrowed value produced by `SQLPage` expression evaluation.
pub(crate) enum SqlPageValue<'a> {
    Null,
    Text(Cow<'a, str>),
    /// A number, boolean, array, or object.
    Json(Cow<'a, Value>),
}

impl<'a> SqlPageValue<'a> {
    pub(crate) fn into_function_argument(self) -> Option<Cow<'a, str>> {
        match self {
            Self::Null => None,
            Self::Text(text) => Some(text),
            Self::Json(value) => Some(Cow::Owned(value.to_string())),
        }
    }

    pub(crate) fn into_json(self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Text(Cow::Borrowed(text)) => Value::String(text.to_owned()),
            Self::Text(Cow::Owned(text)) => Value::String(text),
            Self::Json(Cow::Borrowed(value)) => value.clone(),
            Self::Json(Cow::Owned(value)) => value,
        }
    }
}

/// Supplies external inputs to a `SQLPage` expression.
pub(crate) trait ExprInputs<Input> {
    fn take(&mut self, input: &Input) -> anyhow::Result<SqlPageValue<'static>>;
}

/// Input provider for standalone expressions.
pub(crate) struct NoInputs;

impl ExprInputs<NoRowInput> for NoInputs {
    fn take(&mut self, input: &NoRowInput) -> anyhow::Result<SqlPageValue<'static>> {
        match *input {}
    }
}

/// Private values decoded from one returned database row.
pub(crate) struct RowInputs(Vec<Option<Value>>);

impl RowInputs {
    pub(crate) fn new(values: Vec<Value>) -> Self {
        Self(values.into_iter().map(Some).collect())
    }
}

impl ExprInputs<RowInputId> for RowInputs {
    fn take(&mut self, input: &RowInputId) -> anyhow::Result<SqlPageValue<'static>> {
        let value = self
            .0
            .get_mut(input.index())
            .and_then(Option::take)
            .with_context(|| {
                format!(
                    "Row input {} is missing or was already consumed",
                    input.index()
                )
            })?;
        Ok(value_to_sqlpage_value(value))
    }
}

fn value_to_sqlpage_value(value: Value) -> SqlPageValue<'static> {
    match value {
        Value::Null => SqlPageValue::Null,
        Value::String(text) => SqlPageValue::Text(Cow::Owned(text)),
        value => SqlPageValue::Json(Cow::Owned(value)),
    }
}

impl VariableRef {
    fn evaluate<'a>(&self, request: &'a ExecutionContext) -> SqlPageValue<'a> {
        use crate::webserver::request_variables::{LookupPolicy, VariableAccess, VariableValue};
        let policy = match self.source {
            VariableSource::Url => LookupPolicy::GetOnly,
            VariableSource::SetOrForm => LookupPolicy::SetThenPost,
            VariableSource::SetOrUrl => LookupPolicy::SetThenGet,
        };
        let variables = VariableAccess::new(
            &request.url_params,
            &request.post_variables,
            &request.set_variables,
        );
        match variables.lookup(&self.name, policy) {
            VariableValue::Missing | VariableValue::Null => SqlPageValue::Null,
            VariableValue::Text(value) => SqlPageValue::Text(value),
        }
    }
}

impl<Input> SqlPageExpr<Input> {
    /// Evaluates this expression from left to right.
    pub(crate) async fn evaluate<'a>(
        &'a self,
        request: &'a ExecutionContext,
        db_connection: &mut DbConn,
        inputs: &mut impl ExprInputs<Input>,
    ) -> anyhow::Result<SqlPageValue<'a>> {
        match self {
            Self::Literal(value) => Ok(match value {
                Value::Null => SqlPageValue::Null,
                Value::String(text) => SqlPageValue::Text(Cow::Borrowed(text)),
                value => SqlPageValue::Json(Cow::Borrowed(value)),
            }),
            Self::Variable(variable) => Ok(variable.evaluate(request)),
            Self::Input(input) => inputs.take(input).map(SqlPageValue::into_lifetime),
            Self::Call {
                function,
                arguments,
            } => {
                let mut values = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    values.push(
                        Box::pin(argument.evaluate(request, db_connection, inputs))
                            .await?
                            .into_function_argument(),
                    );
                }
                let result = function
                    .evaluate(request, db_connection, values)
                    .await
                    .with_context(|| format!("Error in function call {function}"))?;
                Ok(result.map_or(SqlPageValue::Null, SqlPageValue::Text))
            }
            Self::Concat {
                arguments,
                null_behavior,
            } => {
                let mut result = String::new();
                for argument in arguments {
                    let value = Box::pin(argument.evaluate(request, db_connection, inputs)).await?;
                    match value.into_function_argument() {
                        Some(value) => result.push_str(&value),
                        None if *null_behavior == ConcatNullBehavior::PropagateNull => {
                            return Ok(SqlPageValue::Null);
                        }
                        None => {}
                    }
                }
                Ok(SqlPageValue::Text(Cow::Owned(result)))
            }
            Self::Coalesce(arguments) => {
                for argument in arguments {
                    let value = Box::pin(argument.evaluate(request, db_connection, inputs)).await?;
                    if !matches!(value, SqlPageValue::Null) {
                        return Ok(value);
                    }
                }
                Ok(SqlPageValue::Null)
            }
            Self::JsonObject(entries) => {
                let mut object = serde_json::Map::with_capacity(entries.len());
                for (key, value) in entries {
                    let key = Box::pin(key.evaluate(request, db_connection, inputs))
                        .await?
                        .into_function_argument()
                        .context("JSON object keys cannot be NULL")?
                        .into_owned();
                    let value = Box::pin(value.evaluate(request, db_connection, inputs))
                        .await?
                        .into_json();
                    object.insert(key, value);
                }
                Ok(SqlPageValue::Json(Cow::Owned(Value::Object(object))))
            }
            Self::JsonArray(elements) => {
                let mut array = Vec::with_capacity(elements.len());
                for element in elements {
                    array.push(
                        Box::pin(element.evaluate(request, db_connection, inputs))
                            .await?
                            .into_json(),
                    );
                }
                Ok(SqlPageValue::Json(Cow::Owned(Value::Array(array))))
            }
        }
    }

    pub(crate) fn contains_function(&self, expected: SqlPageFunctionName) -> bool {
        match self {
            Self::Call {
                function,
                arguments,
            } => {
                *function == expected
                    || arguments
                        .iter()
                        .any(|argument| argument.contains_function(expected))
            }
            Self::Concat { arguments, .. }
            | Self::Coalesce(arguments)
            | Self::JsonArray(arguments) => arguments
                .iter()
                .any(|argument| argument.contains_function(expected)),
            Self::JsonObject(entries) => entries.iter().any(|(key, value)| {
                key.contains_function(expected) || value.contains_function(expected)
            }),
            Self::Literal(_) | Self::Variable(_) | Self::Input(_) => false,
        }
    }
}

impl SqlPageValue<'static> {
    fn into_lifetime<'a>(self) -> SqlPageValue<'a> {
        match self {
            Self::Null => SqlPageValue::Null,
            Self::Text(text) => SqlPageValue::Text(text),
            Self::Json(value) => SqlPageValue::Json(value),
        }
    }
}
