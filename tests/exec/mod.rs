use actix_web::{http::header, test::TestRequest};
use sqlpage::webserver::http::main_handler;

#[actix_web::rt::test(system = "crate::common::TestSystem")]
async fn test_exec() {
    let app_data = crate::common::make_app_data().await;
    let req = TestRequest::get()
        .uri(exec_test_uri())
        .app_data(app_data)
        .insert_header(header::Accept::json())
        .to_srv_request();

    let resp = main_handler(req).await.unwrap();
    let body = actix_web::test::read_body(resp).await;
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    let actual = rows[0]["actual"].as_str().unwrap();

    assert!(actual.contains("It works !"), "actual: {actual:?}");
}

#[cfg(windows)]
fn exec_test_uri() -> &'static str {
    "/tests/exec/exec.sql?exec_program=cmd.exe&exec_arg1=/C&exec_arg2=echo"
}

#[cfg(not(windows))]
fn exec_test_uri() -> &'static str {
    "/tests/exec/exec.sql?exec_program=echo"
}
