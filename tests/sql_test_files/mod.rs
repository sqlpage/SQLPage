use actix_web::test;
use sqlpage::AppState;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[actix_web::test]
async fn run_all_sql_test_files() {
    let app_data = crate::common::make_app_data().await;
    let test_files = get_sql_test_files();

    // Create a shutdown channel for the echo server
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    // Start echo server once for all tests
    let (echo_handle, port) = crate::common::start_echo_server(shutdown_rx);

    // Wait for echo server to be ready
    wait_for_echo_server(port).await;

    for test_file in test_files {
        let test_result = run_sql_test(&test_file, &app_data, &echo_handle, port).await;
        assert_test_result(test_result, &test_file);
    }

    // Signal the echo server to shut down
    let _ = shutdown_tx.send(());
    // Wait for echo server to complete after all tests with a timeout
    match tokio::time::timeout(Duration::from_secs(2), echo_handle).await {
        Ok(_) => (),
        Err(_) => panic!("Echo server did not shut down within 2 seconds"),
    }
}

async fn wait_for_echo_server(port: u16) {
    let client = awc::Client::default();
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(5);

    while start.elapsed() < timeout {
        match client.get(format!("http://localhost:{port}/")).send().await {
            Ok(_) => return,
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
        }
    }
    panic!("Echo server did not become ready within 5 seconds");
}

fn get_sql_test_files() -> Vec<std::path::PathBuf> {
    let path = std::path::Path::new("tests/sql_test_files");
    std::fs::read_dir(path)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "sql" {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

use std::fmt::Write;

#[derive(Debug)]
enum TestResult {
    Success(String),
    Skipped(String),
}

async fn run_sql_test(
    test_file: &std::path::Path,
    app_data: &actix_web::web::Data<AppState>,
    _echo_handle: &JoinHandle<()>,
    port: u16,
) -> anyhow::Result<TestResult> {
    let test_file_path = test_file.to_string_lossy().replace('\\', "/");
    let stem = test_file.file_stem().unwrap().to_str().unwrap();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());
    let db_type = if db_url.starts_with("postgres") {
        "postgres"
    } else if db_url.starts_with("mysql") || db_url.starts_with("mariadb") {
        "mysql"
    } else if db_url.starts_with("mssql") {
        "mssql"
    } else if db_url.starts_with("sqlite") {
        "sqlite"
    } else {
        panic!("Unknown database type in DATABASE_URL: {db_url}");
    };

    if stem.contains(&format!("_no{db_type}")) {
        return Ok(TestResult::Skipped(format!(
            "Test skipped for database type: {db_type}"
        )));
    }

    let mut query_params = "x=1".to_string();
    if test_file_path.contains("fetch") {
        write!(query_params, "&echo_port={port}").unwrap();
    }
    let req_str = format!("/{test_file_path}?{query_params}");

    let resp = tokio::time::timeout(
        Duration::from_secs(5),
        crate::common::req_path_with_app_data(&req_str, app_data.clone()),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Test request timed out after 5 seconds: {}", e))??;

    let body = test::read_body(resp).await;
    Ok(TestResult::Success(String::from_utf8(body.to_vec())?))
}

fn assert_test_result(result: anyhow::Result<TestResult>, test_file: &std::path::Path) {
    match result {
        Ok(TestResult::Skipped(reason)) => {
            println!("⏭️  Skipped {}: {}", test_file.display(), reason);
        }
        Ok(TestResult::Success(body)) => {
            assert_html_response(&body, test_file);
            let lowercase_body = body.to_lowercase();
            let stem = test_file.file_stem().unwrap().to_str().unwrap().to_string();

            if stem.starts_with("it_works") {
                assert_it_works_tests(&body, &lowercase_body, test_file);
            } else if stem.starts_with("error_") {
                assert_error_tests(&stem, &lowercase_body, test_file);
            }
        }
        Err(e) => panic!("Failed to get response for {}: {}", test_file.display(), e),
    }
}

fn assert_html_response(body: &str, test_file: &std::path::Path) {
    assert!(
        body.starts_with("<!DOCTYPE html>"),
        "Response to {} is not HTML",
        test_file.display()
    );
}

fn assert_it_works_tests(body: &str, lowercase_body: &str, test_file: &std::path::Path) {
    if body.contains("<code class=\"sqlpage-error-description\">") {
        let error_desc = body
            .split("<code class=\"sqlpage-error-description\">")
            .nth(1)
            .and_then(|s| s.split("</code>").next())
            .unwrap_or("Unknown error");
        panic!(
            "\n\n❌ TEST FAILED: {} ❌\n\nFull Response:\n{}\n\nError Description: {}\n",
            test_file.display(),
            error_desc,
            body
        );
    }

    assert!(
        body.contains("It works !"),
        "{body}\n❌ Error in file {test_file:?} ❌\n",
    );
    assert!(
        !lowercase_body.contains("error"),
        "{}\n{}\nexpected to not contain: error",
        test_file.display(),
        body
    );
}

fn assert_error_tests(stem: &str, lowercase_body: &str, test_file: &std::path::Path) {
    let expected_error = stem
        .strip_prefix("error_")
        .unwrap()
        .replace('_', " ")
        .to_lowercase();
    assert!(
        lowercase_body.contains(&expected_error),
        "{}\n{}\nexpected to contain: {}",
        test_file.display(),
        lowercase_body,
        expected_error
    );
    assert!(
        lowercase_body.contains("error"),
        "{}\n{}\nexpected to contain: error",
        test_file.display(),
        lowercase_body
    );
}
