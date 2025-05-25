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
async fn test_404() {
    for f in [
        "/does_not_exist.sql",
        "/does_not_exist.html",
        "/does_not_exist/",
    ] {
        let resp_result = req_path(f).await;
        let resp = resp_result.unwrap_err().error_response();
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND, "{f} isnt 404");
    }
}

#[actix_web::test]
async fn test_404_fallback() {
    for f in [
        "/tests/does_not_exist.sql",
        "/tests/does_not_exist.html",
        "/tests/does_not_exist/",
    ] {
        let resp_result = req_path(f).await;
        let resp = resp_result.unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK, "{f} isnt 200");

        let body = test::read_body(resp).await;
        assert!(body.starts_with(b"<!DOCTYPE html>"));
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("But the "));
        assert!(body.contains("404.sql"));
        assert!(body.contains("file saved the day!"));
        assert!(!body.contains("error"));
    }
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
