use std::borrow::Cow;

use crate::webserver::http_request_info::RequestInfo;

/// Returns a specific claim from the ID token.
pub(super) async fn user_info<'a>(
    request: &'a RequestInfo,
    claim: Cow<'a, str>,
) -> anyhow::Result<Option<String>> {
    let Some(claims) = &request.oidc_claims else {
        return Ok(None);
    };

    // Match against known OIDC claims accessible via direct methods.
    let claim_value_str = match claim.as_ref() {
        // Core Claims
        "iss" => Some(claims.issuer().to_string()),
        // aud requires serialization: handled separately if needed
        "exp" => Some(claims.expiration().timestamp().to_string()),
        "iat" => Some(claims.issue_time().timestamp().to_string()),
        "sub" => Some(claims.subject().to_string()),
        "auth_time" => claims.auth_time().map(|t| t.timestamp().to_string()),
        "nonce" => claims.nonce().map(|n| n.secret().clone()), // Assuming Nonce has secret()
        "acr" => claims.auth_context_ref().map(|acr| acr.to_string()),
        // amr requires serialization: handled separately if needed
        "azp" => claims.authorized_party().map(|azp| azp.to_string()),
        "at_hash" => claims.access_token_hash().map(|h| h.to_string()),
        "c_hash" => claims.code_hash().map(|h| h.to_string()),

        // Standard Claims (Profile Scope - subset)
        "name" => claims
            .name()
            .and_then(|n| n.get(None))
            .map(|s| s.to_string()),
        "given_name" => claims
            .given_name()
            .and_then(|n| n.get(None))
            .map(|s| s.to_string()),
        "family_name" => claims
            .family_name()
            .and_then(|n| n.get(None))
            .map(|s| s.to_string()),
        "middle_name" => claims
            .middle_name()
            .and_then(|n| n.get(None))
            .map(|s| s.to_string()),
        "nickname" => claims
            .nickname()
            .and_then(|n| n.get(None))
            .map(|s| s.to_string()),
        "preferred_username" => claims.preferred_username().map(|u| u.to_string()),
        "profile" => claims
            .profile()
            .and_then(|n| n.get(None))
            .map(|url_claim| url_claim.as_str().to_string()),
        "picture" => claims
            .picture()
            .and_then(|n| n.get(None))
            .map(|url_claim| url_claim.as_str().to_string()),
        "website" => claims
            .website()
            .and_then(|n| n.get(None))
            .map(|url_claim| url_claim.as_str().to_string()),
        "gender" => claims.gender().map(|g| g.to_string()), // Assumes GenderClaim impls ToString
        "birthdate" => claims.birthdate().map(|b| b.to_string()), // Assumes Birthdate impls ToString
        "zoneinfo" => claims.zoneinfo().map(|z| z.to_string()),   // Assumes ZoneInfo impls ToString
        "locale" => claims.locale().map(std::string::ToString::to_string), // Assumes Locale impls ToString
        "updated_at" => claims.updated_at().map(|t| t.timestamp().to_string()),

        // Standard Claims (Email Scope)
        "email" => claims.email().map(|e| e.to_string()),
        "email_verified" => claims.email_verified().map(|b| b.to_string()),

        // Standard Claims (Phone Scope)
        "phone_number" => claims.phone_number().map(|p| p.to_string()),
        "phone_number_verified" => claims.phone_number_verified().map(|b| b.to_string()),
        additional_claim => claims
            .additional_claims()
            .0
            .get(additional_claim)
            .map(std::string::ToString::to_string),
    };

    Ok(claim_value_str)
}
