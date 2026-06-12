use super::*;

/// Returns the protocol of the current request (http or https).
pub(super) async fn protocol(request: &RequestInfo) -> &str {
    &request.protocol
}
