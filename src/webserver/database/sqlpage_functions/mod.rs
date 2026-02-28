mod function_definition_macro;
mod function_traits;
pub(super) mod functions;
mod http_fetch_request;
mod url_parameters;

use sqlparser::ast::FunctionArg;

use crate::webserver::http_request_info::{ExecutionContext, RequestInfo};

use super::sql::ParamExtractContext;
use super::syntax_tree::SqlPageFunctionCall;
use super::syntax_tree::StmtParam;

pub(super) fn func_call_to_param(
    func_name: &str,
    arguments: &mut [FunctionArg],
    ctx: &ParamExtractContext,
) -> StmtParam {
    SqlPageFunctionCall::from_func_call(func_name, arguments, ctx).map_or_else(
        |e| StmtParam::Error(format!("{e:#}")),
        StmtParam::FunctionCall,
    )
}
