use crate::webserver::http_request_info::RequestInfo;

/// Returns the raw request body as a string.
/// If the request body is not valid UTF-8, invalid characters are replaced with the Unicode replacement character.
/// Returns NULL if there is no request body or if the request content type is
/// application/x-www-form-urlencoded or multipart/form-data (in this case, the body is accessible via the `post_variables` field).
pub(super) async fn request_body(request: &RequestInfo) -> Option<String> {
    let raw_body = request.raw_body.as_ref()?;
    Some(String::from_utf8_lossy(raw_body).to_string())
}
