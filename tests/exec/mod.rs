use actix_web::http::header;

#[actix_web::test]
async fn test_exec() {
    let resp = crate::common::response_from(
        crate::common::request_for(exec_test_uri()).insert_header(header::Accept::json()),
    )
    .await
    .unwrap();
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
