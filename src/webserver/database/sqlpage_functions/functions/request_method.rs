use super::*;

pub(super) async fn request_method(request: &RequestInfo) -> String {
    request.method.to_string()
}
