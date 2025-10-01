use actix_web::{http::StatusCode, test};
use sqlpage::webserver::http::main_handler;

use crate::common::get_request_to;

#[actix_web::test]
async fn test_webhook_hmac_invalid_signature() -> actix_web::Result<()> {
    // Set up environment variable for webhook secret
    std::env::set_var("WEBHOOK_SECRET", "test-secret-key");

    let webhook_body = r#"{"order_id":12345,"total":"99.99"}"#;
    let invalid_signature = "invalid_signature_base64==";

    let req = get_request_to("/tests/webhook_hmac_validation.sql")
        .await?
        .insert_header(("content-type", "application/json"))
        .insert_header(("X-Webhook-Signature", invalid_signature))
        .set_payload(webhook_body)
        .to_srv_request();

    let resp = main_handler(req).await?;

    // Should redirect to error page when signature is invalid
    assert!(
        resp.status() == StatusCode::FOUND || resp.status() == StatusCode::SEE_OTHER,
        "Expected redirect (302 or 303) for invalid signature, got: {}",
        resp.status()
    );

    let location = resp
        .headers()
        .get("location")
        .expect("Should have Location header");
    let location_str = location.to_str().unwrap();
    assert!(
        location_str.contains("/error.sql"),
        "Should redirect to error page, got: {}",
        location_str
    );
    assert!(
        location_str.contains("Invalid+webhook+signature")
            || location_str.contains("Invalid%20webhook%20signature"),
        "Error message should mention invalid signature, got: {}",
        location_str
    );

    Ok(())
}

#[actix_web::test]
async fn test_webhook_hmac_valid_signature() -> actix_web::Result<()> {
    // Set up environment variable for webhook secret
    std::env::set_var("WEBHOOK_SECRET", "test-secret-key");

    let webhook_body = r#"{"order_id":12345,"total":"99.99"}"#;

    // Calculate the correct HMAC signature using the same algorithm
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(b"test-secret-key").unwrap();
    mac.update(webhook_body.as_bytes());
    let result = mac.finalize();
    let valid_signature =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result.into_bytes());

    let req = get_request_to("/tests/webhook_hmac_validation.sql")
        .await?
        .insert_header(("content-type", "application/json"))
        .insert_header(("X-Webhook-Signature", valid_signature.as_str()))
        .set_payload(webhook_body)
        .to_srv_request();

    let resp = main_handler(req).await?;

    // Should return success when signature is valid
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Expected OK status for valid signature"
    );

    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should contain success message
    assert!(
        body_str.contains("success") || body_str.contains("Success"),
        "Response should indicate success, got: {}",
        body_str
    );
    assert!(
        body_str.contains("Webhook signature verified"),
        "Response should confirm signature verification, got: {}",
        body_str
    );
    assert!(
        body_str.contains("order_id"),
        "Response should contain the webhook body, got: {}",
        body_str
    );

    Ok(())
}

#[actix_web::test]
async fn test_webhook_hmac_missing_signature() -> actix_web::Result<()> {
    // Set up environment variable for webhook secret
    std::env::set_var("WEBHOOK_SECRET", "test-secret-key");

    let webhook_body = r#"{"order_id":12345,"total":"99.99"}"#;

    // Don't include the X-Webhook-Signature header
    let req = get_request_to("/tests/webhook_hmac_validation.sql")
        .await?
        .insert_header(("content-type", "application/json"))
        .set_payload(webhook_body)
        .to_srv_request();

    let resp = main_handler(req).await?;

    // Should redirect to error page when signature is missing
    assert!(
        resp.status() == StatusCode::FOUND || resp.status() == StatusCode::SEE_OTHER,
        "Expected redirect (302 or 303) when signature header is missing, got: {}",
        resp.status()
    );

    let location = resp
        .headers()
        .get("location")
        .expect("Should have Location header");
    let location_str = location.to_str().unwrap();
    assert!(
        location_str.contains("/error.sql"),
        "Should redirect to error page, got: {}",
        location_str
    );

    Ok(())
}
