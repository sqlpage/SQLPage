use actix_web::{http::StatusCode, test};
use sqlpage::webserver::http::main_handler;

use crate::common::get_request_to;

#[actix_web::test]
async fn test_overwrite_variable() -> actix_web::Result<()> {
    let req = get_request_to("/tests/sql_test_files/it_works_set_variable_nomssql.sql")
        .await?
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
