mod function_definition_macro;
pub(super) mod functions;

use awc::http::{header::USER_AGENT, Method};
use sqlparser::ast::FunctionArg;
use std::{borrow::Cow, str::FromStr};

use crate::webserver::{http::SingleOrVec, http_request_info::RequestInfo};

use super::syntax_tree::SqlPageFunctionCall;
use super::syntax_tree::StmtParam;

use super::sql::{extract_variable_argument, FormatArguments};
use anyhow::{anyhow, Context};

pub(super) fn func_call_to_param(func_name: &str, arguments: &mut [FunctionArg]) -> StmtParam {
    match func_name {
        "fetch" => StmtParam::Fetch(Box::new(extract_variable_argument("fetch", arguments))),
        _ => SqlPageFunctionCall::from_func_call(func_name, arguments)
            .with_context(|| {
                format!(
                    "Invalid function call: sqlpage.{func_name}({})",
                    FormatArguments(arguments)
                )
            })
            .map_or_else(
                |e| StmtParam::Error(format!("{e:#}")),
                StmtParam::FunctionCall,
            ),
    }
}

type HeaderVec<'a> = Vec<(Cow<'a, str>, Cow<'a, str>)>;
#[derive(serde::Deserialize)]
struct Req<'b> {
    #[serde(borrow)]
    url: Cow<'b, str>,
    #[serde(borrow)]
    method: Option<Cow<'b, str>>,
    #[serde(borrow, deserialize_with = "deserialize_map_to_vec_pairs")]
    headers: HeaderVec<'b>,
    #[serde(borrow)]
    body: Option<&'b serde_json::value::RawValue>,
}

fn deserialize_map_to_vec_pairs<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<HeaderVec<'de>, D::Error> {
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Vec<(Cow<'de, str>, Cow<'de, str>)>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some((key, value)) = map.next_entry()? {
                vec.push((key, value));
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_map(Visitor)
}

async fn fetch<'a>(
    param0: &StmtParam,
    request: &'a RequestInfo,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    let Some(fetch_target) = Box::pin(extract_req_param(param0, request)).await? else {
        log::debug!("fetch: first argument is NULL, returning NULL");
        return Ok(None);
    };
    let client = awc::Client::builder()
        .add_default_header((USER_AGENT, env!("CARGO_PKG_NAME")))
        .finish();
    let res = if fetch_target.starts_with("http") {
        client.get(fetch_target.as_ref()).send()
    } else {
        let r = serde_json::from_str::<'_, Req<'_>>(&fetch_target)
            .with_context(|| format!("Invalid request: {fetch_target}"))?;
        let method = if let Some(method) = r.method {
            Method::from_str(&method)?
        } else {
            Method::GET
        };
        let mut req = client.request(method, r.url.as_ref());
        for (k, v) in r.headers {
            req = req.insert_header((k.as_ref(), v.as_ref()));
        }
        if let Some(body) = r.body {
            let val = body.get();
            // The body can be either json, or a string representing a raw body
            let body = if val.starts_with('"') {
                serde_json::from_str::<'_, String>(val)?
            } else {
                req = req.content_type("application/json");
                val.to_owned()
            };
            req.send_body(body)
        } else {
            req.send()
        }
    };
    log::info!("Fetching {fetch_target}");
    let mut res = res
        .await
        .map_err(|e| anyhow!("Unable to fetch {fetch_target}: {e}"))?;
    log::debug!("Finished fetching {fetch_target}. Status: {}", res.status());
    let body = res.body().await?.to_vec();
    let response_str = String::from_utf8(body)?.into();
    log::debug!("Fetch response: {response_str}");
    Ok(Some(response_str))
}

/// Extracts the value of a parameter from the request.
/// Returns `Ok(None)` when NULL should be used as the parameter value.
pub(super) async fn extract_req_param_as_json(
    param: &StmtParam,
    request: &RequestInfo,
) -> anyhow::Result<serde_json::Value> {
    if let Some(val) = extract_req_param(param, request).await? {
        Ok(serde_json::Value::String(val.into_owned()))
    } else {
        Ok(serde_json::Value::Null)
    }
}

/// Extracts the value of a parameter from the request.
/// Returns `Ok(None)` when NULL should be used as the parameter value.
pub(super) async fn extract_req_param<'a>(
    param: &StmtParam,
    request: &'a RequestInfo,
) -> anyhow::Result<Option<Cow<'a, str>>> {
    Ok(match param {
        // async functions
        StmtParam::Fetch(inner) => fetch(inner, request).await?,
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
