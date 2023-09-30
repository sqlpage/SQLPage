#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

extern crate core;

pub mod app_config;
pub mod file_cache;
pub mod filesystem;
pub mod render;
pub mod templates;
pub mod utils;
pub mod webserver;

use crate::app_config::AppConfig;
use crate::filesystem::FileSystem;
use crate::webserver::database::{FileCache, ParsedSqlFile};
use std::net::SocketAddr;
use std::path::PathBuf;
use templates::AllTemplates;
use webserver::Database;

pub const TEMPLATES_DIR: &str = "sqlpage/templates";
pub const MIGRATIONS_DIR: &str = "sqlpage/migrations";
pub const ON_CONNECT_FILE: &str = "sqlpage/on_connect.sql";

pub struct AppState {
    pub db: Database,
    all_templates: AllTemplates,
    sql_file_cache: FileCache<ParsedSqlFile>,
    file_system: FileSystem,
    config: AppConfig,
}

impl AppState {
    pub async fn init(config: &AppConfig) -> anyhow::Result<Self> {
        // Connect to the database
        let db = Database::init(config).await?;
        let all_templates = AllTemplates::init()?;
        let mut sql_file_cache = FileCache::new();
        let file_system = FileSystem::init(&config.web_root, &db).await;
        sql_file_cache.add_static(
            PathBuf::from("index.sql"),
            ParsedSqlFile::new(&db, include_str!("../index.sql")).await,
        );
        Ok(AppState {
            db,
            all_templates,
            sql_file_cache,
            file_system,
            config: config.clone(),
        })
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish()
    }
}

pub struct Config {
    pub listen_on: SocketAddr,
}
