use crate::webserver::http_request_info::RequestInfo;

pub(super) async fn client_ip(request: &RequestInfo) -> Option<String> {
    Some(request.client_ip?.to_string())
}
