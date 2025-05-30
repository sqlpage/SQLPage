use std::time::Duration;

use actix_web::{
    dev::{fn_service, ServiceRequest},
    http::header,
    http::header::ContentType,
    test::{self, TestRequest},
    web,
    web::Data,
    App, HttpResponse, HttpServer,
};
use sqlpage::{
    app_config::{test_database_url, AppConfig},
    webserver::http::{form_config, main_handler, payload_config},
    AppState,
};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub async fn get_request_to_with_data(
    path: &str,
    data: Data<AppState>,
) -> actix_web::Result<TestRequest> {
    Ok(test::TestRequest::get()
        .uri(path)
        .insert_header(ContentType::plaintext())
        .insert_header(header::Accept::html())
        .app_data(payload_config(&data))
        .app_data(form_config(&data))
        .app_data(data))
}

pub async fn get_request_to(path: &str) -> actix_web::Result<TestRequest> {
    let data = make_app_data().await;
    get_request_to_with_data(path, data).await
}

pub async fn make_app_data_from_config(config: AppConfig) -> Data<AppState> {
    let state = AppState::init(&config).await.unwrap();
    Data::new(state)
}

pub async fn make_app_data() -> Data<AppState> {
    init_log();
    let config = test_config();
    make_app_data_from_config(config).await
}

pub async fn req_path(
    path: impl AsRef<str>,
) -> Result<actix_web::dev::ServiceResponse, actix_web::Error> {
    let req = get_request_to(path.as_ref()).await?.to_srv_request();
    main_handler(req).await
}

pub async fn srv_req_path_with_app_data(
    path: impl AsRef<str>,
    app_data: Data<AppState>,
) -> actix_web::dev::ServiceRequest {
    test::TestRequest::get()
        .uri(path.as_ref())
        .app_data(app_data)
        .insert_header(("cookie", "test_cook=123"))
        .insert_header(("authorization", "Basic dGVzdDp0ZXN0")) // test:test
        .to_srv_request()
}

const REQ_TIMEOUT: Duration = Duration::from_secs(8);
pub async fn req_path_with_app_data(
    path: impl AsRef<str>,
    app_data: Data<AppState>,
) -> anyhow::Result<actix_web::dev::ServiceResponse> {
    let path = path.as_ref();
    let req = srv_req_path_with_app_data(path, app_data).await;
    let resp = tokio::time::timeout(REQ_TIMEOUT, main_handler(req))
        .await
        .map_err(|e| anyhow::anyhow!("Request to {path} timed out: {e}"))?
        .map_err(|e| {
            anyhow::anyhow!(
                "Request to {path} failed with status {}: {e:#}",
                e.as_response_error().status_code()
            )
        })?;
    Ok(resp)
}

pub fn test_config() -> AppConfig {
    let db_url = test_database_url();
    serde_json::from_str::<AppConfig>(&format!(
        r#"{{
        "database_url": "{db_url}",
        "max_database_pool_connections": 1,
        "database_connection_retries": 3,
        "database_connection_acquire_timeout_seconds": 15,
        "allow_exec": true,
        "max_uploaded_file_size": 123456,
        "listen_on": "111.111.111.111:1",
        "system_root_ca_certificates" : false
    }}"#
    ))
    .unwrap()
}

pub fn init_log() {
    let _ = env_logger::builder()
        .parse_default_env()
        .is_test(true)
        .try_init();
}

fn format_request_line_and_headers(req: &ServiceRequest) -> String {
    let mut out = format!("{} {}", req.method(), req.uri());
    let mut headers: Vec<_> = req.headers().iter().collect();
    headers.sort_by_key(|(k, _)| k.as_str());
    for (k, v) in headers {
        if k.as_str().eq_ignore_ascii_case("date") {
            continue;
        }
        out.push_str(&format!("|{k}: {}", v.to_str().unwrap_or("?")));
    }
    out
}

async fn format_body(req: &mut ServiceRequest) -> Vec<u8> {
    req.extract::<web::Bytes>()
        .await
        .map(|b| b.to_vec())
        .unwrap_or_default()
}

fn build_echo_response(body: Vec<u8>, meta: String) -> HttpResponse {
    let mut resp = meta.into_bytes();
    resp.push(b'|');
    resp.extend_from_slice(&body);
    HttpResponse::Ok()
        .insert_header((header::DATE, "Mon, 24 Feb 2025 12:00:00 GMT"))
        .insert_header((header::CONTENT_TYPE, "text/plain"))
        .body(resp)
}

pub fn start_echo_server(shutdown: oneshot::Receiver<()>) -> (JoinHandle<()>, u16) {
    let listener = std::net::TcpListener::bind("localhost:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = HttpServer::new(|| {
        App::new().default_service(fn_service(|mut req: ServiceRequest| async move {
            let meta = format_request_line_and_headers(&req);
            let body = format_body(&mut req).await;
            let resp = build_echo_response(body, meta);
            Ok(req.into_response(resp))
        }))
    })
    .listen(listener)
    .unwrap()
    .shutdown_timeout(1)
    .run();
    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = server => {},
            _ = shutdown => {},
        }
    });
    (handle, port)
}
