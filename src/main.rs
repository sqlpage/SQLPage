mod render;
mod templates;
mod utils;
mod webserver;

use sqlx::any::AnyConnectOptions;
use sqlx::{AnyPool, ConnectOptions};
use templates::AllTemplates;

const WEB_ROOT: &str = ".";
const CONFIG_DIR: &str = "sqlpage";
const TEMPLATES_DIR: &str = "sqlpage/templates";
const MIGRATIONS_DIR: &str = "sqlpage/migrations";

#[cfg(not(feature = "lambda-web"))]
const DEFAULT_DATABASE: &str = "sqlite://site.db?mode=rwc";
#[cfg(feature = "lambda-web")]
const DEFAULT_DATABASE: &str = "sqlite://:memory:";

pub struct AppState {
    db: AnyPool,
    all_templates: AllTemplates,
}

pub struct Config {
    listen_on: std::net::SocketAddr,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Connect to the database
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE.to_string());

    let mut connect_options: AnyConnectOptions =
        database_url.parse().expect("Invalid database URL");
    connect_options.log_statements(log::LevelFilter::Trace);
    connect_options.log_slow_statements(
        log::LevelFilter::Warn,
        std::time::Duration::from_millis(250),
    );
    let db = AnyPool::connect_with(connect_options)
        .await
        .expect("Failed to connect to database");

    if let Err(e) = webserver::apply_migrations(&db).await {
        log::error!(
            "An error occurred while running the database migration.
        The path '{MIGRATIONS_DIR}' has to point to a directory, which contains valid SQL files
        with names using the format '<VERSION>_<DESCRIPTION>.sql',
        where <VERSION> is a positive number, and <DESCRIPTION> is a string.
        The current state of migrations will be stored in a table called _sqlx_migrations.\n {e:?}"
        )
    }

    log::info!("Connected to database: {database_url}");

    let listen_on = std::env::var("LISTEN_ON")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse::<std::net::SocketAddr>()
        .expect("LISTEN_ON must be a valid IP:PORT");

    log::info!("Starting server on {}", listen_on);

    let all_templates = AllTemplates::init();
    let state = AppState { db, all_templates };
    let config = Config { listen_on };
    webserver::http::run_server(config, state).await
}
