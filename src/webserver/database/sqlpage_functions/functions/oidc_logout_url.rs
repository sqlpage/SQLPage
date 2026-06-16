use std::borrow::Cow;

use crate::webserver::{http_request_info::RequestInfo, single_or_vec::SingleOrVec};

pub(super) async fn oidc_logout_url<'a>(
    request: &'a RequestInfo,
    redirect_uri: Option<Cow<'a, str>>,
) -> anyhow::Result<Option<String>> {
    let Some(oidc_state) = &request.app_state.oidc_state else {
        return Ok(None);
    };

    let redirect_uri = redirect_uri.as_deref().unwrap_or("/");

    if !crate::webserver::oidc::is_safe_relative_redirect(redirect_uri) {
        anyhow::bail!(
            "oidc_logout_url: redirect_uri must be a relative path starting with a single '/'. Got: {redirect_uri}"
        );
    }

    // Bind the logout URL to the current session so that it can only log out
    // the browser it was generated for, never a different user's session. Use
    // the first cookie value, matching how verification reads the auth cookie
    // (HttpRequest::cookie returns the first cookie of that name); signing a
    // JSON array of duplicate cookies here would never match verification.
    let session_token = request
        .cookies
        .get("sqlpage_auth")
        .map(SingleOrVec::first_str);

    let logout_url = oidc_state
        .config
        .create_logout_url(redirect_uri, session_token);

    Ok(Some(logout_url))
}
