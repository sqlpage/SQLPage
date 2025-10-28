use actix_web::http::StatusCode;
use sqlpage::webserver::http::main_handler;

use crate::common::{get_request_to, make_app_data_from_config, test_config};

#[actix_web::test]
async fn test_server_timing_disabled_in_production() -> actix_web::Result<()> {
    let mut config = test_config();
    config.environment = sqlpage::app_config::DevOrProd::Production;
    let app_data = make_app_data_from_config(config).await;

    let req = crate::common::get_request_to_with_data(
        "/tests/sql_test_files/it_works_simple.sql",
        app_data,
    )
    .await?
    .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        resp.headers().get("Server-Timing").is_none(),
        "Server-Timing header should not be present in production mode"
    );
    Ok(())
}

#[actix_web::test]
async fn test_server_timing_enabled_in_development() -> actix_web::Result<()> {
    let mut config = test_config();
    config.environment = sqlpage::app_config::DevOrProd::Development;
    let app_data = make_app_data_from_config(config).await;

    let req = crate::common::get_request_to_with_data(
        "/tests/sql_test_files/it_works_sqrt.sql",
        app_data,
    )
    .await?
    .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let server_timing_header = resp
        .headers()
        .get("Server-Timing")
        .expect("Server-Timing header should be present in development mode");
    let header_value = server_timing_header.to_str().unwrap();

    assert!(
        header_value.contains("sql_file;dur="),
        "Should contain sql_file timing: {header_value}"
    );
    assert!(
        header_value.contains("parse_req;dur="),
        "Should contain parse_req timing: {header_value}"
    );
    assert!(
        header_value.contains("bind_params;dur="),
        "Should contain bind_params timing: {header_value}"
    );
    assert!(
        header_value.contains("db_conn;dur="),
        "Should contain db_conn timing: {header_value}"
    );
    assert!(
        header_value.contains("row;dur="),
        "Should contain row timing: {header_value}"
    );

    Ok(())
}

#[actix_web::test]
async fn test_server_timing_format() -> actix_web::Result<()> {
    let req = get_request_to("/tests/sql_test_files/it_works_sqrt.sql")
        .await?
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let server_timing_header = resp.headers().get("Server-Timing").unwrap();
    let header_value = server_timing_header.to_str().unwrap();

    let parts: Vec<&str> = header_value.split(", ").collect();
    assert!(parts.len() >= 5, "Should have at least 5 timing events");

    for part in parts {
        assert!(
            part.contains(";dur="),
            "Each part should have name;dur= format: {part}"
        );
        let dur_parts: Vec<&str> = part.split(";dur=").collect();
        assert_eq!(dur_parts.len(), 2, "Should have name and duration: {part}");
        let duration: f64 = dur_parts[1]
            .parse()
            .expect("Duration should be a valid number");
        assert!(
            duration >= 0.0,
            "Duration should be non-negative: {duration}"
        );
    }

    Ok(())
}

#[actix_web::test]
async fn test_server_timing_in_redirect() -> actix_web::Result<()> {
    let mut config = test_config();
    config.environment = sqlpage::app_config::DevOrProd::Development;
    let app_data = make_app_data_from_config(config).await;

    let req =
        crate::common::get_request_to_with_data("/tests/server_timing/redirect_test.sql", app_data)
            .await?
            .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(
        resp.status(),
        StatusCode::FOUND,
        "Response should be a redirect"
    );
    let server_timing_header = resp
        .headers()
        .get("Server-Timing")
        .expect("Server-Timing header should be present in redirect responses");
    let header_value = server_timing_header.to_str().unwrap();

    assert!(
        !header_value.is_empty(),
        "Server-Timing header should not be empty: {header_value}"
    );
    assert!(
        header_value.contains(";dur="),
        "Server-Timing header should contain timing events: {header_value}"
    );

    Ok(())
}
