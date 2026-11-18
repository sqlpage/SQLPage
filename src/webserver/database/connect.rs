use std::{future::Future, mem::take, sync::Arc, time::Duration};

use super::Database;
use crate::{
    ON_CONNECT_FILE, ON_RESET_FILE,
    app_config::AppConfig,
    webserver::database::{DbInfo, SupportedDatabase},
};
use anyhow::Context;
use futures_util::future::BoxFuture;
use sqlx::connection::{ConnectOptions, Connection};
use sqlx::executor::Executor;
use sqlx::odbc::OdbcConnectOptions;
use sqlx::{
    any::{Any, AnyConnectOptions, AnyConnection, AnyKind},
    pool::PoolOptions,
    sqlite::{Function, SqliteConnectOptions, SqliteFunctionCtx},
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
        connect_options.log_slow_statements(log::LevelFilter::Warn, Duration::from_millis(250));
        log::debug!(
            "Connecting to a {:?} database on {}",
            connect_options.kind(),
            database_url
        );
        set_custom_connect_options(&mut connect_options, config);
        log::debug!("Connecting to database: {database_url}");
        let mut conn: AnyConnection =
            retry_connecting(config, || AnyConnection::connect_with(&connect_options)).await?;
        let dbms_name: String = conn.dbms_name().await?;
        let database_type = SupportedDatabase::from_dbms_name(&dbms_name);
        drop(conn);

        let db_kind = connect_options.kind();
        let pool = Self::create_pool_options(config, db_kind)
            .connect_with(connect_options)
            .await
            .with_context(|| format!("Unable to open connection pool to {database_url}"))?;

        log::debug!("Initialized {dbms_name:?} database pool: {pool:#?}");
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
        let ConnectionHooks {
            on_connect,
            on_reset,
        } = ConnectionHooks::read(config);
        let mut pool_options = PoolOptions::new()
            .max_connections(max_connections(config, kind))
            .idle_timeout(config.database_connection_idle_timeout)
            .max_lifetime(config.database_connection_max_lifetime)
            .acquire_timeout(Duration::from_secs_f64(
                config.database_connection_acquire_timeout_seconds,
            ));
        pool_options = add_on_return_to_pool(pool_options, on_reset);
        pool_options = add_on_connection_handler(config, pool_options, on_connect);
        pool_options
    }
}

async fn retry_connecting<C, E, F>(
    config: &AppConfig,
    mut connect: impl FnMut() -> F,
) -> anyhow::Result<C>
where
    F: Future<Output = Result<C, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut retries = config.database_connection_retries;
    loop {
        match connect().await {
            Ok(connection) => return Ok(connection),
            Err(e) => {
                if retries == 0 {
                    return Err(anyhow::Error::new(e).context(format!(
                        "Unable to open connection to {}",
                        config.database_url
                    )));
                }
                log::warn!("Failed to connect to the database: {e:#}. Retrying in 5 seconds.");
                retries -= 1;
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

fn max_connections(config: &AppConfig, kind: AnyKind) -> u32 {
    if let Some(max) = config.max_database_pool_connections {
        return max;
    }
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
}

struct ConnectionHooks {
    on_connect: Option<Arc<str>>,
    on_reset: Option<Arc<str>>,
}

impl ConnectionHooks {
    fn read(config: &AppConfig) -> Self {
        Self {
            on_connect: read_hook(config, ON_CONNECT_FILE, "database connection handler"),
            on_reset: read_hook(config, ON_RESET_FILE, "connection cleanup handler"),
        }
    }
}

fn read_hook(config: &AppConfig, file_name: &str, handler: &str) -> Option<Arc<str>> {
    let hook_file = config.configuration_directory.join(file_name);
    if !hook_file.exists() {
        log::debug!(
            "Not creating a custom SQL {handler} because {} does not exist",
            hook_file.display()
        );
        return None;
    }
    log::info!(
        "Creating a custom SQL {handler} from {}",
        hook_file.display()
    );
    match std::fs::read_to_string(&hook_file) {
        Ok(sql) => {
            log::trace!("The custom SQL {handler} is:\n{sql}");
            Some(Arc::from(sql))
        }
        Err(e) => {
            log::error!("Unable to read the file {}: {}", hook_file.display(), e);
            None
        }
    }
}

fn add_on_return_to_pool(
    pool_options: PoolOptions<Any>,
    sql: Option<Arc<str>>,
) -> PoolOptions<Any> {
    pool_options.after_release(move |conn, meta| {
        let sql = sql.clone();
        Box::pin(async move {
            if let Some(sql) = sql {
                on_return_to_pool(conn, meta, sql).await
            } else {
                Ok(true)
            }
        })
    })
}

fn on_return_to_pool(
    conn: &mut AnyConnection,
    meta: sqlx::pool::PoolConnectionMetadata,
    sql: Arc<str>,
) -> BoxFuture<'_, Result<bool, sqlx::error::Error>> {
    use sqlx::row::Row;
    Box::pin(async move {
        log::trace!("Running the custom SQL connection cleanup handler. {meta:?}");
        let query_result = conn.fetch_optional(&*sql).await?;
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
    sql: Option<Arc<str>>,
) -> PoolOptions<Any> {
    let on_connect_file_display = config
        .configuration_directory
        .join(ON_CONNECT_FILE)
        .display()
        .to_string();
    pool_options.after_connect(move |conn, _| {
        let sql = sql.clone();
        let on_connect_file_display = on_connect_file_display.clone();
        Box::pin(async move {
            if let Some(sql) = sql {
                log::debug!("Running {on_connect_file_display} on new connection");
                let r = conn.execute(&*sql).await?;
                log::debug!("Finished running connection handler on new connection: {r:?}");
            }
            Ok(())
        })
    })
}

fn set_custom_connect_options(options: &mut AnyConnectOptions, config: &AppConfig) {
    if let Some(sqlite_options) = options.as_sqlite_mut() {
        set_custom_connect_options_sqlite(sqlite_options, config);
    }

    if let Some(odbc_options) = options.as_odbc_mut() {
        set_custom_connect_options_odbc(odbc_options, config);
    }
}

fn set_custom_connect_options_sqlite(
    sqlite_options: &mut SqliteConnectOptions,
    config: &AppConfig,
) {
    for extension_name in &config.sqlite_extensions {
        log::info!("Loading SQLite extension: {extension_name}");
        *sqlite_options = take(sqlite_options).extension(extension_name.clone());
    }
    *sqlite_options = take(sqlite_options)
        .collation("NOCASE", |a, b| a.to_lowercase().cmp(&b.to_lowercase()))
        .function(make_sqlite_fun("upper", str::to_uppercase))
        .function(make_sqlite_fun("lower", str::to_lowercase));
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

fn set_custom_connect_options_odbc(odbc_options: &mut OdbcConnectOptions, config: &AppConfig) {
    // Allow fetching very large text fields when using ODBC by removing the max column size limit
    let batch_size = config.max_pending_rows.clamp(1, 1024);
    odbc_options.batch_size(batch_size);
    log::trace!("ODBC batch size set to {batch_size}");
    // Disables ODBC batching, but avoids truncation of large text fields
    odbc_options.max_column_size(None);
}

fn set_database_password(options: &mut AnyConnectOptions, password: &str) {
    if let Some(opts) = options.as_postgres_mut() {
        *opts = take(opts).password(password);
    } else if let Some(opts) = options.as_mysql_mut() {
        *opts = take(opts).password(password);
    } else if let Some(opts) = options.as_mssql_mut() {
        *opts = take(opts).password(password);
    } else if let Some(_opts) = options.as_odbc_mut() {
        log::warn!(
            "Setting a password for an ODBC connection is not supported via separate config; include credentials in the DSN or connection string"
        );
    } else if let Some(_opts) = options.as_sqlite_mut() {
        log::warn!("Setting a password for a SQLite database is not supported");
    } else {
        unreachable!("Unsupported database type");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::row::Row;
    use tempfile::TempDir;

    fn config_in(dir: &TempDir, database_url: &str) -> AppConfig {
        let mut config = crate::app_config::tests::test_config();
        config.database_url = database_url.to_owned();
        config.configuration_directory = dir.path().to_path_buf();
        config.max_database_pool_connections = Some(1);
        config
    }

    #[actix_web::test]
    async fn on_connect_sql_runs_on_new_pool_connections() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join(ON_CONNECT_FILE),
            "CREATE TEMPORARY TABLE on_connect_marker(value TEXT);
             INSERT INTO on_connect_marker(value) VALUES ('on_connect ran');",
        )
        .unwrap();

        let db = Database::init(&config_in(&dir, "sqlite::memory:"))
            .await
            .unwrap();
        let value: String = sqlx::query::query("SELECT value FROM on_connect_marker")
            .fetch_one(&db.connection)
            .await
            .unwrap()
            .try_get(0)
            .unwrap();

        assert_eq!(value, "on_connect ran");
    }

    #[actix_web::test]
    async fn on_reset_sql_runs_when_a_connection_returns_to_the_pool() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join(ON_CONNECT_FILE),
            "CREATE TABLE IF NOT EXISTS reset_log(id INTEGER);",
        )
        .unwrap();
        std::fs::write(
            dir.path().join(ON_RESET_FILE),
            "INSERT INTO reset_log(id) VALUES (1);
             SELECT 1 AS is_healthy;",
        )
        .unwrap();
        let db_file = dir.path().join("on_reset.db");
        let config = config_in(&dir, &format!("sqlite://{}?mode=rwc", db_file.display()));

        let db = Database::init(&config).await.unwrap();
        drop(db.connection.acquire().await.unwrap());
        let resets: i64 = sqlx::query::query("SELECT COUNT(*) FROM reset_log")
            .fetch_one(&db.connection)
            .await
            .unwrap()
            .try_get(0)
            .unwrap();

        assert!(resets > 0);
    }

    #[tokio::test(start_paused = true)]
    async fn connection_retries_wait_five_seconds_before_giving_up() {
        for (retries, expected_wait) in [(0, Duration::ZERO), (2, Duration::from_secs(10))] {
            let dir = TempDir::new().unwrap();
            let missing = dir.path().join("nonexistent_directory").join("db.sqlite");
            let mut config = config_in(&dir, &format!("sqlite://{}", missing.display()));
            config.database_connection_retries = retries;

            let start = tokio::time::Instant::now();
            let Err(error) = Database::init(&config).await else {
                panic!("connecting to a missing database must fail");
            };

            assert_eq!(start.elapsed(), expected_wait, "{retries} retries");
            assert!(
                format!("{error:#}").contains("Unable to open connection to"),
                "{error:#}"
            );
        }
    }
}
