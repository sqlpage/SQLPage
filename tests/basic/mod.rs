use actix_web::{
    body::MessageBody,
    http::{self},
    test,
};

use crate::common::{response_for, response_with, test_config};

#[actix_web::test]
async fn test_index_ok() {
    let resp = response_for("/").await;
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = crate::common::read_body_string(resp).await;
    assert!(body.starts_with("<!DOCTYPE html>"));
    assert!(body.contains("It works !"));
    assert!(!body.contains("error"));
}

#[actix_web::test]
async fn test_access_config_forbidden() {
    let resp_result = response_with("/sqlpage/sqlpage.json", test_config()).await;
    assert!(
        resp_result.is_err(),
        "Accessing the config file should be forbidden, but we received a response: {resp_result:?}"
    );
    let resp = resp_result.unwrap_err().error_response();
    assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    assert!(
        String::from_utf8_lossy(&resp.into_body().try_into_bytes().unwrap())
            .to_lowercase()
            .contains("forbidden"),
    );
}

#[actix_web::test]
async fn test_static_files() {
    let resp = response_for("/tests/it_works.txt").await;
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = test::read_body(resp).await;
    assert_eq!(&body, &b"It works !"[..]);
}

#[actix_web::test]
async fn test_spaces_in_file_names() {
    let resp = response_for("/tests/core/spaces%20in%20file%20name.sql").await;
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body_str = crate::common::read_body_string(resp).await;
    assert!(body_str.contains("It works !"), "{body_str}");
}
