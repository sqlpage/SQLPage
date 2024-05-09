mod function_definition_macro;
mod function_traits;
pub(super) mod functions;
mod http_fetch_request;

use sqlparser::ast::FunctionArg;
use std::borrow::Cow;

use crate::webserver::{http::SingleOrVec, http_request_info::RequestInfo};

use super::syntax_tree::SqlPageFunctionCall;
use super::syntax_tree::StmtParam;

use super::sql::FormatArguments;
use anyhow::Context;

pub(super) fn func_call_to_param(func_name: &str, arguments: &mut [FunctionArg]) -> StmtParam {
    SqlPageFunctionCall::from_func_call(func_name, arguments)
        .with_context(|| {
            format!(
                "Invalid function call: sqlpage.{func_name}({})",
                FormatArguments(arguments)
            )
        })
        .map_or_else(
            |e| StmtParam::Error(format!("{e:#}")),
            StmtParam::FunctionCall,
        )
}

/// Extracts the value of a parameter from the request.
/// Returns `Ok(None)` when NULL should be used as the parameter value.
pub(super) async fn extract_req_param<'a>(
    param: &StmtParam,
    request: &'a RequestInfo,
) -> anyhow::Result<Option<Cow<'a, str>>> {
    Ok(match param {
        // sync functions
        StmtParam::Get(x) => request.get_variables.get(x).map(SingleOrVec::as_json_str),
        StmtParam::Post(x) => request.post_variables.get(x).map(SingleOrVec::as_json_str),
        StmtParam::GetOrPost(x) => request
            .post_variables
            .get(x)
            .or_else(|| request.get_variables.get(x))
            .map(SingleOrVec::as_json_str),
        StmtParam::Error(x) => anyhow::bail!("{}", x),
        StmtParam::Literal(x) => Some(Cow::Owned(x.to_string())),
        StmtParam::Concat(args) => concat_params(&args[..], request).await?,
        StmtParam::FunctionCall(func) => func.evaluate(request).await.with_context(|| {
            format!(
                "Error in function call {func}.\nExpected {:#}",
                func.function
            )
        })?,
    })
}

async fn concat_params<'a>(
    args: &[StmtParam],
    request: &'a RequestInfo,
) -> anyhow::Result<Option<Cow<'a, str>>> {
    let mut result = String::new();
    for arg in args {
        let Some(arg) = Box::pin(extract_req_param(arg, request)).await? else {
            return Ok(None);
        };
        result.push_str(&arg);
    }
    Ok(Some(Cow::Owned(result)))
}
