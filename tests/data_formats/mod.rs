use actix_web::{
    http::{header, StatusCode},
    test::{self, TestRequest},
};
use sqlpage::webserver::http::main_handler;

use crate::common::{get_request_to, make_app_data};

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
    let app_data = make_app_data().await;
    let req = TestRequest::get()
        .uri("/tests/data_formats/accept_json_test.sql")
        .insert_header((header::ACCEPT, "application/json"))
        .app_data(app_data)
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/json"
    );
    let body = test::read_body(resp).await;
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(body_json.is_array(), "response should be a JSON array");
    let arr = body_json.as_array().unwrap();
    assert_eq!(arr.len(), 4);
    assert_eq!(arr[0]["component"], "text");
    assert_eq!(arr[0]["contents"], "Hello World");
    assert_eq!(arr[1]["component"], "table");
    assert_eq!(arr[2]["id"], 1);
    assert_eq!(arr[2]["name"], "Alice");
    assert_eq!(arr[3]["id"], 2);
    assert_eq!(arr[3]["name"], "Bob");
    Ok(())
}

#[actix_web::test]
async fn test_accept_ndjson_returns_jsonlines() -> actix_web::Result<()> {
    let app_data = make_app_data().await;
    let req = TestRequest::get()
        .uri("/tests/data_formats/accept_json_test.sql")
        .insert_header((header::ACCEPT, "application/x-ndjson"))
        .app_data(app_data)
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/x-ndjson"
    );
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    let lines: Vec<&str> = body_str.trim().lines().collect();
    assert_eq!(lines.len(), 4);

    let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(first["component"], "text");
    assert_eq!(first["contents"], "Hello World");

    let second: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(second["component"], "table");

    let third: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
    assert_eq!(third["id"], 1);
    assert_eq!(third["name"], "Alice");

    let fourth: serde_json::Value = serde_json::from_str(lines[3]).unwrap();
    assert_eq!(fourth["id"], 2);
    assert_eq!(fourth["name"], "Bob");
    Ok(())
}

#[actix_web::test]
async fn test_accept_html_returns_html() -> actix_web::Result<()> {
    let app_data = make_app_data().await;
    let req = TestRequest::get()
        .uri("/tests/data_formats/accept_json_test.sql")
        .insert_header((header::ACCEPT, "text/html"))
        .app_data(app_data)
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/html; charset=utf-8"
    );
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("<html"), "response should contain HTML");
    assert!(
        body_str.contains("Hello World"),
        "response should contain the text content"
    );
    Ok(())
}

#[actix_web::test]
async fn test_accept_wildcard_returns_html() -> actix_web::Result<()> {
    let app_data = make_app_data().await;
    let req = TestRequest::get()
        .uri("/tests/data_formats/accept_json_test.sql")
        .insert_header((header::ACCEPT, "*/*"))
        .app_data(app_data)
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/html; charset=utf-8"
    );
    Ok(())
}

#[actix_web::test]
async fn test_accept_json_redirect_still_works() -> actix_web::Result<()> {
    let app_data = make_app_data().await;
    let req = TestRequest::get()
        .uri("/tests/data_formats/accept_json_redirect_test.sql")
        .insert_header((header::ACCEPT, "application/json"))
        .app_data(app_data)
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::FOUND);
    assert_eq!(resp.headers().get(header::LOCATION).unwrap(), "/target");
    Ok(())
}

#[actix_web::test]
async fn test_accept_json_headers_still_work() -> actix_web::Result<()> {
    let app_data = make_app_data().await;
    let req = TestRequest::get()
        .uri("/tests/data_formats/accept_json_headers_test.sql")
        .insert_header((header::ACCEPT, "application/json"))
        .app_data(app_data)
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::CREATED);
    let set_cookie = resp.headers().get(header::SET_COOKIE).unwrap();
    assert!(
        set_cookie
            .to_str()
            .unwrap()
            .contains("test_cookie=cookie_value"),
        "Cookie should be set: {:?}",
        set_cookie
    );
    let body = test::read_body(resp).await;
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(body_json.is_array());
    let arr = body_json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["component"], "text");
    assert_eq!(arr[0]["contents"], "Created");
    Ok(())
}
