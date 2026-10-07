use std::fmt::Write as _;
use std::time::Duration;

use actix_web::{
    App, HttpResponse, HttpServer,
    dev::{ServiceRequest, ServiceResponse, fn_service},
    http::header,
    http::header::ContentType,
    test::{self, TestRequest},
    web,
    web::Data,
};
use sqlpage::{
    AppState,
    app_config::{AppConfig, test_database_url},
    telemetry,
    webserver::http::{form_config, main_handler, payload_config},
};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

/// Builds a GET request with the defaults used by the SQL fixtures.
pub(crate) fn request_for(path: impl AsRef<str>) -> TestRequest {
    TestRequest::get()
        .uri(path.as_ref())
        .insert_header(ContentType::plaintext())
        .insert_header(header::Accept::html())
}

pub(crate) fn multipart_request(path: &str, body: impl Into<web::Bytes>) -> TestRequest {
    request_for(path)
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(body)
}

pub(crate) async fn make_app_data_from_config(
    config: AppConfig,
) -> actix_web::Result<Data<AppState>> {
    init_log();
    AppState::init(&config)
        .await
        .map(Data::new)
        .map_err(actix_web::error::ErrorInternalServerError)
}

/// Sends a GET request with the default test configuration, failing the test on error.
pub(crate) async fn response_for(path: impl AsRef<str>) -> ServiceResponse {
    response_with(path.as_ref(), test_config())
        .await
        .unwrap_or_else(|err| panic!("Request to {} failed: {err:#}", path.as_ref()))
}

/// Sends a GET request with custom application configuration.
pub(crate) async fn response_with(
    path: impl AsRef<str>,
    config: AppConfig,
) -> actix_web::Result<ServiceResponse> {
    send_request(request_for(path), make_app_data_from_config(config).await?).await
}

/// Sends a custom request with the default test configuration.
pub(crate) async fn response_from(request: TestRequest) -> actix_web::Result<ServiceResponse> {
    send_request(request, make_app_data_from_config(test_config()).await?).await
}

/// Sends a GET request using existing state, for tests that share a database or cache.
pub(crate) async fn response_with_data(
    path: impl AsRef<str>,
    data: Data<AppState>,
) -> actix_web::Result<ServiceResponse> {
    send_request(request_for(path), data).await
}

/// Runs a request through an Actix service so its request pool has an owner.
/// Dropping the service disables and drains the pool, including on error or panic.
/// Standalone `TestRequest::to_srv_request()` would instead leak the application state.
pub(crate) async fn send_request(
    request: TestRequest,
    data: Data<AppState>,
) -> actix_web::Result<ServiceResponse> {
    let app = test::init_service(
        App::new()
            .app_data(payload_config(&data))
            .app_data(form_config(&data))
            .app_data(data)
            .default_service(fn_service(main_handler)),
    )
    .await;
    tokio::time::timeout(
        Duration::from_secs(8),
        test::try_call_service(&app, request.to_request()),
    )
    .await
    .map_err(actix_web::error::ErrorGatewayTimeout)?
}

/// Reads a whole response body as a UTF-8 string, failing the test otherwise.
pub(crate) async fn read_body_string<B>(resp: ServiceResponse<B>) -> String
where
    B: actix_web::body::MessageBody,
{
    String::from_utf8(test::read_body(resp).await.to_vec()).unwrap()
}

/// Whether the database engine is one of `kinds`.
/// Tests that only run on some engines return early otherwise.
pub(crate) fn supports_database(
    db: &sqlpage::webserver::database::Database,
    kinds: &[sqlpage::webserver::database::SupportedDatabase],
) -> bool {
    kinds.contains(&db.info.database_type)
}

pub(crate) async fn make_app_data() -> Data<AppState> {
    make_app_data_from_config(test_config()).await.unwrap()
}

pub(crate) fn test_config() -> AppConfig {
    serde_json::from_value(serde_json::json!({
        "database_url": test_database_url(),
        "max_database_pool_connections": 1,
        "database_connection_retries": 3,
        "database_connection_acquire_timeout_seconds": 15,
        "allow_exec": true,
        "max_uploaded_file_size": 123_456,
        "listen_on": "111.111.111.111:1",
        "system_root_ca_certificates": false
    }))
    .unwrap()
}

pub(crate) fn init_log() {
    telemetry::init_test_logging();
}

fn format_request_line_and_headers(req: &ServiceRequest) -> String {
    let mut out = format!("{} {}", req.method(), req.uri());
    let mut headers: Vec<_> = req.headers().iter().collect();
    headers.sort_by_key(|(k, _)| k.as_str());
    for (k, v) in headers {
        if k.as_str().eq_ignore_ascii_case("date") {
            continue;
        }
        write!(out, "|{k}: {}", v.to_str().unwrap_or("?")).unwrap();
    }
    out
}

async fn format_body(req: &mut ServiceRequest) -> Vec<u8> {
    req.extract::<web::Bytes>()
        .await
        .map(|b| b.to_vec())
        .unwrap_or_default()
}

fn build_echo_response(body: &[u8], meta: String) -> HttpResponse {
    let mut resp = meta.into_bytes();
    resp.push(b'|');
    resp.extend_from_slice(body);
    HttpResponse::Ok()
        .insert_header((header::DATE, "Mon, 24 Feb 2025 12:00:00 GMT"))
        .insert_header((header::CONTENT_TYPE, "text/plain"))
        .body(resp)
}

pub(crate) fn start_echo_server(shutdown: oneshot::Receiver<()>) -> (JoinHandle<()>, u16) {
    let listener = std::net::TcpListener::bind("localhost:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = HttpServer::new(|| {
        App::new()
            .route(
                "/json",
                web::to(|body: web::Bytes| async move {
                    HttpResponse::Ok()
                        .insert_header((header::CONTENT_TYPE, "application/json"))
                        .body(body)
                }),
            )
            .default_service(fn_service(|mut req: ServiceRequest| async move {
                let meta = format_request_line_and_headers(&req);
                let body = format_body(&mut req).await;
                let resp = build_echo_response(&body, meta);
                Ok(req.into_response(resp))
            }))
    })
    .workers(1)
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
