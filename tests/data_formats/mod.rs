use actix_web::{
    http::{header, StatusCode},
    test,
};
use sqlpage::webserver::http::main_handler;

use crate::common::get_request_to;

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
        "ID;MSG\n0;Hello World !\n1;\"Tu gères ';' et '\"\"' ?\"\n"
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
