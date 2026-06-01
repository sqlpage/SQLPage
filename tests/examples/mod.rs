use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use actix_web::{http::StatusCode, test as actix_test, web::Data};
use serde::Deserialize;
use sqlpage::{AppState, webserver::http::main_handler};
use sqlx::{Connection, Executor};

const EXAMPLES_DIR: &str = "examples";
const TEST_CONFIG_FILE: &str = "sqlpage_test.json";

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Backend {
    Sqlite,
    Postgres,
    MySql,
}

struct ExampleApp {
    name: String,
    web_root: PathBuf,
    config_dir: PathBuf,
    route: String,
    site_prefix: Option<String>,
    backend: Backend,
    skip: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExampleTestConfig {
    backend: Option<Backend>,
    web_root: Option<PathBuf>,
    config_dir: Option<PathBuf>,
    route: Option<String>,
    site_prefix: Option<String>,
    skip: Option<String>,
}

#[actix_web::test]
async fn example_applications_render_their_entrypoint() {
    crate::common::init_log();
    let postgres_base = prepare_postgres_base_url().await;
    let mysql_base = prepare_mysql_base_url().await;

    for example in example_apps() {
        if let Some(reason) = &example.skip {
            eprintln!("skipping {}: {reason}", example.name);
            continue;
        }
        let database_url = match example.backend {
            Backend::Sqlite => "sqlite::memory:".to_string(),
            Backend::Postgres => match &postgres_base {
                Some(base) => format!("{base}_{}", slug(&example.name)),
                None => {
                    eprintln!("skipping {}: PostgreSQL is not available", example.name);
                    continue;
                }
            },
            Backend::MySql => match &mysql_base {
                Some(base) => format!("{base}_{}", slug(&example.name)),
                None => {
                    eprintln!("skipping {}: MySQL is not available", example.name);
                    continue;
                }
            },
        };
        prepare_database(&database_url, example.backend).await;
        smoke_example(&example, &database_url).await;
    }
}

#[test]
fn example_applications_are_discovered_from_the_examples_directory() {
    let apps = example_apps();
    assert!(!apps.is_empty(), "no example applications were discovered");

    let mut names = HashSet::new();
    let mut duplicates = Vec::new();
    for example in apps {
        if !names.insert(example.name.clone()) {
            duplicates.push(example.name);
        }
    }
    assert!(
        duplicates.is_empty(),
        "duplicate example applications discovered: {duplicates:?}"
    );
}

fn example_apps() -> Vec<ExampleApp> {
    let mut apps = Vec::new();
    for entry in std::fs::read_dir(EXAMPLES_DIR).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let root = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let test_config = read_test_config(&root);

        apps.push(ExampleApp {
            name,
            web_root: resolve_example_path(&root, test_config.web_root, "."),
            config_dir: resolve_example_path(&root, test_config.config_dir, "sqlpage"),
            route: test_config.route.unwrap_or_else(|| "/".to_string()),
            site_prefix: test_config.site_prefix,
            backend: test_config.backend.unwrap_or(Backend::Sqlite),
            skip: test_config.skip,
        });
    }
    apps.sort_by(|left, right| left.name.cmp(&right.name));
    apps
}

fn read_test_config(root: &Path) -> ExampleTestConfig {
    let path = root.join(TEST_CONFIG_FILE);
    if !path.exists() {
        return ExampleTestConfig::default();
    }
    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err:#}", path.display()));
    serde_json::from_str(&contents)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err:#}", path.display()))
}

fn resolve_example_path(root: &Path, configured: Option<PathBuf>, default: &str) -> PathBuf {
    let path = configured.unwrap_or_else(|| PathBuf::from(default));
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

async fn smoke_example(example: &ExampleApp, database_url: &str) {
    let mut config = crate::common::test_config();
    config.web_root = example.web_root.clone();
    config.configuration_directory = example.config_dir.clone();
    config.database_url = database_url.to_string();
    config.max_database_pool_connections = Some(1);
    config.database_connection_retries = 0;
    config.sqlite_extensions.clear();
    if let Some(site_prefix) = &example.site_prefix {
        config.site_prefix.clone_from(site_prefix);
    }

    let state = AppState::init(&config)
        .await
        .unwrap_or_else(|err| panic!("{} failed to initialize: {err:#}", example.name));
    sqlpage::webserver::database::migrations::apply(&config, &state.db)
        .await
        .unwrap_or_else(|err| panic!("{} migrations failed: {err:#}", example.name));
    let data = Data::new(state);
    let req = actix_test::TestRequest::get()
        .uri(&example.route)
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
