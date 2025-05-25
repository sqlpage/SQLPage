use actix_web::{
    http::{self, StatusCode},
    test,
};

use crate::common::req_path;

#[actix_web::test]
async fn test_privileged_paths_are_not_accessible() {
    let resp_result = req_path("/sqlpage/migrations/0001_init.sql").await;
    assert!(
        resp_result.is_err(),
        "Accessing a migration file should be forbidden, but received success: {resp_result:?}"
    );
    let resp = resp_result.unwrap_err().error_response();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let srv_resp = actix_web::test::TestRequest::default().to_srv_response(resp);
    let body = test::read_body(srv_resp).await;
    assert!(String::from_utf8_lossy(&body)
        .to_lowercase()
        .contains("forbidden"),);
}

#[actix_web::test]
async fn test_404_fallback() {
    for f in [
        "/tests/errors/does_not_exist.sql",
        "/tests/errors/does_not_exist.html",
        "/tests/errors/does_not_exist/",
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
