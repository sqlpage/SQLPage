use super::*;

/// Returns the path component of the URL of the current request.
pub(super) async fn path(request: &RequestInfo) -> &str {
    &request.path
}
