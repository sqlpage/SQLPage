use actix_web::http::StatusCode;
use sqlpage::webserver::database::SupportedDatabase;
use sqlx::row::Row as _;

use crate::common::{make_app_data, multipart_request, response_with_data, send_request};

#[actix_web::test]
async fn test_transaction_error() -> actix_web::Result<()> {
    let data = make_app_data().await;
    let path = match data.db.info.database_type {
        SupportedDatabase::MySql => "/tests/transactions/failed_transaction_mysql.sql",
        SupportedDatabase::Mssql => "/tests/transactions/failed_transaction_mssql.sql",
        SupportedDatabase::Snowflake | SupportedDatabase::Oracle => {
            return Ok(()); //snowflake and oracle don't support transactions in this test way
        }
        _ => "/tests/transactions/failed_transaction.sql",
    };
    let resp = response_with_data(path, data.clone()).await?;
    let body_str = crate::common::read_body_string(resp)
        .await
        .to_ascii_lowercase();
    assert!(
        body_str.contains("error") && body_str.contains("null"),
        "{body_str}\nexpected to contain: constraint failed"
    );
    if data.db.info.database_type == SupportedDatabase::Sqlite {
        let resp = response_with_data(
            "/tests/transactions/failed_computed_column.sql",
            data.clone(),
        )
        .await?;
        let body_str = crate::common::read_body_string(resp).await;
        assert!(body_str.contains("invalid URL"));
        let row = sqlx::query::query("SELECT COUNT(*) AS count FROM query_event_rollback")
            .fetch_one(&data.db.connection)
            .await
            .unwrap();
        assert_eq!(row.try_get::<i64, _>("count").unwrap(), 0);
    }
    // Now query again, with ?x=1447
    let path_with_param = path.to_string() + "?x=1447";
    let resp = response_with_data(&path_with_param, data.clone()).await?;
    let body_str = crate::common::read_body_string(resp).await;
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
    let req = multipart_request(
        "/tests/sql_test_files/component_rendering/error_failed_to_import_the_csv.sql",
        format!(
            "--1234567890\r\n\
            Content-Disposition: form-data; name=\"recon_csv_file_input\"; filename=\"data.csv\"\r\n\
            Content-Type: text/csv\r\n\
            \r\n\
            {big_csv}\r\n\
            --1234567890--\r\n"
        ),
    );
    let resp = send_request(req, app_data.clone()).await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body_str = crate::common::read_body_string(resp).await;
    assert!(
        body_str.contains("error"),
        "{body_str}\nexpected to contain error message"
    );

    // On postgres, the error message should contain  "The postgres COPY FROM STDIN command failed"
    if crate::common::supports_database(&app_data.db, &[SupportedDatabase::Postgres]) {
        assert!(
            body_str.contains("The postgres COPY FROM STDIN command failed"),
            "{body_str}\nexpected to contain: The postgres COPY FROM STDIN command failed"
        );
    }
    // Now make other requests to verify the connection is still usable
    for path in [
        "/tests/sql_test_files/component_rendering/simple.sql",
        "/tests/sql_test_files/component_rendering/text_markdown.sql",
        "/tests/sql_test_files/component_rendering/text_unsafe_markdown.sql",
    ] {
        let resp = response_with_data(path, app_data.clone()).await?;

        assert_eq!(resp.status(), StatusCode::OK);
        let body_str = crate::common::read_body_string(resp).await;
        assert!(
            body_str.contains("It works !"),
            "{body_str}\nexpected to contain: It works !"
        );
    }
    Ok(())
}
