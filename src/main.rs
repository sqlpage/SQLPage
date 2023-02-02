#![deny(clippy::pedantic)]
extern crate core;

mod app_config;
mod file_cache;
mod filesystem;
mod render;
mod templates;
mod utils;
mod webserver;

use crate::app_config::AppConfig;
use crate::filesystem::FileSystem;
use crate::webserver::database::{FileCache, ParsedSqlFile};
use crate::webserver::Database;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use templates::AllTemplates;

const TEMPLATES_DIR: &str = "sqlpage/templates";
const MIGRATIONS_DIR: &str = "sqlpage/migrations";

pub struct AppState {
    db: Database,
    all_templates: AllTemplates,
    sql_file_cache: FileCache<ParsedSqlFile>,
    file_system: FileSystem,
}

impl AppState {
    async fn init(config: &AppConfig) -> anyhow::Result<Self> {
        // Connect to the database
        let db = Database::init(config).await?;
        let all_templates = AllTemplates::init()?;
        let web_root = get_web_root();
        let sql_file_cache = FileCache::new();
        let file_system = FileSystem::init(&web_root, &db).await;
        Ok(AppState {
            db,
            all_templates,
            sql_file_cache,
            file_system,
        })
    }
}

fn get_web_root() -> PathBuf {
    env::var("WEB_ROOT").map_or_else(
        |_| PathBuf::from(&std::path::Component::CurDir),
        PathBuf::from,
    )
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
    let config = app_config::load()?;
    log::debug!("Starting with the following configuration: {config:?}");
    let state = AppState::init(&config).await?;
    webserver::apply_migrations(&state.db).await?;
    let listen_on = config.listen_on;
    log::info!("Starting server on {}", listen_on);
    let config = Config { listen_on };
    webserver::http::run_server(config, state).await?;
    Ok(())
}

fn init_logging() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
}
