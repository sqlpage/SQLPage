use std::any::Any;

use actix_web::http::StatusCode;
use sqlpage::webserver::database::{DbInfo, SupportedDatabase};
use sqlx::{Connection, database::HasStatementCache};

use crate::common::{make_app_data, response_with_data};

fn cache_size<C: Connection + 'static>(connection: &dyn Any) -> Option<usize>
where
    C::Database: HasStatementCache,
{
    connection
        .downcast_ref::<C>()
        .map(C::cached_statements_size)
}

#[actix_web::rt::test(system = "crate::test_database::System")]
async fn test_parameterized_pages_leave_a_prepared_statement_in_the_cache() -> actix_web::Result<()>
{
    let data = make_app_data().await;
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

    if data.db.info == DbInfo::Native(SupportedDatabase::Mssql) {
        return Ok(()); // The native SQL Server driver has no statement cache.
    }
    let mut connection = data.db.connection.acquire().await.unwrap();
    let native = connection.native();
    let cached_statements = cache_size::<sqlx::SqliteConnection>(native)
        .or_else(|| cache_size::<sqlx::PgConnection>(native))
        .or_else(|| cache_size::<sqlx::MySqlConnection>(native))
        .or_else(|| cache_size::<sqlx_odbc::OdbcConnection>(native))
        .expect("this driver supports statement caching");
    assert!(
        cached_statements > 0,
        "{:?} ran a parameterized query three times without caching a prepared statement",
        data.db.info
    );
    Ok(())
}
