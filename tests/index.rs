use std::{collections::HashMap, path::PathBuf};

use actix_web::{
    body::MessageBody,
    dev::{fn_service, ServerHandle, ServiceRequest, ServiceResponse},
    http::{
        self,
        header::{self, ContentType},
        StatusCode,
    },
    test::{self, TestRequest},
    HttpResponse,
};
use sqlpage::{
    app_config::{test_database_url, AppConfig},
    webserver::{
        self,
        http::{form_config, main_handler, payload_config},
    },
    AppState,
};

#[actix_web::test]
async fn test_index_ok() {
    let resp = req_path("/").await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = test::read_body(resp).await;
    assert!(body.starts_with(b"<!DOCTYPE html>"));
    // the body should contain the string "It works!" and should not contain the string "error"
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("It works !"));
    assert!(!body.contains("error"));
}

#[actix_web::test]
async fn test_access_config_forbidden() {
    let resp_result = req_path("/sqlpage/sqlpage.json").await;
    assert!(resp_result.is_err(), "Accessing the config file should be forbidden, but we received a response: {resp_result:?}");
    let resp = resp_result.unwrap_err().error_response();
    assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    assert!(
        String::from_utf8_lossy(&resp.into_body().try_into_bytes().unwrap())
            .to_lowercase()
            .contains("forbidden"),
    );
}

#[actix_web::test]
async fn test_404() {
    for f in [
        "/does_not_exist.sql",
        "/does_not_exist.html",
        "/does_not_exist/",
    ] {
        let resp_result = req_path(f).await;
        let resp = resp_result.unwrap_err().error_response();
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND, "{f} isnt 404");
    }
}

#[actix_web::test]
async fn test_404_fallback() {
    for f in [
        "/tests/does_not_exist.sql",
        "/tests/does_not_exist.html",
        "/tests/does_not_exist/",
    ] {
        let resp_result = req_path(f).await;
        let resp = resp_result.unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK, "{f} isnt 200");

        let body = test::read_body(resp).await;
        assert!(body.starts_with(b"<!DOCTYPE html>"));
        // the body should contain our happy string, but not the string "error"
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("But the "));
        assert!(body.contains("404.sql"));
        assert!(body.contains("file saved the day!"));
        assert!(!body.contains("error"));
    }
}

#[actix_web::test]
async fn test_concurrent_requests() {
    // send 32 requests (less than the default postgres pool size)
    // at the same time to /tests/multiple_components.sql
    let components = [
        "table", "form", "card", "datagrid", "hero", "list", "timeline",
    ];
    let app_data = make_app_data().await;
    let reqs = (0..64)
        .map(|i| {
            let component = components[i % components.len()];
            req_path_with_app_data(
                format!("/tests/any_component.sql?component={}", component),
                app_data.clone(),
            )
        })
        .collect::<Vec<_>>();
    // send all requests at the same time
    let results = futures_util::future::join_all(reqs).await;
    // check that all requests succeeded
    for result in results.into_iter() {
        let resp = result.unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        assert!(
            body.starts_with(b"<!DOCTYPE html>"),
            "Expected html doctype"
        );
        // the body should contain the string "It works!" and should not contain the string "error"
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(
            body.contains("It works !"),
            "Expected to contain: It works !, but got: {body}"
        );
        assert!(!body.contains("error"));
    }
}

fn start_echo_server() -> ServerHandle {
    async fn echo_server(mut r: ServiceRequest) -> actix_web::Result<ServiceResponse> {
        use std::io::Write;
        let mut f = Vec::new();
        write!(f, "{} {}", r.method(), r.uri()).unwrap();
        let mut sorted_headers = r.headers().into_iter().collect::<Vec<_>>();
        sorted_headers.sort_by_key(|(k, _)| k.as_str());
        for (k, v) in sorted_headers {
            if k.as_str().eq_ignore_ascii_case("date") {
                continue;
            }
            write!(f, "|{k}: ").unwrap();
            f.extend_from_slice(v.as_bytes());
        }
        f.push(b'|');
        f.extend_from_slice(&r.extract::<actix_web::web::Bytes>().await?);
        let resp = HttpResponse::Ok().body(f);
        Ok(r.into_response(resp))
    }
    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new().default_service(fn_service(echo_server))
    })
    .bind("localhost:62802")
    .unwrap()
    .shutdown_timeout(5) // shutdown timeout
    .run();

    let handle = server.handle();
    tokio::spawn(server);

    handle
}

#[actix_web::test]
async fn test_files() {
    // start a dummy server that test files can query
    let echo_server = start_echo_server();
    // Iterate over all the sql test files in the tests/ directory
    let path = std::path::Path::new("tests/sql_test_files");
    let app_data = make_app_data().await;
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let test_file_path = entry.path();
        let test_file_path_string = test_file_path.to_string_lossy().replace('\\', "/");
        let stem = test_file_path.file_stem().unwrap().to_str().unwrap();
        if test_file_path.extension().unwrap_or_default() != "sql" {
            continue;
        }
        if stem.contains(&format!("no{}", app_data.db.to_string().to_lowercase())) {
            // skipping because the test does not support the database we are using
            log::info!(
                "Skipping test file {} on database {}",
                test_file_path_string,
                app_data.db
            );
            continue;
        }
        if test_file_path_string.contains(&format!("no{}", app_data.db.to_string().to_lowercase()))
        {
            // skipping because the test does not support the database
            continue;
        }
        let req_str = format!("/{}?x=1", test_file_path_string);
        let resp = req_path_with_app_data(&req_str, app_data.clone())
            .await
            .unwrap_or_else(|_| panic!("Failed to get response for {req_str}"));
        let body = test::read_body(resp).await;
        assert!(
            body.starts_with(b"<!DOCTYPE html>"),
            "Response to {req_str} is not HTML"
        );
        // the body should contain the string "It works!" and should not contain the string "error"
        let body = String::from_utf8(body.to_vec()).unwrap();
        let lowercase_body = body.to_lowercase();
        if stem.starts_with("it_works") {
            assert!(
                body.contains("It works !"),
                "{req_str}\n{body}\nexpected to contain: It works !"
            );
            assert!(
                !lowercase_body.contains("error"),
                "{body}\nexpected to not contain: error"
            );
        } else if stem.starts_with("error_") {
            let rest = stem.strip_prefix("error_").unwrap();
            let expected_str = rest.replace('_', " ").to_lowercase();
            assert!(
                lowercase_body.contains(&expected_str),
                "{req_str}\n{lowercase_body}\nexpected to contain: {expected_str}"
            );
            assert!(
                lowercase_body.contains("error"),
                "{req_str}\n{body}\nexpected to contain: error"
            );
        }
    }
    echo_server.stop(true).await
}

#[actix_web::test]
async fn test_overwrite_variable() -> actix_web::Result<()> {
    let req = get_request_to("/tests/sql_test_files/it_works_set_variable.sql")
        .await?
        .set_form(HashMap::<&str, &str>::from_iter([(
            "what_does_it_do",
            "does not overwrite variables",
        )]))
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("It works !"),
        "{body_str}\nexpected to contain: It works !"
    );
    Ok(())
}

#[actix_web::test]
async fn test_json_body() -> actix_web::Result<()> {
    let req = get_request_to("/tests/json_data.sql")
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
    let req = get_request_to("/tests/csv_data.sql")
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

async fn test_file_upload(target: &str) -> actix_web::Result<()> {
    let req = get_request_to(target)
        .await?
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"my_file\"; filename=\"testfile.txt\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            Hello, world!\r\n\
            --1234567890--\r\n",
        )
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("Hello, world!"),
        "{body_str}\nexpected to contain: Hello, world!"
    );
    Ok(())
}

#[actix_web::test]
async fn test_file_upload_direct() -> actix_web::Result<()> {
    test_file_upload("/tests/upload_file_test.sql").await
}

#[actix_web::test]
async fn test_file_upload_through_runsql() -> actix_web::Result<()> {
    test_file_upload("/tests/upload_file_runsql_test.sql").await
}

// Diabled because of
#[actix_web::test]
async fn test_blank_file_upload_field() -> actix_web::Result<()> {
    let req = get_request_to("/tests/upload_file_test.sql")
        .await?
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"my_file\"; filename=\"\"\r\n\
            Content-Type: application/octet-stream\r\n\
            \r\n\
            \r\n\
            --1234567890--\r\n",
        )
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("No file uploaded"),
        "{body_str}\nexpected to contain: No file uploaded"
    );
    Ok(())
}

#[actix_web::test]
async fn test_file_upload_too_large() -> actix_web::Result<()> {
    // Files larger than 123456 bytes should be rejected as per the test_config
    let req = get_request_to("/tests/upload_file_test.sql")
        .await?
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"my_file\"; filename=\"testfile.txt\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            "
            .to_string()
                + "a".repeat(123457).as_str()
                + "\r\n\
            --1234567890--\r\n",
        )
        .to_srv_request();
    let err_str = main_handler(req)
        .await
        .expect_err("Expected an error response")
        .to_string();
    assert!(
        err_str.to_ascii_lowercase().contains("max file size"),
        "{err_str}\nexpected to contain: File too large"
    );
    Ok(())
}

#[actix_web::test]
async fn test_large_form_field_roundtrip() -> actix_web::Result<()> {
    // POST payloads smaller than 123456 bytes should be accepted
    let long_string = "a".repeat(123454);
    let req = get_request_to("/tests/display_form_field.sql")
        .await?
        .insert_header(("content-type", "application/x-www-form-urlencoded"))
        .set_payload(["x=", &long_string].concat()) // total size is 123454 + 2 = 123456 bytes
        .to_srv_request();
    let resp = main_handler(req).await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        !body_str.contains("error"),
        "{body_str}\nshouldn't have errors"
    );
    assert!(
        body_str.contains(&long_string),
        "{body_str}\nexpected to contain long string submitted"
    );
    Ok(())
}

#[actix_web::test]
async fn test_upload_file_data_url() -> actix_web::Result<()> {
    let req = get_request_to("/tests/upload_file_data_url_test.sql")
        .await?
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"my_file\"; filename=\"testfile.txt\"\r\n\
            Content-Type: image/svg+xml\r\n\
            \r\n\
            <svg></svg>\r\n\
            --1234567890--\r\n",
        )
        .to_srv_request();
    let resp = main_handler(req).await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    // The file name suffix was ".txt", but the content type was "application/json"
    // so the file should be treated as a JSON file
    assert_eq!(body_str, "data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=");
    Ok(())
}

#[actix_web::test]
async fn test_uploaded_file_name() -> actix_web::Result<()> {
    let req = get_request_to("/tests/uploaded_file_name_test.sql")
        .await?
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"my_file\"; filename=\"testfile.txt\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            Some plain text.\r\n\
            --1234567890--\r\n",
        )
        .to_srv_request();
    let resp = main_handler(req).await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "testfile.txt");
    Ok(())
}

#[actix_web::test]
async fn test_csv_upload() -> actix_web::Result<()> {
    let req = get_request_to("/tests/upload_csv_test.sql")
        .await?
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"people_file\"; filename=\"people.csv\"\r\n\
            Content-Type: text/csv\r\n\
            \r\n\
            name,age\r\n\
            Ophir,29\r\n\
            Max,99\r\n\
            --1234567890--\r\n",
        )
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("Ophir is 29 years old"),
        "{body_str}\nexpected to contain: Ophir is 29 years old"
    );
    Ok(())
}

#[actix_web::test]
async fn test_transaction_error() -> actix_web::Result<()> {
    // First, request the page without any parameter. It should fail because
    // of the not null constraint.
    // But then, when we request again with a parameter, we should not see any side
    // effect coming from the first transaction, and it should succeed
    let data = make_app_data().await;
    let path = match data.db.to_string().to_lowercase().as_str() {
        "mysql" => "/tests/failed_transaction_mysql.sql",
        _ => "/tests/failed_transaction.sql",
    };
    let req = get_request_to_with_data(path, data.clone())
        .await?
        .to_srv_request();
    let resp = main_handler(req).await?;
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec())
        .unwrap()
        .to_ascii_lowercase();
    assert!(
        body_str.contains("error") && body_str.contains("null"),
        "{body_str}\nexpected to contain: constraint failed"
    );
    // Now query again, with ?x=1447
    let path_with_param = path.to_string() + "?x=1447";
    let req = get_request_to_with_data(&path_with_param, data.clone())
        .await?
        .to_srv_request();
    let resp = main_handler(req).await?;
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("1447"),
        "{body_str}\nexpected to contain: 1447"
    );
    Ok(())
}

#[actix_web::test]
/// `/sqlpage/migrations/0001_init.sql` should return a 403 Forbidden
async fn privileged_paths_are_not_accessible() {
    let resp_result = req_path("/sqlpage/migrations/0001_init.sql").await;
    assert!(
        resp_result.is_err(),
        "Accessing a migration file should be forbidden"
    );
    let resp = resp_result.unwrap_err().error_response();
    assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    assert!(
        String::from_utf8_lossy(&resp.into_body().try_into_bytes().unwrap())
            .to_lowercase()
            .contains("forbidden"),
    );
}

#[actix_web::test]
async fn test_static_files() {
    let resp = req_path("/tests/it_works.txt").await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = test::read_body(resp).await;
    assert_eq!(&body, &b"It works !"[..]);
}

#[actix_web::test]
async fn test_with_site_prefix() {
    let mut config = test_config();
    config.site_prefix = "/xxx/".to_string();
    let state = AppState::init(&config).await.unwrap();
    let app_data = actix_web::web::Data::new(state);
    let resp = req_path_with_app_data("/xxx/tests/sql_test_files/it_works_simple.sql", app_data)
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("It works !"),
        "{body_str}\nexpected to contain: It works !"
    );
    assert!(
        body_str.contains("href=\"/xxx/"),
        "{body_str}\nexpected to contain stylesheet link with site prefix"
    );
}

async fn make_app_data_for_official_website() -> actix_web::web::Data<AppState> {
    init_log();
    let config_path = std::path::Path::new("examples/official-site/sqlpage");
    let mut app_config = sqlpage::app_config::load_from_directory(config_path).unwrap();
    app_config.web_root = PathBuf::from("examples/official-site");
    app_config.database_url = "sqlite::memory:".to_string(); // the official site supports only sqlite. Ignore the DATABASE_URL env var
    let app_state = make_app_data_from_config(app_config.clone()).await;
    webserver::database::migrations::apply(&app_config, &app_state.db)
        .await
        .unwrap();
    app_state
}

#[actix_web::test]
async fn test_official_website_documentation() {
    let app_data = make_app_data_for_official_website().await;
    let resp = req_path_with_app_data("/component.sql?component=button", app_data)
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains(r#"<button type="submit" form="poem" formaction="?action"#),
        "{body_str}\nexpected to contain a button with formaction"
    );
}

#[actix_web::test]
async fn test_official_website_basic_auth_example() {
    let resp = req_path_with_app_data(
        "/examples/authentication/basic_auth.sql",
        make_app_data_for_official_website().await,
    )
    .await
    .unwrap();
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("not authorized"),
        "{body_str}\nexpected to contain Unauthorized"
    );
}

async fn get_request_to_with_data(
    path: &str,
    data: actix_web::web::Data<AppState>,
) -> actix_web::Result<TestRequest> {
    Ok(test::TestRequest::get()
        .uri(path)
        .insert_header(ContentType::plaintext())
        .app_data(payload_config(&data))
        .app_data(form_config(&data))
        .app_data(data))
}

async fn get_request_to(path: &str) -> actix_web::Result<TestRequest> {
    let data = make_app_data().await;
    get_request_to_with_data(path, data).await
}

async fn make_app_data_from_config(config: AppConfig) -> actix_web::web::Data<AppState> {
    let state = AppState::init(&config).await.unwrap();
    actix_web::web::Data::new(state)
}

async fn make_app_data() -> actix_web::web::Data<AppState> {
    init_log();
    let config = test_config();
    make_app_data_from_config(config).await
}

async fn req_path(
    path: impl AsRef<str>,
) -> Result<actix_web::dev::ServiceResponse, actix_web::Error> {
    let req = get_request_to(path.as_ref()).await?.to_srv_request();
    main_handler(req).await
}

async fn srv_req_path_with_app_data(
    path: impl AsRef<str>,
    app_data: actix_web::web::Data<AppState>,
) -> actix_web::dev::ServiceRequest {
    test::TestRequest::get()
        .uri(path.as_ref())
        .app_data(app_data)
        .insert_header(("cookie", "test_cook=123"))
        .insert_header(("authorization", "Basic dGVzdDp0ZXN0")) // test:test
        .to_srv_request()
}

async fn req_path_with_app_data(
    path: impl AsRef<str>,
    app_data: actix_web::web::Data<AppState>,
) -> Result<actix_web::dev::ServiceResponse, actix_web::Error> {
    let req = srv_req_path_with_app_data(path, app_data).await;
    main_handler(req).await
}

pub fn test_config() -> AppConfig {
    let db_url = test_database_url();
    serde_json::from_str::<AppConfig>(&format!(
        r#"{{
        "database_url": "{}",
        "max_database_pool_connections": 1,
        "database_connection_retries": 3,
        "database_connection_acquire_timeout_seconds": 15,
        "allow_exec": true,
        "max_uploaded_file_size": 123456,
        "listen_on": "111.111.111.111:1",
        "system_root_ca_certificates" : false
    }}"#,
        db_url
    ))
    .unwrap()
}

fn init_log() {
    let _ = env_logger::builder()
        .parse_default_env()
        .is_test(true)
        .try_init();
}
