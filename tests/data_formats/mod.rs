use actix_web::{
    http::{header, StatusCode},
    test::{self, TestRequest},
};
use sqlpage::webserver::http::main_handler;

use crate::common::{get_request_to, make_app_data};

async fn req_with_accept(
    path: &str,
    accept: &str,
) -> actix_web::Result<actix_web::dev::ServiceResponse> {
    let app_data = make_app_data().await;
    let req = TestRequest::get()
        .uri(path)
        .insert_header((header::ACCEPT, accept))
        .app_data(app_data)
        .to_srv_request();
    main_handler(req).await
}

#[actix_web::test]
async fn test_json_body() -> actix_web::Result<()> {
    let req = get_request_to("/tests/data_formats/json_data.sql")
        .await?
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/json"
    );
    let body = test::read_body(resp).await;
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body_json,
        serde_json::json!([{"message": "It works!"}, {"cool": "cool"}])
    );
    Ok(())
}

#[actix_web::test]
async fn test_csv_body() -> actix_web::Result<()> {
    let req = get_request_to("/tests/data_formats/csv_data.sql")
        .await?
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/csv; charset=utf-8"
    );
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(
        body_str,
        "id;msg\n0;Hello World !\n1;\"Tu gères ';' et '\"\"' ?\"\n"
    );
    Ok(())
}

#[actix_web::test]
async fn test_json_columns() {
    let app_data = crate::common::make_app_data().await;
    if !matches!(
        app_data.db.to_string().to_lowercase().as_str(),
        "postgres" | "sqlite"
    ) {
        log::info!("Skipping test_json_columns on database {}", app_data.db);
        return;
    }

    let resp_result = crate::common::req_path("/tests/data_formats/json_columns.sql").await;
    let resp = resp_result.expect("Failed to request /tests/data_formats/json_columns.sql");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    let body_html_escaped = body_str.replace("&quot;", "\"");
    assert!(
        !body_html_escaped.contains("error"),
        "the request should not have failed, in: {body_html_escaped}"
    );
    assert!(body_html_escaped.contains("1GB Database"));
    assert!(body_html_escaped.contains("Priority Support"));
    assert!(
        !body_html_escaped.contains("\"description\""),
        "the json should have been parsed, not returned as a string, in: {body_html_escaped}"
    );
    assert!(
        !body_html_escaped.contains("{"),
        "the json should have been parsed, not returned as a string, in: {body_html_escaped}"
    );
}

#[actix_web::test]
async fn test_accept_json_returns_json_array() -> actix_web::Result<()> {
    let resp = req_with_accept(
        "/tests/sql_test_files/it_works_simple.sql",
        "application/json",
    )
    .await?;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/json"
    );
    let body = test::read_body(resp).await;
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(body_json.is_array());
    let arr = body_json.as_array().unwrap();
    assert!(arr.len() >= 2);
    assert_eq!(arr[0]["component"], "shell");
    assert_eq!(arr[1]["component"], "text");
    Ok(())
}

#[actix_web::test]
async fn test_accept_ndjson_returns_jsonlines() -> actix_web::Result<()> {
    let resp = req_with_accept(
        "/tests/sql_test_files/it_works_simple.sql",
        "application/x-ndjson",
    )
    .await?;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/x-ndjson"
    );
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    let lines: Vec<&str> = body_str.trim().lines().collect();
    assert!(lines.len() >= 2);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(lines[0]).unwrap()["component"],
        "shell"
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(lines[1]).unwrap()["component"],
        "text"
    );
    Ok(())
}

#[actix_web::test]
async fn test_accept_html_returns_html() -> actix_web::Result<()> {
    let resp = req_with_accept("/tests/sql_test_files/it_works_simple.sql", "text/html").await?;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/html; charset=utf-8"
    );
    let body = test::read_body(resp).await;
    assert!(body.starts_with(b"<!DOCTYPE html>"));
    Ok(())
}

#[actix_web::test]
async fn test_accept_wildcard_returns_html() -> actix_web::Result<()> {
    let resp = req_with_accept("/tests/sql_test_files/it_works_simple.sql", "*/*").await?;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/html; charset=utf-8"
    );
    Ok(())
}

#[actix_web::test]
async fn test_accept_json_redirect_still_works() -> actix_web::Result<()> {
    let resp =
        req_with_accept("/tests/server_timing/redirect_test.sql", "application/json").await?;
    assert_eq!(resp.status(), StatusCode::FOUND);
    assert_eq!(
        resp.headers().get(header::LOCATION).unwrap(),
        "/destination.sql"
    );
    Ok(())
}
