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
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use templates::AllTemplates;
use webserver::Database;

pub const TEMPLATES_DIR: &str = "sqlpage/templates";
pub const MIGRATIONS_DIR: &str = "sqlpage/migrations";

pub struct AppState {
    pub db: Database,
    all_templates: AllTemplates,
    sql_file_cache: FileCache<ParsedSqlFile>,
    file_system: FileSystem,
}

impl AppState {
    pub async fn init(config: &AppConfig) -> anyhow::Result<Self> {
        // Connect to the database
        let db = Database::init(config).await?;
        let all_templates = AllTemplates::init()?;
        let web_root = get_web_root();
        let mut sql_file_cache = FileCache::new();
        let file_system = FileSystem::init(&web_root, &db).await;
        sql_file_cache.add_static(
            PathBuf::from("index.sql"),
            ParsedSqlFile::new(&db, include_str!("../index.sql")).await,
        );
        Ok(AppState {
            db,
            all_templates,
            sql_file_cache,
            file_system,
        })
    }
}

pub fn get_web_root() -> PathBuf {
    env::var("WEB_ROOT").map_or_else(
        |_| PathBuf::from(&std::path::Component::CurDir),
        PathBuf::from,
    )
}

pub struct Config {
    pub listen_on: SocketAddr,
}
