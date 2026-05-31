use actix_web::{http::header, test, web::Data};
use sqlpage::{
    AppState,
    app_config::{self, AppConfig},
    webserver::{
        database::migrations,
        http::{form_config, main_handler, payload_config},
    },
};
use std::collections::BTreeSet;
use std::env;
use std::path::Path;
use std::time::Duration;

struct ExampleSmoke {
    name: &'static str,
    web_root: Option<&'static str>,
    request_path: &'static str,
}

struct DatabaseExampleSmoke {
    name: &'static str,
    web_root: Option<&'static str>,
    request_path: &'static str,
    db_name: &'static str,
}

const SQLITE_SMOKE_EXAMPLES: &[ExampleSmoke] = &[
    ExampleSmoke {
        name: "CRUD - Authentication",
        web_root: Some("www"),
        request_path: "/",
    },
    ExampleSmoke {
        name: "cards-with-remote-content",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "charts, computations and custom components",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "corporate-conundrum",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "forms with a variable number of fields",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "forms-with-multiple-steps",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "handle-404",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "image gallery with user uploads",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "light-dark-toggle",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "master-detail-forms",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "modeling a many to many relationship with a form",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "multiple-choice-question",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "official-site",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "plots tables and forms",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "read-and-set-http-cookies",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "rich-text-editor",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "roundest_pokemon_rating",
        web_root: Some("src"),
        request_path: "/",
    },
    ExampleSmoke {
        name: "sending emails",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "simple-website-example",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "splitwise",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "todo application",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "using react and other custom scripts and styles",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "user-authentication",
        web_root: None,
        request_path: "/",
    },
];

const EXTERNAL_SERVICE_EXAMPLES: &[&str] = &[
    "PostGIS - using sqlpage with geographic data",
    "SQLPage developer user interface",
    "custom form component",
    "make a geographic data application using sqlite extensions",
    "microsoft sql server advanced forms",
    "mysql json handling",
    "nginx",
    "single sign on",
    "telemetry",
    "tiny_twitter",
    "todo application (PostgreSQL)",
    "web servers - apache",
];

const POSTGRES_SMOKE_EXAMPLES: &[DatabaseExampleSmoke] = &[
    DatabaseExampleSmoke {
        name: "SQLPage developer user interface",
        web_root: Some("website"),
        request_path: "/",
        db_name: "sqlpage_example_developer_ui",
    },
    DatabaseExampleSmoke {
        name: "telemetry",
        web_root: Some("website"),
        request_path: "/",
        db_name: "sqlpage_example_telemetry",
    },
    DatabaseExampleSmoke {
        name: "tiny_twitter",
        web_root: None,
        request_path: "/",
        db_name: "sqlpage_example_tiny_twitter",
    },
    DatabaseExampleSmoke {
        name: "todo application (PostgreSQL)",
        web_root: None,
        request_path: "/",
        db_name: "sqlpage_example_todo_postgres",
    },
    DatabaseExampleSmoke {
        name: "user-authentication",
        web_root: None,
        request_path: "/",
        db_name: "sqlpage_example_user_authentication",
    },
];

const MYSQL_SMOKE_EXAMPLES: &[DatabaseExampleSmoke] = &[
    DatabaseExampleSmoke {
        name: "custom form component",
        web_root: None,
        request_path: "/",
        db_name: "sqlpage_example_custom_form",
    },
    DatabaseExampleSmoke {
        name: "mysql json handling",
        web_root: None,
        request_path: "/",
        db_name: "sqlpage_example_mysql_json",
    },
    DatabaseExampleSmoke {
        name: "nginx",
        web_root: Some("website"),
        request_path: "/",
        db_name: "sqlpage_example_nginx",
    },
    DatabaseExampleSmoke {
        name: "web servers - apache",
        web_root: Some("website"),
        request_path: "/my_website/",
        db_name: "sqlpage_example_apache",
    },
];

#[actix_web::test]
async fn examples_folder_is_fully_accounted_for() {
    let actual = top_level_example_directories();
    let accounted_for = SQLITE_SMOKE_EXAMPLES
        .iter()
        .map(|example| example.name.to_owned())
        .chain(
            EXTERNAL_SERVICE_EXAMPLES
                .iter()
                .map(|name| (*name).to_owned()),
        )
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, accounted_for);
}

#[actix_web::test]
async fn postgres_examples_render_their_entry_page_when_database_url_is_provided() {
    let Ok(admin_url) = env::var("SQLPAGE_TEST_EXAMPLES_POSTGRES_ADMIN_URL") else {
        return;
    };

    for example in POSTGRES_SMOKE_EXAMPLES {
        let database_url = recreate_postgres_database(&admin_url, example.db_name)
            .await
            .unwrap_or_else(|err| panic!("failed to prepare {:?}: {err:#}", example.name));
        assert_database_example_renders(example, &database_url).await;
    }
}

#[actix_web::test]
async fn mysql_examples_render_their_entry_page_when_database_url_is_provided() {
    let Ok(admin_url) = env::var("SQLPAGE_TEST_EXAMPLES_MYSQL_ADMIN_URL") else {
        return;
    };

    for example in MYSQL_SMOKE_EXAMPLES {
        let database_url = recreate_mysql_database(&admin_url, example.db_name)
            .await
            .unwrap_or_else(|err| panic!("failed to prepare {:?}: {err:#}", example.name));
        assert_database_example_renders(example, &database_url).await;
    }
}

#[actix_web::test]
async fn sqlite_compatible_examples_render_their_entry_page() {
    for example in SQLITE_SMOKE_EXAMPLES {
        let response = request_example(example).await.unwrap_or_else(|err| {
            panic!(
                "failed to render entry page for example {:?}: {err:#}",
                example.name
            )
        });

        let status = response.status();
        let body = test::read_body(response).await;
        let body = String::from_utf8_lossy(&body);
        assert!(
            status.is_success()
                || status.is_redirection()
                || status == actix_web::http::StatusCode::UNAUTHORIZED,
            "example {:?} returned status {status}; body:\n{body}",
            example.name
        );
        assert!(
            !body.contains("SQLPage Error"),
            "example {:?} rendered an SQLPage error with status {status}; body:\n{body}",
            example.name
        );
    }
}

async fn assert_database_example_renders(example: &DatabaseExampleSmoke, database_url: &str) {
    let response = request_database_example(example, database_url)
        .await
        .unwrap_or_else(|err| {
            panic!(
                "failed to render entry page for example {:?}: {err:#}",
                example.name
            )
        });

    assert_successful_example_response(example.name, response).await;
}

async fn assert_successful_example_response(
    example_name: &str,
    response: actix_web::dev::ServiceResponse,
) {
    let status = response.status();
    let body = test::read_body(response).await;
    let body = String::from_utf8_lossy(&body);
    assert!(
        status.is_success()
            || status.is_redirection()
            || status == actix_web::http::StatusCode::UNAUTHORIZED,
        "example {example_name:?} returned status {status}; body:\n{body}",
    );
    assert!(
        !body.contains("SQLPage Error"),
        "example {example_name:?} rendered an SQLPage error with status {status}; body:\n{body}",
    );
}

async fn request_example(
    example: &ExampleSmoke,
) -> anyhow::Result<actix_web::dev::ServiceResponse> {
    let app_data = make_example_app_data(example).await?;
    let request = test::TestRequest::get()
        .uri(example.request_path)
        .insert_header(header::Accept::html())
        .app_data(payload_config(&app_data))
        .app_data(form_config(&app_data))
        .app_data(app_data)
        .to_srv_request();
    tokio::time::timeout(Duration::from_secs(8), main_handler(request))
        .await
        .map_err(|err| anyhow::anyhow!("request timed out: {err}"))?
        .map_err(|err| anyhow::anyhow!("request failed: {err:#}"))
}

async fn request_database_example(
    example: &DatabaseExampleSmoke,
    database_url: &str,
) -> anyhow::Result<actix_web::dev::ServiceResponse> {
    let app_data = make_database_example_app_data(example, database_url).await?;
    let request = test::TestRequest::get()
        .uri(example.request_path)
        .insert_header(header::Accept::html())
        .app_data(payload_config(&app_data))
        .app_data(form_config(&app_data))
        .app_data(app_data)
        .to_srv_request();
    tokio::time::timeout(Duration::from_secs(8), main_handler(request))
        .await
        .map_err(|err| anyhow::anyhow!("request timed out: {err}"))?
        .map_err(|err| anyhow::anyhow!("request failed: {err:#}"))
}

async fn make_example_app_data(example: &ExampleSmoke) -> anyhow::Result<Data<AppState>> {
    sqlpage::telemetry::init_test_logging();

    let root = Path::new("examples").join(example.name);
    let mut config = load_example_config(&root)?;
    config.database_url = "sqlite://:memory:?cache=shared".to_owned();
    config.max_database_pool_connections = Some(1);
    config.database_connection_retries = 0;
    config.database_connection_acquire_timeout_seconds = 8.0;
    config.web_root = example
        .web_root
        .map_or_else(|| root.clone(), |web_root| root.join(web_root));

    let state = Data::new(AppState::init(&config).await?);
    migrations::apply(&config, &state.db).await?;
    Ok(state)
}

async fn make_database_example_app_data(
    example: &DatabaseExampleSmoke,
    database_url: &str,
) -> anyhow::Result<Data<AppState>> {
    sqlpage::telemetry::init_test_logging();

    let root = Path::new("examples").join(example.name);
    let mut config = load_example_config(&root)?;
    config.database_url = database_url.to_owned();
    config.max_database_pool_connections = Some(1);
    config.database_connection_retries = 0;
    config.database_connection_acquire_timeout_seconds = 8.0;
    config.web_root = example
        .web_root
        .map_or_else(|| root.clone(), |web_root| root.join(web_root));

    let state = Data::new(AppState::init(&config).await?);
    migrations::apply(&config, &state.db).await?;
    Ok(state)
}

fn load_example_config(root: &Path) -> anyhow::Result<AppConfig> {
    let config_dir = if root.join("sqlpage").exists() {
        root.join("sqlpage")
    } else if root.join("sqlpage_config").exists() {
        root.join("sqlpage_config")
    } else {
        root.join("sqlpage")
    };

    if config_dir.join("sqlpage.json").exists() || config_dir.join("sqlpage.yaml").exists() {
        let mut config = app_config::load_from_directory(&config_dir)?;
        config.configuration_directory = config_dir;
        Ok(config)
    } else {
        let mut config = serde_json::from_str::<AppConfig>(
            r#"{
                "database_url": "sqlite://:memory:",
                "max_database_pool_connections": 1,
                "database_connection_retries": 0,
                "database_connection_acquire_timeout_seconds": 8
            }"#,
        )?;
        config.configuration_directory = config_dir;
        Ok(config)
    }
}

async fn recreate_postgres_database(admin_url: &str, db_name: &str) -> anyhow::Result<String> {
    use sqlx::Connection;

    validate_database_name(db_name)?;
    let mut conn = sqlx::PgConnection::connect(admin_url).await?;
    let drop_database = format!(r#"DROP DATABASE IF EXISTS "{db_name}" WITH (FORCE)"#);
    let create_database = format!(r#"CREATE DATABASE "{db_name}""#);
    sqlx::query(sqlx::AssertSqlSafe(drop_database.as_str()))
        .execute(&mut conn)
        .await?;
    sqlx::query(sqlx::AssertSqlSafe(create_database.as_str()))
        .execute(&mut conn)
        .await?;

    database_url_with_path(admin_url, db_name)
}

async fn recreate_mysql_database(admin_url: &str, db_name: &str) -> anyhow::Result<String> {
    use sqlx::Connection;

    validate_database_name(db_name)?;
    let mut conn = sqlx::MySqlConnection::connect(admin_url).await?;
    let drop_database = format!("DROP DATABASE IF EXISTS `{db_name}`");
    let create_database = format!("CREATE DATABASE `{db_name}`");
    sqlx::query(sqlx::AssertSqlSafe(drop_database.as_str()))
        .execute(&mut conn)
        .await?;
    sqlx::query(sqlx::AssertSqlSafe(create_database.as_str()))
        .execute(&mut conn)
        .await?;

    database_url_with_path(admin_url, db_name)
}

fn validate_database_name(db_name: &str) -> anyhow::Result<()> {
    if db_name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit())
    {
        Ok(())
    } else {
        anyhow::bail!("invalid test database name: {db_name}");
    }
}

fn database_url_with_path(admin_url: &str, db_name: &str) -> anyhow::Result<String> {
    let mut url = url::Url::parse(admin_url)?;
    url.set_path(db_name);
    Ok(url.to_string())
}

fn top_level_example_directories() -> BTreeSet<String> {
    std::fs::read_dir("examples")
        .unwrap()
        .map(Result::unwrap)
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.file_name().into_string().unwrap())
        .collect()
}
