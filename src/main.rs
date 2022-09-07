extern crate core;

mod render;
mod templates;
mod utils;
mod webserver;

use crate::webserver::{Database, init_database};
use anyhow::Context;
use std::env;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use templates::AllTemplates;
use crate::webserver::database::{FileCache, ParsedSqlFile};

const WEB_ROOT: &str = ".";
const CONFIG_DIR: &str = "sqlpage";
const TEMPLATES_DIR: &str = "sqlpage/templates";
const MIGRATIONS_DIR: &str = "sqlpage/migrations";

const DEFAULT_DATABASE_FILE: &str = "sqlpage.db";

pub struct AppState {
    db: Database,
    all_templates: AllTemplates,
    web_root: PathBuf,
    sql_file_cache: FileCache<ParsedSqlFile>
}

pub struct Config {
    listen_on: SocketAddr,
}

#[actix_web::main]
async fn main() {
    init_logging();
    if let Err(e) = start().await {
        log::error!("{:?}", e);
        std::process::exit(1);
    }
}

async fn start() -> anyhow::Result<()> {
    // Connect to the database
    let database_url = get_database_url();

    let db = init_database(&database_url).await?;

    webserver::apply_migrations(&db).await?;

    log::info!("Connected to database: {database_url}");
    let listen_on = get_listen_on()?;
    log::info!("Starting server on {}", listen_on);
    let all_templates = AllTemplates::init()?;
    let web_root = std::fs::canonicalize(WEB_ROOT)?;
    let sql_file_cache = FileCache::default();
    let state = AppState { db, all_templates, web_root, sql_file_cache };
    let config = Config { listen_on };
    webserver::http::run_server(config, state).await?;
    Ok(())
}

fn get_listen_on() -> anyhow::Result<SocketAddr> {
    let host_str = env::var("LISTEN_ON").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let mut host_addr = host_str
        .to_socket_addrs()?
        .next()
        .with_context(|| format!("host '{}' does not resolve to an IP", host_str))?;
    if let Ok(port) = env::var("PORT") {
        host_addr.set_port(port.parse().with_context(|| "Invalid PORT")?);
    }
    Ok(host_addr)
}

fn init_logging() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
}

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| default_database_url())
}

fn default_database_url() -> String {
    let prefix = "sqlite://".to_owned();

    #[cfg(not(feature = "lambda-web"))]
    if std::path::Path::new(DEFAULT_DATABASE_FILE).exists() {
        log::info!(
            "No DATABASE_URL, using the default sqlite database './{DEFAULT_DATABASE_FILE}'"
        );
        return prefix + DEFAULT_DATABASE_FILE;
    } else if let Ok(tmp_file) = std::fs::File::create(DEFAULT_DATABASE_FILE) {
        log::info!("No DATABASE_URL provided, the current directory is writeable, creating {DEFAULT_DATABASE_FILE}");
        drop(tmp_file);
        std::fs::remove_file(DEFAULT_DATABASE_FILE).expect("removing temp file");
        return prefix + DEFAULT_DATABASE_FILE + "?mode=rwc";
    }

    log::warn!("No DATABASE_URL provided, and the current directory is not writeable. Using a temporary in-memory SQLite database. All the data created will be lost when this server shuts down.");
    prefix + ":memory:"
}
