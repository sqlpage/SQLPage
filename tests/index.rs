use actix_web::{
    body::MessageBody,
    dev::{fn_service, ServerHandle, ServiceRequest, ServiceResponse},
    http::{self, header::ContentType, StatusCode},
    test::{self, TestRequest},
    HttpResponse,
};
use sqlpage::{app_config::AppConfig, webserver::http::main_handler, AppState};

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
        let req_str = format!("/{}?x=1", test_file_path_string);
        let resp = req_path_with_app_data(&req_str, app_data.clone())
            .await
            .unwrap();
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
            let expected_str = rest.replace('_', " ");
            assert!(
                lowercase_body.contains(&expected_str),
                "{req_str}\n{body}\nexpected to contain: {expected_str}"
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
async fn test_file_upload() -> actix_web::Result<()> {
    let req = get_request_to("/tests/upload_file_test.sql")
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
async fn test_file_upload_too_large() -> actix_web::Result<()> {
    // Files larger than 12345 bytes should be rejected as per the test_config
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
                + "a".repeat(12346).as_str()
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

async fn get_request_to(path: &str) -> actix_web::Result<TestRequest> {
    let data = make_app_data().await;
    Ok(test::TestRequest::get().uri(path).app_data(data))
}

async fn make_app_data() -> actix_web::web::Data<AppState> {
    init_log();
    let config = test_config();
    let state = AppState::init(&config).await.unwrap();

    actix_web::web::Data::new(state)
}

async fn req_path(
    path: impl AsRef<str>,
) -> Result<actix_web::dev::ServiceResponse, actix_web::Error> {
    let req = get_request_to(path.as_ref())
        .await?
        .insert_header(ContentType::plaintext())
        .to_srv_request();
    main_handler(req).await
}

async fn req_path_with_app_data(
    path: impl AsRef<str>,
    app_data: actix_web::web::Data<AppState>,
) -> Result<actix_web::dev::ServiceResponse, actix_web::Error> {
    let req = test::TestRequest::get()
        .uri(path.as_ref())
        .app_data(app_data)
        .to_srv_request();
    main_handler(req).await
}

pub fn test_config() -> AppConfig {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());
    serde_json::from_str::<AppConfig>(&format!(
        r#"{{
        "database_url": "{}",
        "database_connection_retries": 2,
        "database_connection_acquire_timeout_seconds": 10,
        "allow_exec": true,
        "max_uploaded_file_size": 12345,
        "listen_on": "111.111.111.111:1"
    }}"#,
        db_url
    ))
    .unwrap()
}

fn init_log() {
    let _ = env_logger::builder().is_test(true).try_init();
}
