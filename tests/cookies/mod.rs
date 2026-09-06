use crate::common::req_path;
use actix_web::http::header::SET_COOKIE;

async fn set_cookie_header(path: &str) -> String {
    let resp = req_path(path).await.unwrap();
    resp.headers()
        .get(SET_COOKIE)
        .unwrap_or_else(|| panic!("{path} should have sent a Set-Cookie header"))
        .to_str()
        .unwrap()
        .to_owned()
}

#[actix_web::test]
async fn cookies_are_http_only_secure_and_same_site_strict_by_default() {
    let header = set_cookie_header("/tests/cookies/set_cookie_defaults.sql").await;
    assert!(header.starts_with("session=abc123"), "{header}");
    assert!(header.contains("HttpOnly"), "{header}");
    assert!(header.contains("Secure"), "{header}");
    assert!(header.contains("SameSite=Strict"), "{header}");
    assert!(header.contains("Path=/"), "{header}");
}

#[actix_web::test]
async fn zero_turns_off_a_cookie_protection() {
    let header = set_cookie_header("/tests/cookies/set_cookie_opt_out.sql").await;
    assert!(!header.contains("HttpOnly"), "{header}");
    assert!(!header.contains("Secure"), "{header}");
    assert!(header.contains("SameSite=Lax"), "{header}");
    assert!(header.contains("Path=/admin"), "{header}");
}
