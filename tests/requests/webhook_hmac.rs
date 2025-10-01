use actix_web::{http::StatusCode, test};
use sqlpage::webserver::http::main_handler;

use crate::common::get_request_to;

#[actix_web::test]
async fn test_webhook_hmac_invalid_signature() -> actix_web::Result<()> {
    // Set up environment variable for webhook secret
    std::env::set_var("WEBHOOK_SECRET", "test-secret-key");

    let webhook_body = r#"{"order_id":12345,"total":"99.99"}"#;
    let invalid_signature = "96a5f6f65c85a2d4d1f3a37813ab2c0b44041bdc17691fbb0884e3eb52b7c54b";

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
        .expect("Should have Location header")
        .to_str()
        .unwrap();
    assert_eq!(location, "/error.sql?err=bad_webhook_signature");
    Ok(())
}

#[actix_web::test]
async fn test_webhook_hmac_valid_signature() -> actix_web::Result<()> {
    // Set up environment variable for webhook secret
    std::env::set_var("WEBHOOK_SECRET", "test-secret-key");

    let webhook_body = r#"{"order_id":12345,"total":"99.99"}"#;
    let valid_signature = "260b3b5ead84843645588af82d5d2c3fe24c598a950d36c45438c3a5f5bb941c";

    let req = get_request_to("/tests/webhook_hmac_validation.sql")
        .await?
        .insert_header(("content-type", "application/json"))
        .insert_header(("X-Webhook-Signature", valid_signature))
        .set_payload(webhook_body)
        .to_srv_request();

    let resp = main_handler(req).await?;

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

    let location = resp
        .headers()
        .get("location")
        .expect("Should have Location header")
        .to_str()
        .unwrap();
    assert_eq!(location, "/error.sql?err=bad_webhook_signature");

    Ok(())
}
