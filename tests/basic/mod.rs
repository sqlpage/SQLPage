use actix_web::{
    body::MessageBody,
    http::{self},
    test,
};

use crate::common::req_path;

#[actix_web::test]
async fn test_index_ok() {
    let resp = req_path("/").await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = test::read_body(resp).await;
    assert!(body.starts_with(b"<!DOCTYPE html>"));
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("It works !"));
    assert!(!body.contains("error"));
}

#[actix_web::test]
async fn test_access_config_forbidden() {
    let resp_result = req_path("/sqlpage/sqlpage.json").await;
    assert!(resp_result.is_err(), "Accessing the config file should be forbidden, but we received a response: {resp_result:?}");
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
    let resp = req_path("/tests/it_works.txt").await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = test::read_body(resp).await;
    assert_eq!(&body, &b"It works !"[..]);
}

#[actix_web::test]
async fn test_spaces_in_file_names() {
    let resp = req_path("/tests/core/spaces%20in%20file%20name.sql")
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("It works !"), "{body_str}");
}
