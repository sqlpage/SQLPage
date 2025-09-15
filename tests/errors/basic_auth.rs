use crate::common::{get_request_to, req_path};
use actix_web::{http::StatusCode, test};
use sqlpage::webserver::http::main_handler;

#[actix_web::test]
async fn test_basic_auth_not_provided() {
    let resp_result = req_path("/tests/errors/basic_auth.sql").await;
    let resp = resp_result.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        resp.headers().get("www-authenticate").unwrap(),
        "Basic realm=\"Authentication required\", charset=\"UTF-8\""
    );
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("Unauthorized"),
        "{body_str}\nexpected to contain Unauthorized"
    );
    assert!(
        !body_str.contains("Success!"),
        "{body_str}\nexpected not to contain Success!"
    );
}

#[actix_web::test]
async fn test_basic_auth_with_credentials() {
    let req = get_request_to("/tests/errors/basic_auth.sql")
        .await
        .unwrap() // log in with credentials "user:password"
        .append_header(("Authorization", "Basic dXNlcjpwYXNzd29yZA=="))
        .to_srv_request();
    let resp = main_handler(req)
        .await
        .expect("req with credentials should succeed");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("Success!"),
        "{body_str}\nexpected to contain Success"
    );
}
