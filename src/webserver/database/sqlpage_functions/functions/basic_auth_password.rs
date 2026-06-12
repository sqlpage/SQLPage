use super::*;

/// Returns the password from the HTTP basic auth header, if present.
pub(super) async fn basic_auth_password(request: &RequestInfo) -> anyhow::Result<&str> {
    let password = extract_basic_auth(request)?.password().ok_or_else(|| {
        anyhow::Error::new(ErrorWithStatus {
            status: actix_web::http::StatusCode::UNAUTHORIZED,
        })
    })?;
    Ok(password)
}

pub(super) fn extract_basic_auth(
    request: &RequestInfo,
) -> anyhow::Result<&actix_web_httpauth::headers::authorization::Basic> {
    request
        .basic_auth
        .as_ref()
        .ok_or_else(|| {
            anyhow::Error::new(ErrorWithStatus {
                status: actix_web::http::StatusCode::UNAUTHORIZED,
            })
        })
        .with_context(|| "Expected the user to be authenticated with HTTP basic auth")
}
