use actix_web::http::StatusCode;

use crate::common::{make_app_data_from_config, response_with, response_with_data, test_config};
mod basic_auth;
mod invalid_header;

/// Regression test for a cache privilege-escalation bug.
///
/// `sqlpage.run_sql(...)` loads include files with privilege, so it is allowed to
/// load reserved files under the `sqlpage/` prefix and stores their parsed form in
/// the shared `sql_file_cache`. A subsequent *direct* unprivileged HTTP request for
/// that same reserved path must still be rejected with 403, even while the cache
/// entry is fresh. Before the fix, the fresh cache hit short-circuited the
/// unprivileged path guard and the private SQL was executed and served.
#[actix_web::test]
async fn test_private_path_not_accessible_after_privileged_cache_priming() {
    // Keep cache entries "fresh" so the bug (skipping the path guard on fresh hits) is exercised.
    let mut config = test_config();
    config.cache_stale_duration_ms = Some(60_000);
    let app_data = make_app_data_from_config(config).await.unwrap();

    // 1. A trusted page primes the cache by loading the reserved file with privilege.
    let prime = response_with_data("/tests/errors/prime_private_cache.sql", app_data.clone())
        .await
        .expect("priming page should render");
    assert_eq!(prime.status(), StatusCode::OK);
    let prime_body = crate::common::read_body_string(prime).await;
    assert!(
        prime_body.contains("private cache bypass secret"),
        "priming page should have executed the private file via run_sql, got: {prime_body}"
    );

    // 2. A direct unprivileged HTTP request for the reserved path must stay forbidden.
    for path in [
        "/sqlpage/private_cache_bypass_test.sql",
        // Extensionless alias: routing appends .sql and finds the fresh cache entry.
        "/sqlpage/private_cache_bypass_test",
    ] {
        let status = response_with_data(path, app_data.clone())
            .await
            .unwrap_err()
            .as_response_error()
            .status_code();
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "{path} must be forbidden even after privileged cache priming, got {status}"
        );
    }
}

#[actix_web::test]
async fn test_privileged_paths_are_not_accessible() {
    let resp_result = response_with("/sqlpage/migrations/0001_init.sql", test_config()).await;
    assert!(
        resp_result.is_err(),
        "Accessing a migration file should be forbidden, but received success: {resp_result:?}"
    );
    let resp = resp_result.unwrap_err().error_response();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let body = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
    assert!(
        String::from_utf8_lossy(&body)
            .to_lowercase()
            .contains("forbidden"),
    );
}

#[actix_web::test]
async fn test_404_fallback() {
    let app_data = crate::common::make_app_data().await;
    for f in [
        "/tests/errors/does_not_exist.sql",
        "/tests/errors/does_not_exist.html",
        "/tests/errors/does_not_exist/",
    ] {
        let resp = response_with_data(f, app_data.clone()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "{f} isnt 200");
        let body = crate::common::read_body_string(resp).await;
        assert!(body.starts_with("<!DOCTYPE html>"));
        for marker in ["But the ", "404.sql", "file saved the day!"] {
            assert!(body.contains(marker), "{f} should contain {marker:?}");
        }
        assert!(!body.contains("error"), "{f} should not contain an error");
    }
}

#[actix_web::test]
async fn test_default_404() {
    let app_data = crate::common::make_app_data().await;
    let msg = "The page you were looking for does not exist";
    for f in [
        "/i-do-not-exist.html",
        "/i-do-not-exist.sql",
        "/i-do-not-exist/",
        "/i-do-not-exist",
        "/tests/it_works.txt/site/wp-includes/wlwmanifest.xml",
    ] {
        let resp = response_with_data(f, app_data.clone()).await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "{f} should return 404"
        );
        let body = crate::common::read_body_string(resp).await;
        assert!(body.starts_with("<!DOCTYPE html>"), "{f} is not HTML");
        assert!(body.contains(msg), "{f} should contain {msg:?}");
        assert!(!body.contains("error"), "{f} should not contain an error");
    }
}

#[actix_web::test]
async fn test_requesting_a_directory_is_not_found() {
    let resp_result = response_with("/tests/errors/is_a_directory.d", test_config()).await;
    let status = match resp_result {
        Ok(resp) => resp.status(),
        Err(e) => e.as_response_error().status_code(),
    };
    assert_eq!(status, StatusCode::NOT_FOUND);
}
