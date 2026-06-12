use crate::webserver::http_request_info::RequestInfo;

/// Returns the raw request body encoded in base64.
/// Returns NULL if there is no request body or if the request content type is
/// application/x-www-form-urlencoded or multipart/form-data (in this case, the body is accessible via the `post_variables` field).
pub(super) async fn request_body_base64(request: &RequestInfo) -> Option<String> {
    let raw_body = request.raw_body.as_ref()?;
    let mut base64_string = String::with_capacity((raw_body.len() * 4).div_ceil(3));
    base64::Engine::encode_string(
        &base64::engine::general_purpose::STANDARD,
        raw_body,
        &mut base64_string,
    );
    Some(base64_string)
}
