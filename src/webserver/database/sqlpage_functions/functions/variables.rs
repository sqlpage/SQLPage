use super::*;

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
        use serde::{Serializer, ser::SerializeMap};
        let mut res = Vec::new();
        let mut serializer = serde_json::Serializer::new(&mut res);
        let set_vars = request.set_variables.borrow();
        let len = request.url_params.len() + request.post_variables.len() + set_vars.len();
        let mut ser = serializer.serialize_map(Some(len))?;
        let mut seen_keys = std::collections::HashSet::new();
        for (k, v) in &*set_vars {
            seen_keys.insert(k);
            ser.serialize_entry(k, v)?;
        }
        for (k, v) in &request.post_variables {
            if seen_keys.insert(k) {
                ser.serialize_entry(k, v)?;
            }
        }
        for (k, v) in &request.url_params {
            if seen_keys.insert(k) {
                ser.serialize_entry(k, v)?;
            }
        }
        ser.end()?;
        String::from_utf8(res)?
    })
}
