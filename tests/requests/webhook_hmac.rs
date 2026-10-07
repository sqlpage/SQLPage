use actix_web::{http::StatusCode, test};

use crate::common::{request_for, response_from};

/// Builds a webhook validation request, optionally signed with `signature`.
/// Sets `WEBHOOK_SECRET` so every test signs against the same secret.
async fn webhook_request(
    signature: Option<&str>,
) -> actix_web::Result<actix_web::dev::ServiceResponse> {
    // Set up environment variable for webhook secret
    unsafe {
        std::env::set_var("WEBHOOK_SECRET", "test-secret-key");
    }

    let webhook_body = r#"{"order_id":12345,"total":"99.99"}"#;
    let mut req = request_for("/tests/webhook_hmac_validation.sql")
        .insert_header(("content-type", "application/json"));
    if let Some(signature) = signature {
        req = req.insert_header(("X-Webhook-Signature", signature));
    }
    response_from(req.set_payload(webhook_body)).await
}

#[actix_web::test]
async fn test_webhook_hmac_invalid_signature() -> actix_web::Result<()> {
    let invalid_signature = "96a5f6f65c85a2d4d1f3a37813ab2c0b44041bdc17691fbb0884e3eb52b7c54b";
    let resp = webhook_request(Some(invalid_signature)).await?;

    // Should redirect to error page when signature is invalid
    assert!(
        resp.status() == StatusCode::FOUND || resp.status() == StatusCode::SEE_OTHER,
        "Expected redirect (302 or 303) for invalid signature, got: {}",
        resp.status()
    );

    let location = resp
        .headers()
        .get("location")
        .expect("Should have Location header")
        .to_str()
        .unwrap();
    assert_eq!(location, "/error.sql?err=bad_webhook_signature");
    Ok(())
}

#[actix_web::test]
async fn test_webhook_hmac_valid_signature() -> actix_web::Result<()> {
    let valid_signature = "260b3b5ead84843645588af82d5d2c3fe24c598a950d36c45438c3a5f5bb941c";
    let resp = webhook_request(Some(valid_signature)).await?;

    // Should return success when signature is valid
    assert_eq!(resp.status(), StatusCode::OK, "200 resp for signed req");
    assert!(!resp.headers().contains_key("location"), "no redirect");

    assert_eq!(
        test::read_body_json::<serde_json::Value, _>(resp).await,
        serde_json::json! ({"msg": "Webhook signature is valid !"})
    );
    Ok(())
}

#[actix_web::test]
async fn test_webhook_hmac_missing_signature() -> actix_web::Result<()> {
    // Don't include the X-Webhook-Signature header
    let resp = webhook_request(None).await?;

    let location = resp
        .headers()
        .get("location")
        .expect("Should have Location header")
        .to_str()
        .unwrap();
    assert_eq!(location, "/error.sql?err=bad_webhook_signature");

    Ok(())
}
