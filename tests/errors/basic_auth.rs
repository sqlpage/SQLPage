use crate::common::{request_for, response_for, response_from};
use actix_web::http::StatusCode;

#[actix_web::test]
async fn test_basic_auth_not_provided() {
    let resp = response_for("/tests/errors/basic_auth.sql").await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        resp.headers().get("www-authenticate").unwrap(),
        "Basic realm=\"Authentication required\", charset=\"UTF-8\""
    );
    let body_str = crate::common::read_body_string(resp).await;
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
    let req = request_for("/tests/errors/basic_auth.sql") // log in with credentials "user:password"
        .append_header(("Authorization", "Basic dXNlcjpwYXNzd29yZA=="));
    let resp = response_from(req)
        .await
        .expect("req with credentials should succeed");
    assert_eq!(resp.status(), StatusCode::OK);
    let body_str = crate::common::read_body_string(resp).await;
    assert!(
        body_str.contains("Success!"),
        "{body_str}\nexpected to contain Success"
    );
}
