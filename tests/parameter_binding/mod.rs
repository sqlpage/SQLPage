use actix_web::{http::StatusCode, test};
use sqlx::any::AnyKind;
use sqlx::connection::Connection as _;

use sqlpage::webserver::database::SupportedDatabase;
use sqlpage::webserver::http::main_handler;

use crate::common::{get_request_to_with_data, make_app_data};

async fn rendered_page(
    path: &str,
    data: actix_web::web::Data<sqlpage::AppState>,
) -> actix_web::Result<String> {
    let req = get_request_to_with_data(path, data).await?.to_srv_request();
    let resp = main_handler(req).await?;
    assert_eq!(resp.status(), StatusCode::OK);
    Ok(String::from_utf8(test::read_body(resp).await.to_vec()).unwrap())
}

#[actix_web::test]
async fn test_a_parameter_in_a_projection_reaches_the_database() -> actix_web::Result<()> {
    let data = make_app_data().await;
    let path = match data.db.info.database_type {
        SupportedDatabase::Mssql => "/tests/parameter_binding/parameter_in_projection_mssql.sql",
        SupportedDatabase::Oracle => return Ok(()), // no CREATE TEMPORARY TABLE
        _ => "/tests/parameter_binding/parameter_in_projection.sql",
    };

    let page = rendered_page(&format!("{path}?x=1447"), data).await?;
    assert!(
        page.contains("1447"),
        "{page}\nexpected the bound parameter to reach the inserted row"
    );
    Ok(())
}

#[actix_web::test]
async fn test_parameterized_pages_leave_a_prepared_statement_in_the_cache() -> actix_web::Result<()>
{
    let data = make_app_data().await;
    if data.db.info.kind == AnyKind::Mssql {
        return Ok(()); // the MSSQL backend keeps no statement cache
    }

    for _ in 0..3 {
        let page = rendered_page(
            "/tests/parameter_binding/echo_parameter.sql?x=1447",
            data.clone(),
        )
        .await?;
        assert!(
            page.contains("1447"),
            "{page}\nexpected the bound parameter to reach the query"
        );
    }

    let connection = data.db.connection.acquire().await.unwrap();
    assert!(
        connection.cached_statements_size() > 0,
        "{:?} ran a parameterized query three times without caching a prepared statement",
        data.db.info.kind
    );
    Ok(())
}
