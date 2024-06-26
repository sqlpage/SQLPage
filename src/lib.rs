#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

extern crate core;

pub mod app_config;
pub mod dynamic_component;
pub mod file_cache;
pub mod filesystem;
pub mod render;
pub mod template_helpers;
pub mod templates;
pub mod utils;
pub mod webserver;

use crate::app_config::AppConfig;
use crate::filesystem::FileSystem;
use crate::webserver::database::ParsedSqlFile;
use file_cache::FileCache;
use std::path::PathBuf;
use templates::AllTemplates;
use webserver::Database;

/// `TEMPLATES_DIR` is the directory where .handlebars files are stored
/// When a template is requested, it is looked up in `sqlpage/templates/component_name.handlebars` in the database,
/// or in `$SQLPAGE_CONFIGURATION_DIRECTORY/templates/component_name.handlebars` in the filesystem.
pub const TEMPLATES_DIR: &str = "sqlpage/templates/";
pub const MIGRATIONS_DIR: &str = "migrations";
pub const ON_CONNECT_FILE: &str = "on_connect.sql";

pub struct AppState {
    pub db: Database,
    all_templates: AllTemplates,
    sql_file_cache: FileCache<ParsedSqlFile>,
    file_system: FileSystem,
    config: AppConfig,
}

impl AppState {
    pub async fn init(config: &AppConfig) -> anyhow::Result<Self> {
        let db = Database::init(config).await?;
        Self::init_with_db(config, db).await
    }
    pub async fn init_with_db(config: &AppConfig, db: Database) -> anyhow::Result<Self> {
        let all_templates = AllTemplates::init(config)?;
        let mut sql_file_cache = FileCache::new();
        let file_system = FileSystem::init(&config.web_root, &db).await;
        sql_file_cache.add_static(
            PathBuf::from("index.sql"),
            ParsedSqlFile::new(&db, include_str!("../index.sql")),
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
