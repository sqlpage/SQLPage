use actix_web::{
    http::{self, header::ContentType},
    test,
};
use sqlpage::{app_config::AppConfig, webserver::http::main_handler, AppState};

#[actix_web::test]
async fn test_index_ok() {
    init_log();
    let config = test_config();
    let state = AppState::init(&config).await.unwrap();
    let data = actix_web::web::Data::new(state);
    let req = test::TestRequest::get()
        .uri("/")
        .app_data(data)
        .insert_header(ContentType::plaintext())
        .to_srv_request();
    let resp = main_handler(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = test::read_body(resp).await;
    assert!(body.starts_with(b"<!DOCTYPE html>"));
    // the body should contain the strint "It works!" and should not contain the string "error"
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("It works !"));
    assert!(!body.contains("error"));
}

pub fn test_config() -> AppConfig {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());
    serde_json::from_str::<AppConfig>(&format!(
        r#"{{
        "database_url": "{}",
        "database_connection_retries": 2,
        "database_connection_acquire_timeout_seconds": 1,
        "listen_on": "111.111.111.111:1"
    }}"#,
        db_url
    ))
    .unwrap()
}

fn init_log() {
    env_logger::builder().is_test(true).try_init().unwrap();
}
