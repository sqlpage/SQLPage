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
    let echo_handle = crate::common::start_echo_server(shutdown_rx);

    // Wait for echo server to be ready
    wait_for_echo_server().await;

    for test_file in test_files {
        let test_result = run_sql_test(&test_file, &app_data, &echo_handle).await;
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

async fn wait_for_echo_server() {
    let client = awc::Client::default();
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(5);

    while start.elapsed() < timeout {
        match client.get("http://localhost:62802/").send().await {
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

async fn run_sql_test(
    test_file: &std::path::Path,
    app_data: &actix_web::web::Data<AppState>,
    _echo_handle: &JoinHandle<()>,
) -> anyhow::Result<String> {
    let test_file_path = test_file.to_string_lossy().replace('\\', "/");
    let req_str = format!("/{test_file_path}?x=1");

    let resp = tokio::time::timeout(
        Duration::from_secs(5),
        crate::common::req_path_with_app_data(&req_str, app_data.clone()),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Test request timed out after 5 seconds: {}", e))??;

    let body = test::read_body(resp).await;
    Ok(String::from_utf8(body.to_vec())?)
}

fn assert_test_result(result: anyhow::Result<String>, test_file: &std::path::Path) {
    let (body, stem) = get_test_body_and_stem(result, test_file);
    assert_html_response(&body, test_file);
    let lowercase_body = body.to_lowercase();

    if stem.starts_with("it_works") {
        assert_it_works_tests(&body, &lowercase_body, test_file);
    } else if stem.starts_with("error_") {
        assert_error_tests(&stem, &lowercase_body, test_file);
    }
}

fn get_test_body_and_stem(
    result: anyhow::Result<String>,
    test_file: &std::path::Path,
) -> (String, String) {
    let stem = test_file.file_stem().unwrap().to_str().unwrap().to_string();
    let body = result
        .unwrap_or_else(|e| panic!("Failed to get response for {}: {}", test_file.display(), e));
    (body, stem)
}

fn assert_html_response(body: &str, test_file: &std::path::Path) {
    assert!(
        body.starts_with("<!DOCTYPE html>"),
        "Response to {} is not HTML",
        test_file.display()
    );
}

fn assert_it_works_tests(body: &str, lowercase_body: &str, test_file: &std::path::Path) {
    assert!(
        body.contains("It works !"),
        "{}\n{}\nexpected to contain: It works !",
        test_file.display(),
        body
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
