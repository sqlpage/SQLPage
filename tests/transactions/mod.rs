use actix_web::{http::StatusCode, test};
use sqlpage::webserver::http::main_handler;

use crate::common::{get_request_to_with_data, make_app_data};

#[actix_web::test]
async fn test_transaction_error() -> actix_web::Result<()> {
    let data = make_app_data().await;
    let path = match data.db.to_string().to_lowercase().as_str() {
        "mysql" => "/tests/transactions/failed_transaction_mysql.sql",
        "mssql" => "/tests/transactions/failed_transaction_mssql.sql",
        _ => "/tests/transactions/failed_transaction.sql",
    };
    let req = get_request_to_with_data(path, data.clone())
        .await?
        .to_srv_request();
    let resp = main_handler(req).await?;
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec())
        .unwrap()
        .to_ascii_lowercase();
    assert!(
        body_str.contains("error") && body_str.contains("null"),
        "{body_str}\nexpected to contain: constraint failed"
    );
    // Now query again, with ?x=1447
    let path_with_param = path.to_string() + "?x=1447";
    let req = get_request_to_with_data(&path_with_param, data.clone())
        .await?
        .to_srv_request();
    let resp = main_handler(req).await?;
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("1447"),
        "{body_str}\nexpected to contain: 1447"
    );
    Ok(())
}

#[actix_web::test]
async fn test_failed_copy_followed_by_query() -> actix_web::Result<()> {
    let app_data = make_app_data().await;
    let big_csv = "col1,col2\nval1,val2\n".repeat(1000);
    let req = get_request_to_with_data(
        "/tests/sql_test_files/error_failed_to_import_the_csv.sql",
        app_data.clone(),
    )
    .await?
        .insert_header(("content-type", "multipart/form-data; boundary=1234567890"))
        .set_payload(format!(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"recon_csv_file_input\"; filename=\"data.csv\"\r\n\
            Content-Type: text/csv\r\n\
            \r\n\
            {big_csv}\r\n\
            --1234567890--\r\n"
        ))
        .to_srv_request();
    let resp = main_handler(req).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("error"),
        "{body_str}\nexpected to contain error message"
    );

    // On postgres, the error message should contain  "The postgres COPY FROM STDIN command failed"
    if matches!(app_data.db.to_string().to_lowercase().as_str(), "postgres") {
        assert!(
            body_str.contains("The postgres COPY FROM STDIN command failed"),
            "{body_str}\nexpected to contain: The postgres COPY FROM STDIN command failed"
        );
    }
    // Now make other requests to verify the connection is still usable
    for path in [
        "/tests/sql_test_files/it_works_lower.sql",
        "/tests/sql_test_files/it_works_simple.sql",
        "/tests/sql_test_files/it_works_path.sql",
    ] {
        let req = get_request_to_with_data(path, app_data.clone())
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
    }
    Ok(())
}
