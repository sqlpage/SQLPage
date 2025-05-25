use actix_web::{
    http::StatusCode,
    test,
};
use sqlpage::webserver::http::main_handler;

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
    assert!(
        String::from_utf8_lossy(&body)
            .to_lowercase()
            .contains("forbidden"),
    );
} 