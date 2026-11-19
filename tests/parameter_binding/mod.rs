use actix_web::http::StatusCode;
use sqlpage::webserver::database::{DatabaseConnection, DatabaseDriver};
use sqlx::Connection as _;

use crate::common::{make_app_data, response_with_data};

#[actix_web::test]
async fn test_parameterized_pages_leave_a_prepared_statement_in_the_cache() -> actix_web::Result<()>
{
    let data = make_app_data().await;
    if data.db.info.kind == DatabaseDriver::Mssql {
        return Ok(()); // the MSSQL backend keeps no statement cache
    }

    for _ in 0..3 {
        let resp = response_with_data(
            "/tests/parameter_binding/echo_parameter.sql?x=1447",
            data.clone(),
        )
        .await?;
        assert_eq!(resp.status(), StatusCode::OK);
        let page = crate::common::read_body_string(resp).await;
        assert!(
            page.contains("1447"),
            "{page}\nexpected the bound parameter to reach the query"
        );
    }

    let connection = data.db.connection.acquire().await.unwrap();
    let cached_statements = match connection {
        DatabaseConnection::Postgres(conn) => conn.cached_statements_size(),
        DatabaseConnection::MySql(conn) => conn.cached_statements_size(),
        DatabaseConnection::Sqlite(conn) => conn.cached_statements_size(),
        DatabaseConnection::Odbc(conn) => conn.cached_statements_size(),
        DatabaseConnection::Mssql(_) => unreachable!("SQL Server has no statement cache"),
    };
    assert!(
        cached_statements > 0,
        "{:?} ran a parameterized query three times without caching a prepared statement",
        data.db.info.kind
    );
    Ok(())
}
