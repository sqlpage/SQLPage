use actix_web::test;
use sqlpage::AppState;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[actix_web::test]
async fn run_all_sql_test_files() {
    let app_data = crate::common::make_app_data().await;
    let test_files = get_sql_test_cases();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let (echo_handle, port) = crate::common::start_echo_server(shutdown_rx);
    wait_for_echo_server(port).await;

    for test_file in test_files {
        run_sql_test(&test_file, &app_data, &echo_handle, port).await;
    }

    let _ = shutdown_tx.send(());
    let _ = tokio::time::timeout(Duration::from_secs(2), echo_handle).await;
}

async fn wait_for_echo_server(port: u16) {
    let client = awc::Client::default();
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        if client
            .get(format!("http://localhost:{port}/"))
            .send()
            .await
            .is_ok()
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("Echo server did not become ready");
}

#[derive(Clone, Copy)]
enum SqlTestFormat {
    Html,
    Json,
}

struct SqlTestCase {
    path: std::path::PathBuf,
    format: SqlTestFormat,
}

fn get_sql_test_cases() -> Vec<SqlTestCase> {
    let mut tests = Vec::new();
    tests.extend(read_sql_tests_in_dir(
        "tests/sql_test_files/component_rendering",
        SqlTestFormat::Html,
    ));
    tests.extend(read_sql_tests_in_dir(
        "tests/sql_test_files/data",
        SqlTestFormat::Json,
    ));
    tests
}

fn read_sql_tests_in_dir(dir: &str, format: SqlTestFormat) -> Vec<SqlTestCase> {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| {
            let path = e.ok()?.path();
            if path.is_dir() || path.extension()? != "sql" {
                return None;
            }
            Some(SqlTestCase { path, format })
        })
        .collect()
}

async fn run_sql_test(
    test_case: &SqlTestCase,
    app_data: &actix_web::web::Data<AppState>,
    _echo_handle: &JoinHandle<()>,
    port: u16,
) {
    let test_file = &test_case.path;
    let test_file_path = test_file.to_string_lossy().replace('\\', "/");
    let stem = test_file.file_stem().unwrap().to_str().unwrap();

    let db_type = format!("{:?}", app_data.db.info.database_type).to_lowercase();
    if stem.contains(&format!("_no{db_type}")) {
        println!("Skipped {}: {}", test_file.display(), db_type);
        return;
    }

    let mut query_params = "x=1".to_string();
    if test_file_path.contains("fetch") {
        query_params.push_str(&format!("&echo_port={port}"));
    }
    let req_str = format!("/{test_file_path}?{query_params}");

    let use_json = matches!(test_case.format, SqlTestFormat::Json);

    let resp = tokio::time::timeout(Duration::from_secs(5), async {
        if use_json {
            crate::common::req_path_with_app_data_json(&req_str, app_data.clone()).await
        } else {
            crate::common::req_path_with_app_data(&req_str, app_data.clone()).await
        }
    })
    .await
    .unwrap_or_else(|_| panic!("Test timeout: {}", test_file.display()))
    .unwrap_or_else(|e| panic!("Request failed: {}: {}", test_file.display(), e));

    let body = String::from_utf8(test::read_body(resp).await.to_vec())
        .unwrap_or_else(|_| panic!("Invalid UTF-8: {}", test_file.display()));

    if use_json {
        assert_json_test(&body, test_file);
    } else {
        assert_html_test(&body, test_file, stem);
    }
}

fn format_error(obj: &serde_json::Map<String, serde_json::Value>) -> Option<String> {
    if obj.get("component").and_then(|v| v.as_str()) != Some("error") {
        return None;
    }
    let mut msg = String::new();
    if let Some(desc) = obj.get("description").and_then(|v| v.as_str()) {
        msg.push_str(desc);
    }
    if let Some(bt) = obj.get("backtrace").and_then(|v| v.as_array()) {
        for frame in bt {
            if let Some(s) = frame.as_str() {
                msg.push_str(&format!("\n  {}", s));
            }
        }
    }
    Some(msg)
}

fn assert_json_test(body: &str, test_file: &std::path::Path) {
    let rows: Vec<serde_json::Value> = serde_json::from_str(body)
        .unwrap_or_else(|_| panic!("Invalid JSON: {}", test_file.display()));

    assert!(
        !rows.is_empty(),
        "No rows returned: {}",
        test_file.display()
    );

    for row in rows {
        let obj = match row.as_object() {
            Some(o) => o,
            None => continue,
        };

        if let Some(err) = format_error(obj) {
            panic!("Error in {}:\n{}", test_file.display(), err);
        }

        let actual = obj
            .get("actual")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let actual_str = json_to_string(&actual);

        let expected: Vec<serde_json::Value> = obj
            .get("expected")
            .map(|v| match v {
                serde_json::Value::Array(arr) => arr.clone(),
                _ => vec![v.clone()],
            })
            .unwrap_or_default();

        let expected_contains: Vec<String> = obj
            .get("expected_contains")
            .map(|v| match v {
                serde_json::Value::Array(arr) => arr.iter().map(json_to_string).collect(),
                _ => vec![json_to_string(v)],
            })
            .unwrap_or_default();

        if expected.is_empty() && expected_contains.is_empty() {
            panic!(
                "No expected values found in {}: {}",
                test_file.display(),
                row
            );
        }

        let exact_ok = expected.is_empty() || expected.iter().any(|e| e == &actual);
        let contains_ok = expected_contains.is_empty()
            || expected_contains.iter().all(|e| actual_str.contains(e));

        if !exact_ok || !contains_ok {
            let mut msg = format!("Test failed: {}\n", test_file.display());
            if !expected.is_empty() {
                let expected_strs: Vec<String> = expected.iter().map(|d| d.to_string()).collect();
                msg.push_str(&format!("Expected: {}\n", expected_strs.join(" or ")));
            }
            if !expected_contains.is_empty() {
                msg.push_str(&format!(
                    "Expected to contain: {}\n",
                    expected_contains.join(", ")
                ));
            }
            msg.push_str(&format!("Actual:   {}\n", actual));
            panic!("{}", msg);
        }
    }
}

fn assert_html_test(body: &str, test_file: &std::path::Path, stem: &str) {
    assert!(
        body.starts_with("<!DOCTYPE html>"),
        "Not HTML: {}",
        test_file.display()
    );

    if stem.starts_with("error_") {
        let expected = stem.strip_prefix("error_").unwrap().replace('_', " ");
        assert!(
            body.to_lowercase().contains(&expected.to_lowercase()),
            "Should contain '{}': {}",
            expected,
            test_file.display()
        );
    } else {
        if let Some(error) = extract_error(body) {
            panic!("Error in {}: {}", test_file.display(), error);
        }
        assert!(
            body.contains("It works !"),
            "Should contain 'It works !': {}",
            test_file.display()
        );
        assert!(
            !body.to_lowercase().contains("error"),
            "Unexpected error: {}",
            test_file.display()
        );
    }
}

fn extract_error(body: &str) -> Option<String> {
    body.split("<code class=\"sqlpage-error-description\">")
        .nth(1)?
        .split("</code>")
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

fn json_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => v.to_string(),
    }
}
