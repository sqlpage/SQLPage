use actix_web::{http::StatusCode, test};
use sqlpage::{webserver, AppState};
use sqlx::Executor as _;

use crate::common::{make_app_data_from_config, req_path, req_path_with_app_data, test_config};

#[actix_web::test]
async fn test_concurrent_requests() {
    let components = [
        "table", "form", "card", "datagrid", "hero", "list", "timeline",
    ];
    let app_data = make_app_data_from_config(test_config()).await;
    let reqs = (0..64)
        .map(|i| {
            let component = components[i % components.len()];
            req_path_with_app_data(
                format!("/tests/components/any_component.sql?component={component}"),
                app_data.clone(),
            )
        })
        .collect::<Vec<_>>();
    let results = futures_util::future::join_all(reqs).await;
    for result in results.into_iter() {
        let resp = result.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        assert!(
            body.starts_with(b"<!DOCTYPE html>"),
            "Expected html doctype"
        );
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(
            body.contains("It works !"),
            "Expected to contain: It works !, but got: {body}"
        );
        assert!(!body.contains("error"));
    }
}

#[actix_web::test]
async fn test_routing_with_db_fs() {
    let mut config = test_config();
    if config.database_url.contains("memory") {
        return;
    }

    config.site_prefix = "/prefix/".to_string();
    let state = AppState::init(&config).await.unwrap();

    let create_table_sql =
        sqlpage::filesystem::DbFsQueries::get_create_table_sql(state.db.connection.any_kind());
    state
        .db
        .connection
        .execute(format!("DROP TABLE IF EXISTS sqlpage_files; {create_table_sql}").as_ref())
        .await
        .unwrap();

    let insert_sql = match state.db.connection.any_kind() {
        sqlx::any::AnyKind::Mssql => "INSERT INTO sqlpage_files(path, contents) VALUES ('on_db.sql', CONVERT(VARBINARY(MAX), 'select ''text'' as component, ''Hi from db !'' AS contents;'))",
        _ => "INSERT INTO sqlpage_files(path, contents) VALUES ('on_db.sql', 'select ''text'' as component, ''Hi from db !'' AS contents;')"
    };
    state.db.connection.execute(insert_sql).await.unwrap();

    let state = AppState::init(&config).await.unwrap();
    let app_data = actix_web::web::Data::new(state);

    let resp = req_path_with_app_data("/prefix/on_db.sql", app_data.clone())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("Hi from db !"),
        "{body_str}\nexpected to contain: Hi from db !"
    );
}

#[actix_web::test]
async fn test_routing_with_prefix() {
    let mut config = test_config();
    config.site_prefix = "/prefix/".to_string();
    let state = AppState::init(&config).await.unwrap();

    let app_data = actix_web::web::Data::new(state);
    let resp = req_path_with_app_data(
        "/prefix/tests/sql_test_files/it_works_simple.sql",
        app_data.clone(),
    )
    .await
    .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("It works !"),
        "{body_str}\nexpected to contain: It works !"
    );
    assert!(
        body_str.contains("href=\"/prefix/"),
        "{body_str}\nexpected to contain links with site prefix"
    );

    let resp = req_path_with_app_data("/prefix/nonexistent.sql", app_data.clone())
        .await
        .expect_err("Expected 404 error")
        .to_string();
    assert!(
        resp.contains("404"),
        "Response should contain \"404\", but got:\n{resp}"
    );

    let resp = req_path_with_app_data("/prefix/sqlpage/migrations/0001_init.sql", app_data.clone())
        .await
        .expect_err("Expected forbidden error")
        .to_string();
    assert!(resp.to_lowercase().contains("forbidden"), "{resp}");

    let resp = req_path_with_app_data("/tests/sql_test_files/it_works_simple.sql", app_data)
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::MOVED_PERMANENTLY);
    let location = resp
        .headers()
        .get("location")
        .expect("location header should be present");
    assert_eq!(location.to_str().unwrap(), "/prefix/");
}

#[actix_web::test]
async fn test_hidden_files() {
    let resp_result = req_path("/tests/core/.hidden.sql").await;
    assert!(
        resp_result.is_err(),
        "Accessing a hidden file should be forbidden, but received success: {resp_result:?}"
    );
    let resp = resp_result.unwrap_err().error_response();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let srv_resp = actix_web::test::TestRequest::default().to_srv_response(resp);
    let body = test::read_body(srv_resp).await;
    assert!(String::from_utf8_lossy(&body)
        .to_lowercase()
        .contains("forbidden"),);
}

#[actix_web::test]
async fn test_official_website_documentation() {
    let app_data = make_app_data_for_official_website().await;
    let resp = req_path_with_app_data("/component.sql?component=button", app_data)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to get response for /component.sql?component=button: {e}")
        });
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains(r#"<button type="submit" form="poem" formaction="?action"#),
        "{body_str}\nexpected to contain a button with formaction"
    );
}

#[actix_web::test]
async fn test_official_website_basic_auth_example() {
    let resp = req_path_with_app_data(
        "/examples/authentication/basic_auth.sql",
        make_app_data_for_official_website().await,
    )
    .await
    .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        body_str.contains("not authorized"),
        "{body_str}\nexpected to contain Unauthorized"
    );
}

async fn make_app_data_for_official_website() -> actix_web::web::Data<AppState> {
    crate::common::init_log();
    let config_path = std::path::Path::new("examples/official-site/sqlpage");
    let mut app_config = sqlpage::app_config::load_from_directory(config_path).unwrap();
    app_config.web_root = std::path::PathBuf::from("examples/official-site");
    app_config.database_url = "sqlite::memory:".to_string();
    let app_state = make_app_data_from_config(app_config.clone()).await;
    webserver::database::migrations::apply(&app_config, &app_state.db)
        .await
        .unwrap();
    app_state
}
