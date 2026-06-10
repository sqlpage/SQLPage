use actix_web::{
    http::{self, StatusCode},
    test,
};
use sqlpage::{AppState, webserver::http::main_handler};

use crate::common::{make_app_data_from_config, req_path, req_path_with_app_data, test_config};
mod basic_auth;
mod invalid_header;

/// Sends a direct unprivileged GET request through the main handler and returns the
/// resulting HTTP status, whether the handler returned an `Ok` response or an `Err`.
async fn direct_request_status(path: &str, app_data: actix_web::web::Data<AppState>) -> StatusCode {
    let req = test::TestRequest::get()
        .uri(path)
        .app_data(app_data)
        .insert_header(actix_web::http::header::Accept::html())
        .to_srv_request();
    match main_handler(req).await {
        Ok(resp) => resp.status(),
        Err(e) => e.error_response().status(),
    }
}

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
    let app_data = make_app_data_from_config(config).await;

    // 1. A trusted page primes the cache by loading the reserved file with privilege.
    let prime = req_path_with_app_data("/tests/errors/prime_private_cache.sql", app_data.clone())
        .await
        .expect("priming page should render");
    assert_eq!(prime.status(), StatusCode::OK);
    let prime_body = String::from_utf8(test::read_body(prime).await.to_vec()).unwrap();
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
        let status = direct_request_status(path, app_data.clone()).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "{path} must be forbidden even after privileged cache priming, got {status}"
        );
    }
}

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

#[actix_web::test]
async fn test_default_404() {
    for f in [
        "/i-do-not-exist.html",
        "/i-do-not-exist.sql",
        "/i-do-not-exist/",
    ] {
        let resp_result = req_path(f).await;
        let resp = resp_result.unwrap();
        assert_eq!(
            resp.status(),
            http::StatusCode::NOT_FOUND,
            "{f} should return 404"
        );

        let body = test::read_body(resp).await;
        assert!(body.starts_with(b"<!DOCTYPE html>"));
        let body = String::from_utf8(body.to_vec()).unwrap();
        let msg = "The page you were looking for does not exist";
        assert!(
            body.contains(msg),
            "{f} should contain '{msg}' but got:\n{body}"
        );
        assert!(!body.contains("error"));
    }
}

#[actix_web::test]
async fn test_default_404_with_redirect() {
    let resp_result = req_path("/i-do-not-exist").await;
    let resp = resp_result.unwrap();
    assert_eq!(
        resp.status(),
        http::StatusCode::NOT_FOUND,
        "/i-do-not-exist should return 404"
    );

    let resp_result = req_path("/i-do-not-exist/").await;
    let resp = resp_result.unwrap();
    assert_eq!(
        resp.status(),
        http::StatusCode::NOT_FOUND,
        "/i-do-not-exist/ should return 404"
    );

    let body = test::read_body(resp).await;
    assert!(body.starts_with(b"<!DOCTYPE html>"));
    let body = String::from_utf8(body.to_vec()).unwrap();
    let msg = "The page you were looking for does not exist";
    assert!(
        body.contains(msg),
        "/i-do-not-exist/ should contain '{msg}' but got:\n{body}"
    );
    assert!(!body.contains("error"));
}

#[actix_web::test]
async fn test_default_404_when_request_path_descends_into_file() {
    let resp_result = req_path("/tests/it_works.txt/site/wp-includes/wlwmanifest.xml").await;
    let resp = resp_result.unwrap();
    assert_eq!(
        resp.status(),
        http::StatusCode::NOT_FOUND,
        "descending into a file path should behave like a missing resource"
    );

    let body = test::read_body(resp).await;
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("The page you were looking for does not exist"));
    assert!(!body.contains("error"));
}
