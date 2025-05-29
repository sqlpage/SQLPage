#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

//! [SQLPage](https://sql-page.com) is a high-performance web server that converts SQL queries
//! into dynamic web applications by rendering [handlebars templates](https://sql-page.com/custom_components.sql)
//! with data coming from SQL queries declared in `.sql` files.
//!
//! # Overview
//!
//! `SQLPage` is a web server that lets you build data-centric applications using only SQL queries.
//! It automatically converts database queries into professional-looking web pages using pre-built components
//! for common UI patterns like [tables](https://sql-page.com/component.sql?component=table),
//! [charts](https://sql-page.com/component.sql?component=chart),
//! [forms](https://sql-page.com/component.sql?component=form), and more.
//!
//! # Key Features
//!
//! - **SQL-Only Development**: Build full web applications without HTML, CSS, or JavaScript
//! - **Built-in Components**: Rich library of [pre-made UI components](https://sql-page.com/documentation.sql)
//! - **Security**: Protection against [SQL injection, XSS and other vulnerabilities](https://sql-page.com/safety.sql)
//! - **Performance**: [Optimized request handling and rendering](https://sql-page.com/performance.sql)
//! - **Database Support**: Works with `SQLite`, `PostgreSQL`, `MySQL`, and MS SQL Server
//!
//! # Architecture
//!
//! The crate is organized into several key modules:
//!
//! - [`webserver`]: Core HTTP server implementation using actix-web
//! - [`render`]: Component rendering system, streaming rendering of the handlebars templates with data
//! - [`templates`]: Pre-defined UI component definitions
//! - [`file_cache`]: Caching layer for SQL file parsing
//! - [`filesystem`]: Abstract interface for disk and DB-stored files
//! - [`app_config`]: Configuration and environment handling
//!
//! # Query Processing Pipeline
//!
//! When processing a request, `SQLPage`:
//!
//! 1. Parses the SQL using sqlparser-rs. Once a SQL file is parsed, it is cached for later reuse.
//! 2. Executes queries through sqlx.
//! 3. Finds the requested component's handlebars template in the database or in the filesystem.
//! 4. Maps results to the component template, using handlebars-rs.
//! 5. Streams rendered HTML to the client.
//!
//! # Extended Functionality
//!
//! - [Custom SQL Functions](https://sql-page.com/functions.sql)
//! - [Custom Components](https://sql-page.com/custom_components.sql)
//! - [Authentication & Sessions](https://sql-page.com/examples/authentication)
//! - [File Uploads](https://sql-page.com/examples/handle_picture_upload.sql)
//!
//! # Example
//!
//! ```sql
//! -- Open a data list component
//! SELECT 'list' as component, 'Users' as title;
//!
//! -- Populate it with data
//! SELECT
//!     name as title,
//!     email as description
//! FROM users
//! ORDER BY created_at DESC;
//! ```
//!
//! For more examples and documentation, visit:
//! - [Getting Started Guide](https://sql-page.com/get%20started.sql)
//! - [Component Reference](https://sql-page.com/components.sql)
//! - [Example Gallery](https://sql-page.com/examples/tabs)

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
use crate::webserver::oidc::OidcState;
use file_cache::FileCache;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use templates::AllTemplates;
use webserver::Database;

/// `TEMPLATES_DIR` is the directory where .handlebars files are stored
/// When a template is requested, it is looked up in `sqlpage/templates/component_name.handlebars` in the database,
/// or in `$SQLPAGE_CONFIGURATION_DIRECTORY/templates/component_name.handlebars` in the filesystem.
pub const TEMPLATES_DIR: &str = "sqlpage/templates/";
pub const MIGRATIONS_DIR: &str = "migrations";
pub const ON_CONNECT_FILE: &str = "on_connect.sql";
pub const ON_RESET_FILE: &str = "on_reset.sql";

pub struct AppState {
    pub db: Database,
    all_templates: AllTemplates,
    sql_file_cache: FileCache<ParsedSqlFile>,
    file_system: FileSystem,
    config: AppConfig,
    pub oidc_state: Option<Arc<OidcState>>,
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
            ParsedSqlFile::new(&db, include_str!("../index.sql"), Path::new("index.sql")),
        );
        sql_file_cache.add_static(
            PathBuf::from("_default_404.sql"),
            ParsedSqlFile::new(
                &db,
                include_str!("../_default_404.sql"),
                Path::new("_default_404.sql"),
            ),
        );

        let oidc_state = crate::webserver::oidc::initialize_oidc_state(config).await?;

        Ok(AppState {
            db,
            all_templates,
            sql_file_cache,
            file_system,
            config: config.clone(),
            oidc_state,
        })
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish()
    }
}
