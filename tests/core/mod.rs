use actix_web::http::StatusCode;
use sqlpage::{
    AppState,
    webserver::{self, make_placeholder},
};

use crate::common::{
    make_app_data_from_config, response_for, response_with, response_with_data, test_config,
};

mod path_aliases;

/// Creates the `sqlpage_files` table if needed and stores `contents` at `path`.
/// Other tests share this database, so the table is never dropped.
async fn store_file_in_db(state: &AppState, path: &str, contents: &[u8]) {
    use sqlx::executor::Executor as _;

    let create_table_sql =
        sqlpage::filesystem::DbFsQueries::get_create_table_sql(state.db.info.database_type);
    if state
        .db
        .connection
        .execute("SELECT 1 FROM sqlpage_files WHERE 1 = 0")
        .await
        .is_err()
    {
        state.db.connection.execute(create_table_sql).await.unwrap();
    }
    let delete_sql = format!("DELETE FROM sqlpage_files WHERE path = '{path}'");
    state
        .db
        .connection
        .execute(delete_sql.as_str())
        .await
        .unwrap();
    let insert_sql = format!(
        "INSERT INTO sqlpage_files(path, contents) VALUES ({}, {})",
        make_placeholder(state.db.info.kind, 1),
        make_placeholder(state.db.info.kind, 2)
    );
    sqlx::query::query(&insert_sql)
        .bind(path)
        .bind(contents)
        .execute(&state.db.connection)
        .await
        .unwrap();
}

#[actix_web::test]
async fn test_concurrent_requests() {
    let components = [
        "table", "form", "card", "datagrid", "hero", "list", "timeline",
    ];
    let app_data = crate::common::make_app_data().await;
    let reqs = (0..64)
        .map(|i| {
            let component = components[i % components.len()];
            response_with_data(
                format!("/tests/components/any_component.sql?component={component}"),
                app_data.clone(),
            )
        })
        .collect::<Vec<_>>();
    let results = futures_util::future::join_all(reqs).await;
    for result in results {
        let resp = result.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = crate::common::read_body_string(resp).await;
        assert!(body.starts_with("<!DOCTYPE html>"), "Expected html doctype");
        assert!(
            body.contains("It works !"),
            "Expected to contain: It works !, but got: {body}"
        );
        assert!(!body.contains("error"));
    }
}

#[actix_web::test]
async fn test_datagrid_description_presence_controls_placeholder() {
    let resp = response_for("/tests/components/datagrid_icon_only.sql").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = crate::common::read_body_string(resp).await;
    assert!(body.contains("Facebook"), "{body}");
    assert!(body.contains("Empty"), "{body}");
    assert!(body.contains("Missing"), "{body}");
    assert!(body.contains("<svg"), "{body}");
    assert_eq!(body.matches('–').count(), 1, "{body}");
}

#[actix_web::test]
async fn test_routing_with_db_fs() {
    let mut config = test_config();
    if config.database_url.contains("memory") {
        return;
    }

    config.site_prefix = "/prefix/".to_string();
    let state = AppState::init(&config).await.unwrap();

    if crate::common::supports_database(
        &state.db,
        &[webserver::database::SupportedDatabase::Oracle],
    ) {
        return;
    }

    store_file_in_db(
        &state,
        "on_db.sql",
        b"select ''text'' as component, ''Hi from db !'' AS contents;",
    )
    .await;

    let resp = response_with("/prefix/on_db.sql", config).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_str = crate::common::read_body_string(resp).await;
    assert!(
        body_str.contains("Hi from db !"),
        "{body_str}\nexpected to contain: Hi from db !"
    );
}

#[cfg(unix)]
#[actix_web::test]
async fn test_non_unicode_static_path_returns_bad_request_with_db_fs() {
    let mut config = test_config();
    if !config.database_url.starts_with("sqlite") {
        return;
    }
    config.database_url =
        "sqlite://file:test_non_unicode_static_path?mode=memory&cache=shared".to_string();

    let state = AppState::init(&config).await.unwrap();
    let expected_db_path = "\u{FFFD}.txt";
    store_file_in_db(&state, expected_db_path, b"file from db fs").await;

    let err = response_with("/%FF.txt", config)
        .await
        .expect_err("non-unicode path should not panic and must return bad request");
    assert_eq!(
        err.as_response_error().status_code(),
        StatusCode::BAD_REQUEST
    );
}

#[actix_web::test]
async fn test_routing_with_prefix() {
    let mut config = test_config();
    config.site_prefix = "/prefix/".to_string();
    let app_data = make_app_data_from_config(config).await.unwrap();
    let resp = response_with_data(
        "/prefix/tests/sql_test_files/component_rendering/simple.sql",
        app_data.clone(),
    )
    .await
    .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_str = crate::common::read_body_string(resp).await;
    assert!(
        body_str.contains("It works !"),
        "{body_str}\nexpected to contain: It works !"
    );
    assert!(
        body_str.contains("href=\"/prefix/"),
        "{body_str}\nexpected to contain links with site prefix"
    );

    let resp = response_with_data("/prefix/nonexistent.sql", app_data.clone())
        .await
        .expect("should handle 404");
    let body_str = crate::common::read_body_string(resp).await;
    assert!(
        body_str.contains("404"),
        "Response should contain \"404\", but got:\n{body_str}"
    );

    let resp = response_with_data("/prefix/sqlpage/migrations/0001_init.sql", app_data.clone())
        .await
        .expect_err("Expected forbidden error")
        .as_response_error()
        .status_code();
    assert_eq!(resp, StatusCode::FORBIDDEN);

    let resp = response_with_data(
        "/tests/sql_test_files/component_rendering/simple.sql",
        app_data,
    )
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
    let resp_result = response_with("/tests/core/.hidden.sql", test_config()).await;
    assert!(
        resp_result.is_err(),
        "Accessing a hidden file should be forbidden, but received success: {resp_result:?}"
    );
    let resp = resp_result.unwrap_err().error_response();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let body = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
    assert!(
        String::from_utf8_lossy(&body)
            .to_lowercase()
            .contains("forbidden"),
    );
}

#[actix_web::test]
async fn test_official_website_documentation() {
    let app_data = make_app_data_for_official_website().await;
    let resp = response_with_data("/component.sql?component=button", app_data)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to get response for /component.sql?component=button: {e}")
        });
    assert_eq!(resp.status(), StatusCode::OK);
    let body_str = crate::common::read_body_string(resp).await;
    assert!(
        body_str.contains(r#"<button type="submit" form="poem" formaction="?action"#),
        "{body_str}\nexpected to contain a button with formaction"
    );
}

#[actix_web::test]
async fn test_official_website_basic_auth_example() {
    let resp = response_with_data(
        "/examples/authentication/basic_auth.sql",
        make_app_data_for_official_website().await,
    )
    .await
    .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let body_str = crate::common::read_body_string(resp).await;
    assert!(
        body_str.contains("Unauthorized"),
        "{body_str}\nexpected to contain Unauthorized"
    );
}

async fn make_app_data_for_official_website() -> actix_web::web::Data<AppState> {
    crate::common::init_log();
    let config_path = std::path::Path::new("examples/official-site/sqlpage");
    let mut app_config = sqlpage::app_config::load_from_directory(config_path).unwrap();
    app_config.web_root = std::path::PathBuf::from("examples/official-site");
    app_config.database_url = "sqlite::memory:".to_string();
    let app_state = make_app_data_from_config(app_config.clone()).await.unwrap();
    webserver::database::migrations::apply(&app_config, &app_state.db)
        .await
        .unwrap();
    app_state
}
