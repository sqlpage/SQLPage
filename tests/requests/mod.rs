use actix_web::{http::StatusCode, test};
use serde_json::json;
use sqlpage::webserver::http::main_handler;

use crate::common::get_request_to;

#[actix_web::test]
async fn test_request_body() -> actix_web::Result<()> {
    let req = get_request_to("/tests/requests/request_body_test.sql")
        .await?
        .insert_header(("content-type", "text/plain"))
        .set_payload("Hello, world!")
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("Hello, world!"),
        "{body_str}\nexpected to contain: Hello, world!"
    );

    // Test with form data - should return NULL
    let req = get_request_to("/tests/requests/request_body_test.sql")
        .await?
        .insert_header(("content-type", "application/x-www-form-urlencoded"))
        .set_payload("key=value")
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("NULL"),
        "{body_str}\nexpected NULL for form data"
    );
    Ok(())
}

#[actix_web::test]
async fn test_request_body_base64() -> actix_web::Result<()> {
    let binary_data = (0u8..=255u8).collect::<Vec<_>>();
    let expected_base64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &binary_data);

    let req = get_request_to("/tests/requests/request_body_base64_test.sql")
        .await?
        .insert_header(("content-type", "application/octet-stream"))
        .set_payload(binary_data)
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains(&expected_base64),
        "{body_str}\nexpected to contain base64: {expected_base64}"
    );

    // Test with form data - should return NULL
    let req = get_request_to("/tests/requests/request_body_base64_test.sql")
        .await?
        .insert_header(("content-type", "application/x-www-form-urlencoded"))
        .set_payload("key=value")
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("NULL"),
        "{body_str}\nexpected NULL for form data"
    );
    Ok(())
}

#[actix_web::test]
async fn test_download_data_url() -> actix_web::Result<()> {
    let req = get_request_to("/tests/requests/request_download_test.sql")
        .await?
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp.headers().get("content-type").unwrap();
    assert_eq!(ct, "text/plain");
    let content_disposition = resp.headers().get("content-disposition").unwrap();
    assert_eq!(
        content_disposition,
        "attachment; filename=\"my text file.txt\""
    );
    let body = test::read_body(resp).await;
    assert_eq!(&body, &b"Hello download!"[..]);
    Ok(())
}

#[actix_web::test]
async fn test_large_form_field_roundtrip() -> actix_web::Result<()> {
    let long_string = "a".repeat(123454);
    let req = get_request_to("/tests/components/display_form_field.sql")
        .await?
        .insert_header(("content-type", "application/x-www-form-urlencoded"))
        .set_payload(["x=", &long_string].concat()) // total size is 123454 + 2 = 123456 bytes
        .to_srv_request();
    let resp = main_handler(req).await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        !body_str.contains("error"),
        "{body_str}\nshouldn't have errors"
    );
    assert!(
        body_str.contains(&long_string),
        "{body_str}\nexpected to contain long string submitted"
    );
    Ok(())
}

#[actix_web::test]
async fn test_variables_function() -> actix_web::Result<()> {
    let url = "/tests/requests/variables.sql?common=get_value&get_only=get_val";
    let req_body = "common=post_value&post_only=post_val";
    let req = get_request_to(url)
        .await?
        .insert_header(("content-type", "application/x-www-form-urlencoded"))
        .insert_header(("accept", "application/json"))
        .set_payload(req_body)
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body_json: serde_json::Value = test::read_body_json(resp).await;

    let expected = [
        [
            (
                "all_vars",
                json!({"get_only": "get_val", "common": "get_value", "post_only": "post_val", "common": "post_value"}),
            ),
            (
                "get_vars",
                json!({"get_only": "get_val", "common": "get_value"}),
            ),
            (
                "post_vars",
                json!({"post_only": "post_val", "common": "post_value"}),
            ),
            ("set_vars", json!({})),
        ],
        [
            (
                "all_vars",
                json!({"get_only": "get_val", "common": "set_common_value", "post_only": "post_val", "my_set_var": "set_value"}),
            ),
            (
                "get_vars",
                json!({"get_only": "get_val", "common": "get_value"}),
            ),
            (
                "post_vars",
                json!({"post_only": "post_val", "common": "post_value"}),
            ),
            (
                "set_vars",
                json!({"common": "set_common_value", "my_set_var": "set_value"}),
            ),
        ],
    ];

    let actual_array = body_json.as_array().expect("response is nota json array");
    for (i, expected_step) in expected.into_iter().enumerate() {
        let actual = &actual_array[i];
        for (key, expected_value) in expected_step {
            let actual_decoded: serde_json::Value =
                serde_json::from_str(actual[key].as_str().unwrap()).unwrap();
            assert_eq!(
                actual_decoded, expected_value,
                "step {i}: {key} mismatch: {actual_decoded:#} != {expected_value:#}"
            )
        }
    }

    Ok(())
}

mod webhook_hmac;
