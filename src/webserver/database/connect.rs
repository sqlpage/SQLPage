use std::{mem::take, time::Duration};

use super::Database;
use crate::{
    app_config::AppConfig,
    webserver::database::{DbInfo, SupportedDatabase},
    ON_CONNECT_FILE, ON_RESET_FILE,
};
use anyhow::Context;
use futures_util::future::BoxFuture;
use sqlx::{
    any::{Any, AnyConnectOptions, AnyKind},
    pool::PoolOptions,
    sqlite::{Function, SqliteFunctionCtx},
    ConnectOptions, Connection, Executor,
};

impl Database {
    pub async fn init(config: &AppConfig) -> anyhow::Result<Self> {
        let database_url = &config.database_url;
        let mut connect_options: AnyConnectOptions = database_url
            .parse()
            .with_context(|| format!("\"{database_url}\" is not a valid database URL. Please change the \"database_url\" option in the configuration file."))?;
        if let Some(password) = &config.database_password {
            set_database_password(&mut connect_options, password);
        }
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
        log::debug!("Connecting to database: {database_url}");
        let mut retries = config.database_connection_retries;
        let db_kind = connect_options.kind();
        let pool = loop {
            match Self::create_pool_options(config, db_kind)
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
        let dbms_name: String = pool.acquire().await?.dbms_name().await?;
        let database_type = SupportedDatabase::from_dbms_name(&dbms_name);

        log::debug!("Initialized {dbms_name} database pool: {pool:#?}");
        Ok(Database {
            connection: pool,
            info: DbInfo {
                dbms_name,
                database_type,
                kind: db_kind,
            },
        })
    }

    fn create_pool_options(config: &AppConfig, kind: AnyKind) -> PoolOptions<Any> {
        let mut pool_options = PoolOptions::new()
            .max_connections(if let Some(max) = config.max_database_pool_connections {
                max
            } else {
                // Different databases have a different number of max concurrent connections allowed by default
                match kind {
                    AnyKind::Postgres | AnyKind::Odbc => 50, // Default to PostgreSQL-like limits for Generic
                    AnyKind::MySql => 75,
                    AnyKind::Sqlite => {
                        if config.database_url.contains(":memory:") {
                            128
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
                    .or_else(|| match kind {
                        AnyKind::Sqlite => None,
                        _ => Some(Duration::from_secs(30 * 60)),
                    }),
            )
            .max_lifetime(
                config
                    .database_connection_max_lifetime_seconds
                    .map(Duration::from_secs_f64)
                    .or_else(|| match kind {
                        AnyKind::Sqlite => None,
                        _ => Some(Duration::from_secs(60 * 60)),
                    }),
            )
            .acquire_timeout(Duration::from_secs_f64(
                config.database_connection_acquire_timeout_seconds,
            ));
        pool_options = add_on_return_to_pool(config, pool_options);
        pool_options = add_on_connection_handler(config, pool_options);
        pool_options
    }
}

fn add_on_return_to_pool(config: &AppConfig, pool_options: PoolOptions<Any>) -> PoolOptions<Any> {
    let on_disconnect_file = config.configuration_directory.join(ON_RESET_FILE);
    if !on_disconnect_file.exists() {
        log::debug!(
            "Not creating a custom SQL connection cleanup handler because {} does not exist",
            on_disconnect_file.display()
        );
        return pool_options;
    }
    log::info!(
        "Creating a custom SQL connection cleanup handler from {}",
        on_disconnect_file.display()
    );
    let sql = match std::fs::read_to_string(&on_disconnect_file) {
        Ok(sql) => std::sync::Arc::new(sql),
        Err(e) => {
            log::error!(
                "Unable to read the file {}: {}",
                on_disconnect_file.display(),
                e
            );
            return pool_options;
        }
    };
    log::trace!("The custom SQL connection cleanup handler is:\n{sql}");
    let sql = sql.clone();
    pool_options
        .after_release(move |conn, meta| on_return_to_pool(conn, meta, std::sync::Arc::clone(&sql)))
}

fn on_return_to_pool(
    conn: &mut sqlx::AnyConnection,
    meta: sqlx::pool::PoolConnectionMetadata,
    sql: std::sync::Arc<String>,
) -> BoxFuture<'_, Result<bool, sqlx::Error>> {
    use sqlx::Row;
    Box::pin(async move {
        log::trace!("Running the custom SQL connection cleanup handler. {meta:?}");
        let query_result = conn.fetch_optional(sql.as_str()).await?;
        if let Some(query_result) = query_result {
            let is_healthy = query_result.try_get::<bool, _>(0);
            log::debug!("Is the connection healthy? {is_healthy:?}");
            is_healthy
        } else {
            log::debug!("No result from the custom SQL connection cleanup handler");
            Ok(true)
        }
    })
}

fn add_on_connection_handler(
    config: &AppConfig,
    pool_options: PoolOptions<Any>,
) -> PoolOptions<Any> {
    let on_connect_file = config.configuration_directory.join(ON_CONNECT_FILE);
    if !on_connect_file.exists() {
        log::debug!(
            "Not creating a custom SQL database connection handler because {} does not exist",
            on_connect_file.display()
        );
        return pool_options;
    }
    log::info!(
        "Creating a custom SQL database connection handler from {}",
        on_connect_file.display()
    );
    let sql = match std::fs::read_to_string(&on_connect_file) {
        Ok(sql) => std::sync::Arc::new(sql),
        Err(e) => {
            log::error!(
                "Unable to read the file {}: {}",
                on_connect_file.display(),
                e
            );
            return pool_options;
        }
    };
    log::trace!("The custom SQL database connection handler is:\n{sql}");
    pool_options.after_connect(move |conn, _metadata| {
        log::debug!("Running {} on new connection", on_connect_file.display());
        let sql = std::sync::Arc::clone(&sql);
        Box::pin(async move {
            let r = conn.execute(sql.as_str()).await?;
            log::debug!("Finished running connection handler on new connection: {r:?}");
            Ok(())
        })
    })
}

fn set_custom_connect_options(options: &mut AnyConnectOptions, config: &AppConfig) {
    if let Some(sqlite_options) = options.as_sqlite_mut() {
        for extension_name in &config.sqlite_extensions {
            log::info!("Loading SQLite extension: {extension_name}");
            *sqlite_options = std::mem::take(sqlite_options).extension(extension_name.clone());
        }
        *sqlite_options = std::mem::take(sqlite_options)
            .collation("NOCASE", |a, b| a.to_lowercase().cmp(&b.to_lowercase()))
            .function(make_sqlite_fun("upper", str::to_uppercase))
            .function(make_sqlite_fun("lower", str::to_lowercase));
    }
}

fn make_sqlite_fun(name: &str, f: fn(&str) -> String) -> Function {
    Function::new(name, move |ctx: &SqliteFunctionCtx| {
        let arg = ctx.try_get_arg::<Option<&str>>(0);
        match arg {
            Ok(Some(s)) => ctx.set_result(f(s)),
            Ok(None) => ctx.set_result(None::<String>),
            Err(e) => ctx.set_error(&e.to_string()),
        }
    })
}

fn set_database_password(options: &mut AnyConnectOptions, password: &str) {
    if let Some(opts) = options.as_postgres_mut() {
        *opts = take(opts).password(password);
    } else if let Some(opts) = options.as_mysql_mut() {
        *opts = take(opts).password(password);
    } else if let Some(opts) = options.as_mssql_mut() {
        *opts = take(opts).password(password);
    } else if let Some(_opts) = options.as_sqlite_mut() {
        log::warn!("Setting a password for a SQLite database is not supported");
    } else {
        unreachable!("Unsupported database type");
    }
}
