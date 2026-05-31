use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use actix_web::{http::StatusCode, test as actix_test, web::Data};
use sqlpage::{AppState, webserver::http::main_handler};
use sqlx::{Connection, Executor};

#[derive(Clone, Copy)]
enum Backend {
    Sqlite,
    Postgres,
    MySql,
}

#[derive(Clone, Copy)]
struct ExampleApp {
    name: &'static str,
    web_root: &'static str,
    config_dir: &'static str,
    route: &'static str,
    backend: Backend,
}

const SKIPPED_EXAMPLES: &[(&str, &str)] = &[
    (
        "PostGIS - using sqlpage with geographic data",
        "requires a PostGIS-enabled PostgreSQL database",
    ),
    (
        "make a geographic data application using sqlite extensions",
        "requires the SpatiaLite SQLite extension",
    ),
    (
        "microsoft sql server advanced forms",
        "requires a SQL Server service",
    ),
    (
        "single sign on",
        "requires a Keycloak/OpenID Connect service",
    ),
];

#[actix_web::test]
async fn example_applications_render_their_entrypoint() {
    crate::common::init_log();
    let postgres_base = prepare_postgres_base_url().await;
    let mysql_base = prepare_mysql_base_url().await;

    for example in example_apps() {
        let database_url = match example.backend {
            Backend::Sqlite => "sqlite::memory:".to_string(),
            Backend::Postgres => match &postgres_base {
                Some(base) => format!("{base}_{}", slug(example.name)),
                None => {
                    eprintln!("skipping {}: PostgreSQL is not available", example.name);
                    continue;
                }
            },
            Backend::MySql => match &mysql_base {
                Some(base) => format!("{base}_{}", slug(example.name)),
                None => {
                    eprintln!("skipping {}: MySQL is not available", example.name);
                    continue;
                }
            },
        };
        prepare_database(&database_url, example.backend).await;
        smoke_example(example, &database_url).await;
    }
}

#[test]
fn all_example_applications_are_smoked_or_explicitly_skipped() {
    let tested = example_apps()
        .into_iter()
        .map(|example| example.name)
        .collect::<HashSet<_>>();
    let skipped = SKIPPED_EXAMPLES
        .iter()
        .map(|(name, _reason)| *name)
        .collect::<HashSet<_>>();
    let mut missing = Vec::new();
    for entry in std::fs::read_dir("examples").unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !tested.contains(name.as_str()) && !skipped.contains(name.as_str()) {
            missing.push(name);
        }
    }
    missing.sort();
    assert!(
        missing.is_empty(),
        "examples missing from smoke coverage or explicit skip list: {missing:?}"
    );
}

fn example_apps() -> [ExampleApp; 31] {
    [
        ExampleApp {
            name: "CRUD - Authentication",
            web_root: "examples/CRUD - Authentication/www",
            config_dir: "examples/CRUD - Authentication/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "cards-with-remote-content",
            web_root: "examples/cards-with-remote-content",
            config_dir: "examples/cards-with-remote-content/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "charts, computations and custom components",
            web_root: "examples/charts, computations and custom components",
            config_dir: "examples/charts, computations and custom components/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "corporate-conundrum",
            web_root: "examples/corporate-conundrum",
            config_dir: "examples/corporate-conundrum/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "forms with a variable number of fields",
            web_root: "examples/forms with a variable number of fields",
            config_dir: "examples/forms with a variable number of fields/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "forms-with-multiple-steps",
            web_root: "examples/forms-with-multiple-steps",
            config_dir: "examples/forms-with-multiple-steps/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "handle-404",
            web_root: "examples/handle-404",
            config_dir: "examples/handle-404/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "image gallery with user uploads",
            web_root: "examples/image gallery with user uploads",
            config_dir: "examples/image gallery with user uploads/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "light-dark-toggle",
            web_root: "examples/light-dark-toggle",
            config_dir: "examples/light-dark-toggle/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "master-detail-forms",
            web_root: "examples/master-detail-forms",
            config_dir: "examples/master-detail-forms/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "modeling a many to many relationship with a form",
            web_root: "examples/modeling a many to many relationship with a form",
            config_dir: "examples/modeling a many to many relationship with a form/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "multiple-choice-question",
            web_root: "examples/multiple-choice-question",
            config_dir: "examples/multiple-choice-question/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "official-site",
            web_root: "examples/official-site",
            config_dir: "examples/official-site/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "plots tables and forms",
            web_root: "examples/plots tables and forms",
            config_dir: "examples/plots tables and forms/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "read-and-set-http-cookies",
            web_root: "examples/read-and-set-http-cookies",
            config_dir: "examples/read-and-set-http-cookies/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "rich-text-editor",
            web_root: "examples/rich-text-editor",
            config_dir: "examples/rich-text-editor/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "roundest_pokemon_rating",
            web_root: "examples/roundest_pokemon_rating/src",
            config_dir: "examples/roundest_pokemon_rating/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "sending emails",
            web_root: "examples/sending emails",
            config_dir: "examples/sending emails/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "simple-website-example",
            web_root: "examples/simple-website-example",
            config_dir: "examples/simple-website-example/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "splitwise",
            web_root: "examples/splitwise",
            config_dir: "examples/splitwise/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "todo application",
            web_root: "examples/todo application",
            config_dir: "examples/todo application/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "user-authentication",
            web_root: "examples/user-authentication",
            config_dir: "examples/user-authentication/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "using react and other custom scripts and styles",
            web_root: "examples/using react and other custom scripts and styles",
            config_dir: "examples/using react and other custom scripts and styles/sqlpage",
            route: "/",
            backend: Backend::Sqlite,
        },
        ExampleApp {
            name: "web servers - apache",
            web_root: "examples/web servers - apache/website",
            config_dir: "examples/web servers - apache/sqlpage_config",
            route: "/my_website/",
            backend: Backend::MySql,
        },
        ExampleApp {
            name: "custom form component",
            web_root: "examples/custom form component",
            config_dir: "examples/custom form component/sqlpage",
            route: "/",
            backend: Backend::MySql,
        },
        ExampleApp {
            name: "mysql json handling",
            web_root: "examples/mysql json handling",
            config_dir: "examples/mysql json handling/sqlpage",
            route: "/",
            backend: Backend::MySql,
        },
        ExampleApp {
            name: "nginx",
            web_root: "examples/nginx/website",
            config_dir: "examples/nginx/sqlpage_config",
            route: "/",
            backend: Backend::MySql,
        },
        ExampleApp {
            name: "SQLPage developer user interface",
            web_root: "examples/SQLPage developer user interface/website",
            config_dir: "examples/SQLPage developer user interface/sqlpage",
            route: "/",
            backend: Backend::Postgres,
        },
        ExampleApp {
            name: "telemetry",
            web_root: "examples/telemetry/website",
            config_dir: "examples/telemetry/sqlpage",
            route: "/",
            backend: Backend::Postgres,
        },
        ExampleApp {
            name: "tiny_twitter",
            web_root: "examples/tiny_twitter",
            config_dir: "examples/tiny_twitter/sqlpage",
            route: "/",
            backend: Backend::Postgres,
        },
        ExampleApp {
            name: "todo application (PostgreSQL)",
            web_root: "examples/todo application (PostgreSQL)",
            config_dir: "examples/todo application (PostgreSQL)/sqlpage",
            route: "/",
            backend: Backend::Postgres,
        },
    ]
}

async fn smoke_example(example: ExampleApp, database_url: &str) {
    let mut config = crate::common::test_config();
    config.web_root = PathBuf::from(example.web_root);
    config.configuration_directory = PathBuf::from(example.config_dir);
    config.database_url = database_url.to_string();
    config.max_database_pool_connections = Some(1);
    config.database_connection_retries = 0;
    config.sqlite_extensions.clear();
    if example.route.starts_with("/my_website/") {
        config.site_prefix = "/my_website/".to_string();
    }

    let state = AppState::init(&config)
        .await
        .unwrap_or_else(|err| panic!("{} failed to initialize: {err:#}", example.name));
    sqlpage::webserver::database::migrations::apply(&config, &state.db)
        .await
        .unwrap_or_else(|err| panic!("{} migrations failed: {err:#}", example.name));
    let data = Data::new(state);
    let req = actix_test::TestRequest::get()
        .uri(example.route)
        .app_data(data)
        .to_srv_request();
    let response = main_handler(req)
        .await
        .unwrap_or_else(|err| panic!("{} request failed: {err:#}", example.name));
    let status = response.status();
    let body = String::from_utf8_lossy(&actix_test::read_body(response).await).into_owned();

    assert!(
        status < StatusCode::INTERNAL_SERVER_ERROR,
        "{} returned {status}: {body}",
        example.name
    );
    assert!(
        !body.contains("sqlpage-error-description"),
        "{} rendered a SQLPage error: {body}",
        example.name
    );
}

async fn prepare_postgres_base_url() -> Option<String> {
    let admin_url = "postgres://root:Password123!@localhost/postgres";
    let mut conn = sqlx::PgConnection::connect(admin_url).await.ok()?;
    conn.execute("SELECT 1").await.ok()?;
    Some("postgres://root:Password123!@localhost/sqlpage_examples".to_string())
}

async fn prepare_mysql_base_url() -> Option<String> {
    let mut conn = sqlx::MySqlConnection::connect("mysql://root:Password123!@localhost/mysql")
        .await
        .ok()?;
    conn.execute("SELECT 1").await.ok()?;
    Some("mysql://root:Password123!@localhost/sqlpage_examples".to_string())
}

async fn prepare_database(database_url: &str, backend: Backend) {
    match backend {
        Backend::Sqlite => {}
        Backend::Postgres => {
            let db_name = database_url.rsplit('/').next().unwrap();
            let mut conn =
                sqlx::PgConnection::connect("postgres://root:Password123!@localhost/postgres")
                    .await
                    .unwrap();
            let _ = conn
                .execute(format!("DROP DATABASE IF EXISTS {db_name} WITH (FORCE)").as_str())
                .await;
            conn.execute(format!("CREATE DATABASE {db_name}").as_str())
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Backend::MySql => {
            let db_name = database_url.rsplit('/').next().unwrap();
            let mut conn =
                sqlx::MySqlConnection::connect("mysql://root:Password123!@localhost/mysql")
                    .await
                    .unwrap();
            conn.execute(format!("DROP DATABASE IF EXISTS {db_name}").as_str())
                .await
                .unwrap();
            conn.execute(format!("CREATE DATABASE {db_name}").as_str())
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

fn slug(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
