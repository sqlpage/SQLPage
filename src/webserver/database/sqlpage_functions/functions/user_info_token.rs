use super::*;

/// Returns the ID token claims as a JSON object.
pub(super) async fn user_info_token(request: &RequestInfo) -> anyhow::Result<Option<String>> {
    let Some(claims) = &request.oidc_claims else {
        return Ok(None);
    };
    Ok(Some(serde_json::to_string(claims)?))
}
