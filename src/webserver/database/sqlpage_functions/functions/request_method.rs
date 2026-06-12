use crate::webserver::http_request_info::RequestInfo;

pub(super) async fn request_method(request: &RequestInfo) -> String {
    request.method.to_string()
}
