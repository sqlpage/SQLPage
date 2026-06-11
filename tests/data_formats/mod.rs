use actix_web::{
    http::{StatusCode, header},
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
    let body_json: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body_json,
        serde_json::json!([{"message": "It works!"}, {"cool": "cool"}])
    );
    Ok(())
}

#[actix_web::test]
async fn test_csv_body() -> actix_web::Result<()> {
    let app_data = make_app_data().await;
    if matches!(
        app_data.db.info.database_type,
        sqlpage::webserver::database::SupportedDatabase::Oracle
    ) {
        return Ok(());
    }

    let req = crate::common::get_request_to_with_data("/tests/data_formats/csv_data.sql", app_data)
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
async fn test_csv_filename_header_injection() -> actix_web::Result<()> {
    use actix_web::http::header::ContentDisposition;

    // The csv `filename` is `report.csv; filename*=UTF-8''evil.html`, which
    // tries to smuggle an extra `filename*` parameter into the
    // Content-Disposition header. The attacker-supplied value must NOT create a
    // second, agent-preferred parameter; it has to stay inside a single,
    // properly quoted `filename` value.
    let resp = crate::common::req_path("/tests/data_formats/csv_filename_injection.sql")
        .await
        .expect("request failed");
    assert_eq!(resp.status(), StatusCode::OK);
    let raw = resp
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .expect("missing Content-Disposition header")
        .clone();

    // Parse the header the same way a compliant user agent would, so that
    // `;` and `=` inside a quoted value are treated as literal data, not as
    // parameter separators.
    let disposition = ContentDisposition::from_raw(&raw)
        .unwrap_or_else(|e| panic!("invalid Content-Disposition {raw:?}: {e}"));

    // No extended `filename*` parameter must have been injected.
    assert!(
        disposition.get_filename_ext().is_none(),
        "attacker injected a separate filename* parameter: {raw:?}"
    );
    // The whole attacker payload must remain a single, inert `filename` value.
    assert_eq!(
        disposition.get_filename(),
        Some("report.csv; filename*=UTF-8''evil.html"),
        "the attacker payload should stay inside a single filename value: {raw:?}"
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
        "/tests/sql_test_files/component_rendering/simple.sql",
        "application/json",
    )
    .await?;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/json"
    );
    let body_json: serde_json::Value = test::read_body_json(resp).await;
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
        "/tests/sql_test_files/component_rendering/simple.sql",
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
    let resp = req_with_accept(
        "/tests/sql_test_files/component_rendering/simple.sql",
        "text/html",
    )
    .await?;
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
    let resp = req_with_accept(
        "/tests/sql_test_files/component_rendering/simple.sql",
        "*/*",
    )
    .await?;
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

/// Builds an `AppState` running in production mode.
async fn make_prod_app_data() -> actix_web::web::Data<sqlpage::AppState> {
    crate::common::init_log();
    let mut config = crate::common::test_config();
    config.environment = sqlpage::app_config::DevOrProd::Production;
    crate::common::make_app_data_from_config(config).await
}

async fn req_prod_with_accept(path: &str, accept: &str) -> String {
    let app_data = make_prod_app_data().await;
    let req = TestRequest::get()
        .uri(path)
        .insert_header((header::ACCEPT, accept))
        .app_data(app_data)
        .to_srv_request();
    let resp = main_handler(req)
        .await
        .expect("request should not fail at the handler level");
    let body = test::read_body(resp).await;
    String::from_utf8(body.to_vec()).unwrap()
}

/// In production, a SQL error that happens mid-stream must not leak the SQL
/// statement, the source file path, or the raw database error text.
fn assert_no_sql_leak(body: &str, context: &str) {
    for needle in [
        "definitely_missing_table_xyz",
        "The SQL statement sent by SQLPage",
        "error_leak.sql",
        ".sql\"",
    ] {
        assert!(
            !body.contains(needle),
            "production error response leaked {needle:?} in {context}: {body}"
        );
    }
    assert!(
        body.to_lowercase().contains("administrator"),
        "production error response should contain a generic message in {context}: {body}"
    );
}

#[actix_web::test]
async fn test_prod_json_error_does_not_leak_sql() {
    let body = req_prod_with_accept(
        "/tests/data_formats/json_error_leak.sql",
        "application/json",
    )
    .await;
    assert!(
        body.contains("before the error"),
        "the good row should still be streamed: {body}"
    );
    assert_no_sql_leak(&body, "json error");
}

#[actix_web::test]
async fn test_prod_csv_error_does_not_leak_sql() {
    let app_data = make_prod_app_data().await;
    if matches!(
        app_data.db.info.database_type,
        sqlpage::webserver::database::SupportedDatabase::Oracle
    ) {
        return;
    }
    let req = TestRequest::get()
        .uri("/tests/data_formats/csv_error_leak.sql")
        .insert_header((header::ACCEPT, "text/csv"))
        .app_data(app_data)
        .to_srv_request();
    let resp = main_handler(req).await.expect("handler should not fail");
    let body = test::read_body(resp).await;
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body.contains("before the error"),
        "the good row should still be streamed: {body}"
    );
    assert_no_sql_leak(&body, "csv error");
}

/// A CSV page can hit an error before its first data row (so no header has been
/// written and `columns` is empty). The generic error message must still be
/// emitted instead of an empty record.
#[actix_web::test]
async fn test_prod_csv_error_before_any_row_still_reports() {
    let app_data = make_prod_app_data().await;
    if matches!(
        app_data.db.info.database_type,
        sqlpage::webserver::database::SupportedDatabase::Oracle
    ) {
        return;
    }
    let req = TestRequest::get()
        .uri("/tests/data_formats/csv_error_no_rows.sql")
        .insert_header((header::ACCEPT, "text/csv"))
        .app_data(app_data)
        .to_srv_request();
    let resp = main_handler(req).await.expect("handler should not fail");
    let body = test::read_body(resp).await;
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body.to_lowercase().contains("administrator"),
        "csv error before the first row must still emit the generic error message: {body:?}"
    );
    assert_no_sql_leak(&body, "csv error before any row");
}

/// An author may only intend a page to be served as HTML, but a client can
/// request it with `Accept: application/json` and pick the JSON renderer.
/// In production that path must not leak SQL text either.
#[actix_web::test]
async fn test_prod_html_page_requested_as_json_does_not_leak_sql() {
    let body = req_prod_with_accept(
        "/tests/data_formats/text_error_leak.sql",
        "application/json",
    )
    .await;
    assert_no_sql_leak(&body, "text page requested as json");
}
