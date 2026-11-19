use actix_web::{http::StatusCode, test};

use crate::common::{multipart_request, response_from};

async fn test_file_upload(target: &str) -> actix_web::Result<()> {
    let resp = response_from(multipart_request(
        target,
        "--1234567890\r\n\
            Content-Disposition: form-data; name=\"my_file\"; filename=\"testfile.txt\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            Hello, world!\r\n\
            --1234567890--\r\n",
    ))
    .await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body_str = crate::common::read_body_string(resp).await;
    assert!(
        body_str.contains("Hello, world!"),
        "{body_str}\nexpected to contain: Hello, world!"
    );
    Ok(())
}

#[actix_web::test]
async fn test_persist_uploaded_file_mode() -> actix_web::Result<()> {
    let resp = response_from(
        multipart_request(
            "/tests/uploads/persist_with_mode.sql?mode=644",
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"my_file\"; filename=\"test.txt\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            Hello\r\n\
            --1234567890--\r\n",
        )
        .insert_header(("accept", "application/json")),
    )
    .await?;

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
    let resp = response_from(multipart_request(
        "/tests/uploads/upload_file_test.sql",
        "--1234567890\r\n\
        Content-Disposition: form-data; name=\"my_file\"; filename=\"\"\r\n\
        Content-Type: application/octet-stream\r\n\
        \r\n\
        \r\n\
        --1234567890--\r\n",
    ))
    .await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body_str = crate::common::read_body_string(resp).await;
    assert!(
        body_str.contains("No file uploaded"),
        "{body_str}\nexpected to contain: No file uploaded"
    );
    Ok(())
}

#[actix_web::test]
async fn test_file_upload_too_large() -> actix_web::Result<()> {
    let err_str = response_from(multipart_request(
        "/tests/uploads/upload_file_test.sql",
        "--1234567890\r\n\
        Content-Disposition: form-data; name=\"my_file\"; filename=\"testfile.txt\"\r\n\
        Content-Type: text/plain\r\n\
        \r\n\
        "
        .to_string()
            + "a".repeat(123_457).as_str()
            + "\r\n\
        --1234567890--\r\n",
    ))
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
    let resp = response_from(multipart_request(
        "/tests/uploads/upload_file_data_url_test.sql",
        "--1234567890\r\n\
        Content-Disposition: form-data; name=\"my_file\"; filename=\"testfile.txt\"\r\n\
        Content-Type: image/svg+xml\r\n\
        \r\n\
        <svg></svg>\r\n\
        --1234567890--\r\n",
    ))
    .await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let body_str = crate::common::read_body_string(resp).await;
    assert_eq!(body_str, "data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=");
    Ok(())
}

#[actix_web::test]
async fn test_uploaded_file_name() -> actix_web::Result<()> {
    let resp = response_from(multipart_request(
        "/tests/uploads/uploaded_file_name_test.sql",
        "--1234567890\r\n\
        Content-Disposition: form-data; name=\"my_file\"; filename=\"testfile.txt\"\r\n\
        Content-Type: text/plain\r\n\
        \r\n\
        Some plain text.\r\n\
        --1234567890--\r\n",
    ))
    .await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let body_str = crate::common::read_body_string(resp).await;
    assert_eq!(body_str, "testfile.txt");
    Ok(())
}

#[actix_web::test]
async fn test_csv_upload() -> actix_web::Result<()> {
    let resp = response_from(multipart_request(
        "/tests/uploads/upload_csv_test.sql",
        "--1234567890\r\n\
        Content-Disposition: form-data; name=\"people_file\"; filename=\"people.csv\"\r\n\
        Content-Type: text/csv\r\n\
        \r\n\
        name,age\r\n\
        Ophir,29\r\n\
        Max,99\r\n\
        --1234567890--\r\n",
    ))
    .await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body_str = crate::common::read_body_string(resp).await;
    assert!(
        body_str.contains("Ophir is 29 years old"),
        "{body_str}\nexpected to contain: Ophir is 29 years old"
    );
    Ok(())
}

fn two_files_and_one(third_file_name: &str) -> String {
    format!(
        "--1234567890\r\n\
        Content-Disposition: form-data; name=\"photos[]\"; filename=\"a.txt\"\r\n\
        Content-Type: text/plain\r\n\
        \r\n\
        A\r\n\
        --1234567890\r\n\
        Content-Disposition: form-data; name=\"photos[]\"; filename=\"b.txt\"\r\n\
        Content-Type: text/plain\r\n\
        \r\n\
        B\r\n\
        --1234567890\r\n\
        Content-Disposition: form-data; name=\"photos[]\"; filename=\"{third_file_name}\"\r\n\
        Content-Type: text/plain\r\n\
        \r\n\
        C\r\n\
        --1234567890--\r\n"
    )
}

#[actix_web::test]
async fn test_persist_uploaded_files() -> actix_web::Result<()> {
    let resp = response_from(multipart_request(
        "/tests/uploads/persist_multiple.sql",
        two_files_and_one("c.txt"),
    ))
    .await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let files: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert_eq!(files.len(), 3);
    let mut contents = Vec::new();
    for (file, name) in files.iter().zip(["a.txt", "b.txt", "c.txt"]) {
        assert_eq!(file["name"], name);
        assert_eq!(file["mime_type"], "text/plain");
        assert_eq!(file["size"], 1);
        let path = file["path"].as_str().expect("persisted path");
        let file_path = std::path::Path::new(path.trim_start_matches('/'));
        contents.push(std::fs::read_to_string(file_path)?);
        std::fs::remove_file(file_path)?;
    }
    assert_eq!(contents, ["A", "B", "C"]);
    Ok(())
}

#[actix_web::test]
async fn test_persist_uploaded_files_rejects_all_on_bad_extension() -> actix_web::Result<()> {
    // Dedicated folder: tests run in parallel, and another test persists files in persisted_multiple.
    let folder = std::path::Path::new("tests/uploads/persisted_multiple_rejected");
    let count_files = || std::fs::read_dir(folder).map_or(0, Iterator::count);
    let before = count_files();
    let resp = response_from(multipart_request(
        "/tests/uploads/persist_multiple_rejected.sql",
        two_files_and_one("c.sql"),
    ))
    .await?;
    let body = crate::common::read_body_string(resp).await;
    assert!(body.contains("file extension sql is not allowed"), "{body}");
    assert_eq!(
        count_files(),
        before,
        "no file must be written when one is rejected"
    );
    Ok(())
}

#[actix_web::test]
async fn test_persist_uploaded_files_without_files() -> actix_web::Result<()> {
    let resp = response_from(multipart_request(
        "/tests/uploads/persist_multiple.sql",
        "--1234567890\r\n\
        Content-Disposition: form-data; name=\"x\"\r\n\
        \r\n\
        1\r\n\
        --1234567890--\r\n",
    ))
    .await?;
    let body = crate::common::read_body_string(resp).await;
    assert_eq!(body.trim(), "null");
    Ok(())
}
