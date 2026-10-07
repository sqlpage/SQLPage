use actix_web::http::StatusCode;
use sqlpage::app_config::{AppConfig, DevOrProd};

use crate::common::{response_for, response_with, test_config};

/// Returns the value of the `Server-Timing` response header, failing the test
/// when the header is missing.
fn server_timing_header(resp: &actix_web::dev::ServiceResponse) -> &str {
    resp.headers()
        .get("Server-Timing")
        .expect("Server-Timing header should be present")
        .to_str()
        .unwrap()
}

#[actix_web::test]
async fn test_server_timing_disabled_in_production() -> actix_web::Result<()> {
    let resp = response_with(
        "/tests/sql_test_files/component_rendering/simple.sql",
        AppConfig {
            environment: DevOrProd::Production,
            ..test_config()
        },
    )
    .await?;

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        resp.headers().get("Server-Timing").is_none(),
        "Server-Timing header should not be present in production mode"
    );
    Ok(())
}

#[actix_web::test]
async fn test_server_timing_enabled_in_development() -> actix_web::Result<()> {
    let resp = response_with(
        "/tests/sql_test_files/data/postgres_cast_syntax.sql",
        AppConfig {
            environment: DevOrProd::Development,
            ..test_config()
        },
    )
    .await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let header_value = server_timing_header(&resp);

    for token in ["sql_file", "parse_req", "bind_params", "db_conn", "row"] {
        assert!(
            header_value.contains(&format!("{token};dur=")),
            "Should contain {token} timing: {header_value}"
        );
    }

    Ok(())
}

#[actix_web::test]
async fn test_server_timing_format() -> actix_web::Result<()> {
    let resp = response_for("/tests/sql_test_files/data/postgres_cast_syntax.sql").await;

    assert_eq!(resp.status(), StatusCode::OK);
    let header_value = server_timing_header(&resp);

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
    let resp = response_with(
        "/tests/server_timing/redirect_test.sql",
        AppConfig {
            environment: DevOrProd::Development,
            ..test_config()
        },
    )
    .await?;

    assert_eq!(
        resp.status(),
        StatusCode::FOUND,
        "Response should be a redirect"
    );
    let header_value = server_timing_header(&resp);

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
