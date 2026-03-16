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
async fn test_persist_uploaded_file_mode() -> actix_web::Result<()> {
    let app_data = crate::common::make_app_data().await;
    let req = actix_web::test::TestRequest::get()
        .uri("/tests/uploads/persist_with_mode.sql?mode=644")
        .app_data(app_data.clone())
        .app_data(sqlpage::webserver::http::payload_config(&app_data))
        .app_data(sqlpage::webserver::http::form_config(&app_data))
        .insert_header((actix_web::http::header::ACCEPT, "application/json"))
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"my_file\"; filename=\"test.txt\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            Hello\r\n\
            --1234567890--\r\n",
        )
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body_json: serde_json::Value = test::read_body_json(resp).await;
    let persisted_path = body_json[0]["contents"]
        .as_str()
        .expect("Path not found in JSON response");

    // The path is relative to web root, we need to find it on disk.
    // In tests, web root is the repo root.
    // We normalize the path separators to be platform-specific for the file system check.
    let normalized_path = persisted_path
        .replace('\\', "/")
        .trim_start_matches('/')
        .replace('/', std::path::MAIN_SEPARATOR_STR);
    let file_path = std::path::Path::new(&normalized_path);
    assert!(
        file_path.exists(),
        "Persisted file {} does not exist. Persisted path: {}, JSON: {}",
        file_path.display(),
        persisted_path,
        body_json
    );
    let contents = std::fs::read_to_string(file_path)?;
    assert_eq!(contents, "Hello");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(file_path)?;
        assert_eq!(metadata.permissions().mode() & 0o777, 0o644);
    }

    std::fs::remove_file(file_path)?;
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
    let msg = "max file size";
    assert!(
        err_str.to_ascii_lowercase().contains(msg),
        "{err_str}\nexpected to contain: {msg}"
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
