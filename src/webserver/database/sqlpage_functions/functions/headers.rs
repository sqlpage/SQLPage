use crate::webserver::http_request_info::RequestInfo;

pub(super) async fn headers(request: &RequestInfo) -> String {
    serde_json::to_string(&request.headers).unwrap_or_default()
}
