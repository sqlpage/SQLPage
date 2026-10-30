use std::borrow::Cow;

use anyhow::anyhow;

use crate::webserver::http_request_info::ExecutionContext;

/// Returns all variables in the request as a JSON object.
pub(super) async fn variables<'a>(
    request: &'a ExecutionContext,
    get_or_post: Option<Cow<'a, str>>,
) -> anyhow::Result<String> {
    Ok(if let Some(get_or_post) = get_or_post {
        if get_or_post.eq_ignore_ascii_case("get") {
            serde_json::to_string(&request.url_params)?
        } else if get_or_post.eq_ignore_ascii_case("post") {
            serde_json::to_string(&request.post_variables)?
        } else if get_or_post.eq_ignore_ascii_case("set") {
            serde_json::to_string(&*request.set_variables.borrow())?
        } else {
            return Err(anyhow!(
                "Expected 'get', 'post', or 'set' as the argument to sqlpage.variables"
            ));
        }
    } else {
        let variables = crate::webserver::request_variables::VariableAccess::new(
            &request.url_params,
            &request.post_variables,
            &request.set_variables,
        );
        serde_json::to_string(&variables)?
    })
}
