use actix_web::{http::StatusCode, test};
use sqlpage::webserver::http::main_handler;

use crate::common::get_request_to;

async fn test_file_upload(target: &str) -> actix_web::Result<()> {
    let req = get_request_to(target)
        .await?
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"my_file\"; filename=\"testfile.txt\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            Hello, world!\r\n\
            --1234567890--\r\n",
        )
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("Hello, world!"),
        "{body_str}\nexpected to contain: Hello, world!"
    );
    Ok(())
}

#[actix_web::test]
async fn test_file_upload_direct() -> actix_web::Result<()> {
    test_file_upload("/tests/uploads/upload_file_test.sql").await
}

#[actix_web::test]
async fn test_file_upload_through_runsql() -> actix_web::Result<()> {
    test_file_upload("/tests/uploads/upload_file_runsql_test.sql").await
}

#[actix_web::test]
async fn test_blank_file_upload_field() -> actix_web::Result<()> {
    let req = get_request_to("/tests/uploads/upload_file_test.sql")
        .await?
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"my_file\"; filename=\"\"\r\n\
            Content-Type: application/octet-stream\r\n\
            \r\n\
            \r\n\
            --1234567890--\r\n",
        )
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("No file uploaded"),
        "{body_str}\nexpected to contain: No file uploaded"
    );
    Ok(())
}

#[actix_web::test]
async fn test_file_upload_too_large() -> actix_web::Result<()> {
    let req = get_request_to("/tests/uploads/upload_file_test.sql")
        .await?
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"my_file\"; filename=\"testfile.txt\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            "
            .to_string()
                + "a".repeat(123457).as_str()
                + "\r\n\
            --1234567890--\r\n",
        )
        .to_srv_request();
    let err_str = main_handler(req)
        .await
        .expect_err("Expected an error response")
        .to_string();
    assert!(
        err_str.to_ascii_lowercase().contains("max file size"),
        "{err_str}\nexpected to contain: File too large"
    );
    Ok(())
}

#[actix_web::test]
async fn test_upload_file_data_url() -> actix_web::Result<()> {
    let req = get_request_to("/tests/uploads/upload_file_data_url_test.sql")
        .await?
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"my_file\"; filename=\"testfile.txt\"\r\n\
            Content-Type: image/svg+xml\r\n\
            \r\n\
            <svg></svg>\r\n\
            --1234567890--\r\n",
        )
        .to_srv_request();
    let resp = main_handler(req).await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=");
    Ok(())
}

#[actix_web::test]
async fn test_uploaded_file_name() -> actix_web::Result<()> {
    let req = get_request_to("/tests/uploads/uploaded_file_name_test.sql")
        .await?
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"my_file\"; filename=\"testfile.txt\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            Some plain text.\r\n\
            --1234567890--\r\n",
        )
        .to_srv_request();
    let resp = main_handler(req).await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "testfile.txt");
    Ok(())
}

#[actix_web::test]
async fn test_csv_upload() -> actix_web::Result<()> {
    let req = get_request_to("/tests/uploads/upload_csv_test.sql")
        .await?
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"people_file\"; filename=\"people.csv\"\r\n\
            Content-Type: text/csv\r\n\
            \r\n\
            name,age\r\n\
            Ophir,29\r\n\
            Max,99\r\n\
            --1234567890--\r\n",
        )
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("Ophir is 29 years old"),
        "{body_str}\nexpected to contain: Ophir is 29 years old"
    );
    Ok(())
}
