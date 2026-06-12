use super::*;

pub(super) async fn client_ip(request: &RequestInfo) -> Option<String> {
    Some(request.client_ip?.to_string())
}
