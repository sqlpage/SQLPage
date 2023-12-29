use std::{sync::Arc, time::Duration};

use super::Database;
use crate::{app_config::AppConfig, ON_CONNECT_FILE};
use futures_util::future::BoxFuture;
use sqlx::{
    any::{Any, AnyConnectOptions, AnyKind},
    pool::PoolOptions,
    sqlite, AnyConnection, ConnectOptions, Executor,
};

impl Database {
    pub async fn init(config: &AppConfig) -> anyhow::Result<Self> {
        let database_url = &config.database_url;
        let mut connect_options: AnyConnectOptions =
            database_url.parse().expect("Invalid database URL");
        connect_options.log_statements(log::LevelFilter::Trace);
        connect_options.log_slow_statements(
            log::LevelFilter::Warn,
            std::time::Duration::from_millis(250),
        );
        log::debug!(
            "Connecting to a {:?} database on {}",
            connect_options.kind(),
            database_url
        );
        set_custom_connect_options(&mut connect_options, config);
        log::info!("Connecting to database: {database_url}");
        let mut retries = config.database_connection_retries;
        let connection = loop {
            match Self::create_pool_options(config, connect_options.kind())
                .connect_with(connect_options.clone())
                .await
            {
                Ok(c) => break c,
                Err(e) => {
                    if retries == 0 {
                        return Err(anyhow::Error::new(e)
                            .context(format!("Unable to open connection to {database_url}")));
                    }
                    log::warn!("Failed to connect to the database: {e:#}. Retrying in 5 seconds.");
                    retries -= 1;
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        };
        log::debug!("Initialized database pool: {connection:#?}");
        Ok(Database { connection })
    }

    fn create_pool_options(config: &AppConfig, db_kind: AnyKind) -> PoolOptions<Any> {
        let mut pool_options = PoolOptions::new()
            .max_connections(if let Some(max) = config.max_database_pool_connections {
                max
            } else {
                // Different databases have a different number of max concurrent connections allowed by default
                match db_kind {
                    AnyKind::Postgres => 50,
                    AnyKind::MySql => 75,
                    AnyKind::Sqlite => {
                        if config.database_url.contains(":memory:") {
                            // Create no more than a single in-memory database connection
                            1
                        } else {
                            16
                        }
                    }
                    AnyKind::Mssql => 100,
                }
            })
            .idle_timeout(
                config
                    .database_connection_idle_timeout_seconds
                    .map(Duration::from_secs_f64)
                    .or_else(|| match db_kind {
                        AnyKind::Sqlite => None,
                        _ => Some(Duration::from_secs(30 * 60)),
                    }),
            )
            .max_lifetime(
                config
                    .database_connection_max_lifetime_seconds
                    .map(Duration::from_secs_f64)
                    .or_else(|| match db_kind {
                        AnyKind::Sqlite => None,
                        _ => Some(Duration::from_secs(60 * 60)),
                    }),
            )
            .acquire_timeout(Duration::from_secs_f64(
                config.database_connection_acquire_timeout_seconds,
            ));
        pool_options = add_on_connection_handler(pool_options);
        pool_options
    }
}

fn add_on_connection_handler(pool_options: PoolOptions<Any>) -> PoolOptions<Any> {
    let on_connect_sql = on_connection_handler_file();
    pool_options.after_connect(move |conn, _metadata| {
        let on_connect_sql = on_connect_sql.clone();
        Box::pin(async move {
            if let Some(sql) = (&on_connect_sql).as_ref() {
                log::debug!("Running {ON_CONNECT_FILE:?} on new connection");
                let sql = std::sync::Arc::clone(&sql);
                let r = conn.execute(sql.as_str()).await?;
                log::debug!("Finished running connection handler on new connection: {r:?}");
            }
            if let sqlx::any::AnyConnectionKind::Sqlite(sqlite_conn) = conn.private_get_mut() {
                sqlite_on_connection_handler(sqlite_conn).await?;
            }
            Ok(())
        })
    })
}

async fn sqlite_on_connection_handler(
    sqlite_conn: &mut sqlx::sqlite::SqliteConnection,
) -> sqlx::Result<()> {
    let handle = sqlite_conn.lock_handle().await?;
    log::warn!("TODO: bind sqlite functions");
    Ok(())
}

fn on_connection_handler_file() -> Option<Arc<String>> {
    let on_connect_file = std::env::current_dir()
        .unwrap_or_default()
        .join(ON_CONNECT_FILE);
    if !on_connect_file.exists() {
        log::debug!("Not creating a custom SQL database connection handler because {on_connect_file:?} does not exist");
        return None;
    }
    log::info!("Creating a custom SQL database connection handler from {on_connect_file:?}");
    let sql = match std::fs::read_to_string(&on_connect_file) {
        Ok(sql) => std::sync::Arc::new(sql),
        Err(e) => {
            log::error!("Unable to read the file {on_connect_file:?}: {e}");
            return None;
        }
    };
    log::trace!("The custom SQL database connection handler is:\n{sql}");
    Some(sql)
}

fn set_custom_connect_options(options: &mut AnyConnectOptions, config: &AppConfig) {
    if let Some(sqlite_options) = options.as_sqlite_mut() {
        for extension_name in &config.sqlite_extensions {
            log::info!("Loading SQLite extension: {}", extension_name);
            *sqlite_options = std::mem::take(sqlite_options).extension(extension_name.clone());
        }
        *sqlite_options = std::mem::take(sqlite_options).collation("NOCASE", sqlite_collate_nocase);
        *sqlite_options =
            std::mem::take(sqlite_options).thread_name(|i| format!("sqlpage_sqlite_{}", i));
    }
}

fn sqlite_collate_nocase(a: &str, b: &str) -> std::cmp::Ordering {
    a.to_lowercase().cmp(&b.to_lowercase())
}
