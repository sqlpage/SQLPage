use actix_web::{body::to_bytes, http::StatusCode};
use sqlpage::webserver::http::main_handler;

use crate::common::get_request_to;

async fn assert_sql_response(path: &str, expected_status: StatusCode) {
    let request = get_request_to(path).await.unwrap().to_srv_request();
    let response = match main_handler(request).await {
        Ok(response) => response.into_parts().1,
        Err(error) => error.error_response(),
    };
    assert_eq!(response.status(), expected_status, "{path}");
    let body = to_bytes(response.into_body()).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body.contains("SQLPAGE_SOURCE_CANARY"), "{path}: {body}");
    if expected_status == StatusCode::OK {
        assert!(body.starts_with("<!DOCTYPE html>"), "{path}: {body}");
        assert!(body.contains("SQL executed"), "{path}: {body}");
    }
}

#[actix_web::rt::test(system = "crate::common::TestSystem")]
async fn mixed_case_sql_file_is_executed() {
    assert_sql_response("/tests/core/mixed_case.%53ql", StatusCode::OK).await;
}

#[actix_web::rt::test(system = "crate::common::TestSystem")]
async fn trailing_space_sql_path_is_forbidden() {
    assert_sql_response("/tests/core/sql_source.sql%20", StatusCode::FORBIDDEN).await;
}

#[cfg(windows)]
async fn assert_windows_alias_response(path: &str, expected_status: StatusCode) {
    // Establish that this spelling really reaches the SQL source on this filesystem.
    let source = std::fs::read(path.strip_prefix('/').unwrap()).unwrap();
    assert_eq!(source, include_bytes!("sql_source.sql"));
    assert_sql_response(path, expected_status).await;
}

#[cfg(windows)]
#[actix_web::rt::test(system = "crate::common::TestSystem")]
async fn windows_mixed_case_sql_alias_is_executed() {
    assert_windows_alias_response("/tests/core/sql_source.SQL", StatusCode::OK).await;
}

#[cfg(windows)]
#[actix_web::rt::test(system = "crate::common::TestSystem")]
async fn windows_trailing_dot_sql_alias_is_forbidden() {
    assert_windows_alias_response("/tests/core/sql_source.sql.", StatusCode::FORBIDDEN).await;
}

#[cfg(windows)]
#[actix_web::rt::test(system = "crate::common::TestSystem")]
async fn windows_ntfs_stream_sql_alias_is_forbidden() {
    assert_windows_alias_response("/tests/core/sql_source.sql::$DATA", StatusCode::FORBIDDEN).await;
}
