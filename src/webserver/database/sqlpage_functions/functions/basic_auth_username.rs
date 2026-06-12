use super::*;

/// Returns the username from the HTTP basic auth header, if present.
/// Otherwise, returns an HTTP 401 Unauthorized error.
pub(super) async fn basic_auth_username(request: &RequestInfo) -> anyhow::Result<&str> {
    Ok(extract_basic_auth(request)?.user_id())
}
