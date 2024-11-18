/// This module contains the syntax tree for sqlpage statement parameters.
/// In a query like `SELECT sqlpage.some_function($my_param)`,
/// The stored database statement will be just `SELECT $1`,
/// and the `StmtParam` will contain a the following tree:
///
/// ```text
/// StmtParam::FunctionCall(
///    SqlPageFunctionCall {
///       function: SqlPageFunctionName::some_function,
///      arguments: vec![StmtParam::Get("$my_param")]
///   }
/// )
/// ```
use std::borrow::Cow;
use std::str::FromStr;

use sqlparser::ast::FunctionArg;

use crate::webserver::http::SingleOrVec;
use crate::webserver::http_request_info::RequestInfo;

use super::{
    execute_queries::DbConn, sql::function_args_to_stmt_params,
    sqlpage_functions::functions::SqlPageFunctionName,
};
use anyhow::Context as _;

/// Represents a parameter to a SQL statement.
/// Objects of this type are created during SQL parsing.
/// Every time a SQL statement is executed, the parameters are evaluated to produce the actual values that are passed to the database.
/// Parameter evaluation can involve asynchronous operations, and extracting values from the request.
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum StmtParam {
    Get(String),
    Post(String),
    PostOrGet(String),
    Error(String),
    Literal(String),
    Null,
    Concat(Vec<StmtParam>),
    JsonObject(Vec<StmtParam>),
    JsonArray(Vec<StmtParam>),
    FunctionCall(SqlPageFunctionCall),
}

impl std::fmt::Display for StmtParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StmtParam::Get(name) => write!(f, "?{name}"),
            StmtParam::Post(name) => write!(f, ":{name}"),
            StmtParam::PostOrGet(name) => write!(f, "${name}"),
            StmtParam::Literal(x) => write!(f, "'{}'", x.replace('\'', "''")),
            StmtParam::Null => write!(f, "NULL"),
            StmtParam::Concat(items) => {
                write!(f, "CONCAT(")?;
                for item in items {
                    write!(f, "{item}, ")?;
                }
                write!(f, ")")
            }
            StmtParam::JsonObject(items) => {
                write!(f, "JSON_OBJECT(")?;
                for item in items {
                    write!(f, "{item}, ")?;
                }
                write!(f, ")")
            }
            StmtParam::JsonArray(items) => {
                write!(f, "JSON_ARRAY(")?;
                for item in items {
                    write!(f, "{item}, ")?;
                }
                write!(f, ")")
            }
            StmtParam::FunctionCall(call) => write!(f, "{call}"),
            StmtParam::Error(x) => {
                if let Some((i, _)) = x.char_indices().nth(21) {
                    write!(f, "## {}... ##", &x[..i])
                } else {
                    write!(f, "## {x} ##")
                }
            }
        }
    }
}

/// Represents a call to a `sqlpage.` function.
/// Objects of this type are created during SQL parsing and used to evaluate the function at runtime.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SqlPageFunctionCall {
    pub function: SqlPageFunctionName,
    pub arguments: Vec<StmtParam>,
}

impl SqlPageFunctionCall {
    pub fn from_func_call(func_name: &str, arguments: &mut [FunctionArg]) -> anyhow::Result<Self> {
        let function = SqlPageFunctionName::from_str(func_name)?;
        let arguments = function_args_to_stmt_params(arguments)?;
        Ok(Self {
            function,
            arguments,
        })
    }

    pub async fn evaluate<'a, 'b>(
        &self,
        request: &'a RequestInfo,
        db_connection: &'b mut DbConn,
    ) -> anyhow::Result<Option<Cow<'a, str>>> {
        let mut params = Vec::with_capacity(self.arguments.len());
        for param in &self.arguments {
            params.push(Box::pin(extract_req_param(param, request, db_connection)).await?);
        }
        log::trace!("Starting function call to {self}");
        let result = self
            .function
            .evaluate(request, db_connection, params)
            .await?;
        log::trace!(
            "Function call to {self} returned: {}",
            result.as_deref().unwrap_or("NULL")
        );
        Ok(result)
    }
}

impl std::fmt::Display for SqlPageFunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}(", self.function)?;
        // interleave the arguments with commas
        let mut it = self.arguments.iter();
        if let Some(x) = it.next() {
            write!(f, "{x}")?;
        }
        for x in it {
            write!(f, ", {x}")?;
        }
        write!(f, ")")
    }
}

/// Extracts the value of a parameter from the request.
/// Returns `Ok(None)` when NULL should be used as the parameter value.
pub(super) async fn extract_req_param<'a, 'b>(
    param: &StmtParam,
    request: &'a RequestInfo,
    db_connection: &'b mut DbConn,
) -> anyhow::Result<Option<Cow<'a, str>>> {
    Ok(match param {
        // sync functions
        StmtParam::Get(x) => request.get_variables.get(x).map(SingleOrVec::as_json_str),
        StmtParam::Post(x) => request.post_variables.get(x).map(SingleOrVec::as_json_str),
        StmtParam::PostOrGet(x) => if let Some(v) = request.post_variables.get(x) {
            log::warn!("Deprecation warning! ${x} was used to reference a form field value (a POST variable) instead of a URL parameter. This will stop working soon. Please use :{x} instead.");
            Some(v)
        } else {
            request.get_variables.get(x)
        }
        .map(SingleOrVec::as_json_str),
        StmtParam::Error(x) => anyhow::bail!("{}", x),
        StmtParam::Literal(x) => Some(Cow::Owned(x.to_string())),
        StmtParam::Null => None,
        StmtParam::Concat(args) => concat_params(&args[..], request, db_connection).await?,
        StmtParam::JsonObject(args) => json_object_params(&args[..], request, db_connection).await?,
        StmtParam::JsonArray(args) => json_array_params(&args[..], request, db_connection).await?,
        StmtParam::FunctionCall(func) => func.evaluate(request, db_connection).await.with_context(|| {
            format!(
                "Error in function call {func}.\nExpected {:#}",
                func.function
            )
        })?,
    })
}

async fn concat_params<'a, 'b>(
    args: &[StmtParam],
    request: &'a RequestInfo,
    db_connection: &'b mut DbConn,
) -> anyhow::Result<Option<Cow<'a, str>>> {
    let mut result = String::new();
    for arg in args {
        let Some(arg) = Box::pin(extract_req_param(arg, request, db_connection)).await? else {
            return Ok(None);
        };
        result.push_str(&arg);
    }
    Ok(Some(Cow::Owned(result)))
}

async fn json_object_params<'a, 'b>(
    args: &[StmtParam],
    request: &'a RequestInfo,
    db_connection: &'b mut DbConn,
) -> anyhow::Result<Option<Cow<'a, str>>> {
    use serde::{ser::SerializeMap, Serializer};
    let mut result = Vec::new();
    let mut ser = serde_json::Serializer::new(&mut result);
    let mut map_ser = ser.serialize_map(Some(args.len()))?;
    let mut it = args.iter();
    while let Some(key) = it.next() {
        let key = Box::pin(extract_req_param(key, request, db_connection)).await?;
        map_ser.serialize_key(&key)?;
        let val = it
            .next()
            .ok_or_else(|| anyhow::anyhow!("Odd number of arguments in JSON_OBJECT"))?;

        match val {
            StmtParam::JsonObject(args) => {
                let raw_json = Box::pin(json_object_params(args, request, db_connection)).await?;
                let obj = cow_to_raw_json(&raw_json);
                map_ser.serialize_value(&obj)?;
            }
            StmtParam::JsonArray(args) => {
                let raw_json = Box::pin(json_array_params(args, request, db_connection)).await?;
                let obj = cow_to_raw_json(&raw_json);
                map_ser.serialize_value(&obj)?;
            }
            val => {
                let evaluated = Box::pin(extract_req_param(val, request, db_connection)).await?;
                map_ser.serialize_value(&evaluated)?;
            }
        };
    }
    map_ser.end()?;
    Ok(Some(Cow::Owned(String::from_utf8(result)?)))
}

async fn json_array_params<'a, 'b>(
    args: &[StmtParam],
    request: &'a RequestInfo,
    db_connection: &'b mut DbConn,
) -> anyhow::Result<Option<Cow<'a, str>>> {
    use serde::{ser::SerializeSeq, Serializer};
    let mut result = Vec::new();
    let mut ser = serde_json::Serializer::new(&mut result);
    let mut seq_ser = ser.serialize_seq(Some(args.len()))?;
    for element in args {
        match element {
            StmtParam::JsonObject(args) => {
                let raw_json = json_object_params(args, request, db_connection).await?;
                let obj = cow_to_raw_json(&raw_json);
                seq_ser.serialize_element(&obj)?;
            }
            StmtParam::JsonArray(args) => {
                let raw_json = Box::pin(json_array_params(args, request, db_connection)).await?;
                let obj = cow_to_raw_json(&raw_json);
                seq_ser.serialize_element(&obj)?;
            }
            element => {
                let evaluated =
                    Box::pin(extract_req_param(element, request, db_connection)).await?;
                seq_ser.serialize_element(&evaluated)?;
            }
        };
    }
    seq_ser.end()?;
    Ok(Some(Cow::Owned(String::from_utf8(result)?)))
}

fn cow_to_raw_json<'a>(
    raw_json: &'a Option<Cow<'a, str>>,
) -> Option<&'a serde_json::value::RawValue> {
    raw_json
        .as_deref()
        .map(serde_json::from_str::<&'a serde_json::value::RawValue>)
        .map(Result::unwrap)
}
