use std::time::Duration;

use actix_web::{
    http::header::ContentType,
    test::{self, TestRequest},
    web::Data,
};
use sqlpage::{
    app_config::{test_database_url, AppConfig},
    webserver::http::{form_config, main_handler, payload_config},
    AppState,
};

pub async fn get_request_to_with_data(
    path: &str,
    data: Data<AppState>,
) -> actix_web::Result<TestRequest> {
    Ok(test::TestRequest::get()
        .uri(path)
        .insert_header(ContentType::plaintext())
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
