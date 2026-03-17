use crate::common::req_path;
use actix_web::{http::StatusCode, test};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde_json::{json, Value};

struct InvalidHeaderCase {
    name: &'static str,
    properties: Value,
}

async fn assert_invalid_header_response(case: &InvalidHeaderCase) {
    let properties = serde_json::to_string(&case.properties).unwrap();
    let properties = utf8_percent_encode(&properties, NON_ALPHANUMERIC).to_string();
    let path = format!("/tests/errors/invalid_header.sql?properties={properties}");

    let resp = req_path(&path).await.unwrap_or_else(|err| {
        panic!(
            "{} should return an error response instead of failing the request: {err:#}",
            case.name
        )
    });

    assert_eq!(
        resp.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "{} should return a 500 response",
        case.name
    );

    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.to_lowercase().contains("error"),
        "{} should render an error response body, got:\n{body_str}",
        case.name
    );
}

#[actix_web::test]
async fn test_invalid_header_components_return_an_error_response() {
    let cases = vec![
        InvalidHeaderCase {
            name: "cookie domain with newline",
            properties: json!({
                "component": "cookie",
                "name": "boom",
                "value": "x",
                "domain": "\n",
            }),
        },
        InvalidHeaderCase {
            name: "cookie path with DEL",
            properties: json!({
                "component": "cookie",
                "name": "boom",
                "value": "x",
                "path": "\u{007f}",
            }),
        },
        InvalidHeaderCase {
            name: "http_header value with carriage return",
            properties: json!({
                "component": "http_header",
                "X-Test": "\r",
            }),
        },
        InvalidHeaderCase {
            name: "redirect link with NUL",
            properties: json!({
                "component": "redirect",
                "link": "\u{0000}",
            }),
        },
        InvalidHeaderCase {
            name: "authentication link with unit separator",
            properties: json!({
                "component": "authentication",
                "link": "\u{001f}",
            }),
        },
        InvalidHeaderCase {
            name: "download filename with newline",
            properties: json!({
                "component": "download",
                "data_url": "data:text/plain,ok",
                "filename": "\n",
            }),
        },
        InvalidHeaderCase {
            name: "csv filename with carriage return",
            properties: json!({
                "component": "csv",
                "filename": "\r",
            }),
        },
        InvalidHeaderCase {
            name: "csv title with NUL",
            properties: json!({
                "component": "csv",
                "title": "\u{0000}",
            }),
        },
    ];

    for case in &cases {
        assert_invalid_header_response(case).await;
    }
}
