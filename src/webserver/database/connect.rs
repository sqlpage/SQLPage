use std::{future::Future, str::FromStr, sync::Arc, time::Duration};

use anyhow::Context;
use sqlx::pool::PoolOptions;
use sqlx::{
    AssertSqlSafe, ColumnIndex, ConnectOptions, Decode, Executor, Row, Type,
    mysql::MySqlConnectOptions, postgres::PgConnectOptions,
};
use sqlx_odbc::OdbcConnectOptions;
use sqlx_sqlserver::MssqlConnectOptions;

use super::{Database, DatabaseDriver, DatabasePool, DbInfo, SupportedDatabase};
use crate::{ON_CONNECT_FILE, ON_RESET_FILE, app_config::AppConfig};

mod sqlite;

impl FromStr for DatabaseDriver {
    type Err = anyhow::Error;

    fn from_str(url: &str) -> anyhow::Result<Self> {
        match url.split_once(':').map(|(scheme, _)| scheme) {
            Some("postgres" | "postgresql") => Ok(Self::Postgres),
            Some("mysql" | "mariadb") => Ok(Self::MySql),
            Some("sqlite") => Ok(Self::Sqlite),
            Some("mssql" | "sqlserver") => Ok(Self::Mssql),
            Some("odbc") => Ok(Self::Odbc),
            _ => {
                let upper = url.to_ascii_uppercase();
                anyhow::ensure!(
                    upper.starts_with("DSN=")
                        || upper.starts_with("DRIVER=")
                        || upper.starts_with("FILEDSN=")
                        || (upper.contains("DRIVER=") && upper.contains(';')),
                    "Unsupported database URL scheme in database_url"
                );
                Ok(Self::Odbc)
            }
        }
    }
}

impl Database {
    pub async fn init(config: &AppConfig) -> anyhow::Result<Self> {
        let url = config.database_url.as_str();
        let kind = url.parse::<DatabaseDriver>()?;
        let password = config.database_password.as_deref();
        let hooks = ConnectionHooks::read(config);
        let mut dbms_name = SupportedDatabase::from(kind).display_name().to_owned();

        let connection = match kind {
            DatabaseDriver::Postgres => {
                let mut options = url.parse::<PgConnectOptions>()?;
                if let Some(password) = password {
                    options = options.password(password);
                }
                DatabasePool::Postgres(connect(options, config, kind, hooks, no_setup).await?)
            }
            DatabaseDriver::MySql => {
                let mut options = url.parse::<MySqlConnectOptions>()?;
                if let Some(password) = password {
                    options = options.password(password);
                }
                DatabasePool::MySql(connect(options, config, kind, hooks, no_setup).await?)
            }
            DatabaseDriver::Sqlite => {
                if password.is_some() {
                    log::warn!("Setting a password for a SQLite database is not supported");
                }
                DatabasePool::Sqlite(
                    connect(
                        sqlite::options(config)?,
                        config,
                        kind,
                        hooks,
                        sqlite::register_functions,
                    )
                    .await?,
                )
            }
            DatabaseDriver::Mssql => {
                let mut url =
                    openidconnect::url::Url::parse(url).context("Invalid SQL Server URL")?;
                if let Some(password) = password {
                    let encoded = percent_encoding::utf8_percent_encode(
                        password,
                        percent_encoding::NON_ALPHANUMERIC,
                    )
                    .to_string();
                    url.set_password(Some(&encoded))
                        .map_err(|()| anyhow::anyhow!("Unable to set the SQL Server password"))?;
                }
                let options = url.as_str().parse::<MssqlConnectOptions>()?;
                DatabasePool::Mssql(connect(options, config, kind, hooks, no_setup).await?)
            }
            DatabaseDriver::Odbc => {
                if password.is_some() {
                    log::warn!(
                        "Setting a password for an ODBC connection is not supported via separate config; include credentials in the DSN or connection string"
                    );
                }
                let mut options = url.parse::<OdbcConnectOptions>()?;
                options
                    .batch_size(config.max_pending_rows.clamp(1, 1024))
                    .max_column_size(None);
                dbms_name = retry_connecting(config, || options.connect())
                    .await?
                    .dbms_name()?;
                DatabasePool::Odbc(connect(options, config, kind, hooks, no_setup).await?)
            }
        };

        log::debug!("Initialized {dbms_name:?} database pool");
        Ok(Self {
            connection,
            info: DbInfo {
                database_type: SupportedDatabase::from_dbms_name(&dbms_name),
                dbms_name,
                kind,
            },
        })
    }
}

type SetupConnection<DB> =
    for<'c> fn(
        &'c mut <DB as sqlx::Database>::Connection,
    ) -> futures_util::future::BoxFuture<'c, Result<(), sqlx::Error>>;

fn no_setup<C>(_: &mut C) -> futures_util::future::BoxFuture<'_, Result<(), sqlx::Error>> {
    Box::pin(std::future::ready(Ok(())))
}

async fn connect<DB>(
    options: <DB::Connection as sqlx::Connection>::Options,
    config: &AppConfig,
    kind: DatabaseDriver,
    hooks: ConnectionHooks,
    setup: SetupConnection<DB>,
) -> anyhow::Result<sqlx::Pool<DB>>
where
    DB: sqlx::Database,
    for<'c> &'c mut DB::Connection: Executor<'c, Database = DB>,
    bool: Type<DB> + for<'r> Decode<'r, DB>,
    usize: ColumnIndex<DB::Row>,
    DB::QueryResult: std::fmt::Debug,
{
    let options = options
        .log_statements(log::LevelFilter::Trace)
        .log_slow_statements(log::LevelFilter::Warn, Duration::from_millis(250));
    if kind != DatabaseDriver::Odbc {
        drop(retry_connecting(config, || options.connect()).await?);
    }
    let ConnectionHooks {
        on_connect,
        on_reset,
    } = hooks;
    PoolOptions::<DB>::new()
        .max_connections(max_connections(config, kind))
        .idle_timeout(config.database_connection_idle_timeout)
        .max_lifetime(config.database_connection_max_lifetime)
        .acquire_timeout(Duration::from_secs_f64(
            config.database_connection_acquire_timeout_seconds,
        ))
        .after_connect(move |conn, _| {
            let sql = on_connect.clone();
            Box::pin(async move {
                setup(conn).await?;
                if let Some(sql) = sql {
                    let result = conn.execute(AssertSqlSafe(sql)).await?;
                    log::debug!(
                        "Finished running connection handler on new connection: {result:?}"
                    );
                }
                Ok(())
            })
        })
        .after_release(move |conn, metadata| {
            let sql = on_reset.clone();
            Box::pin(async move {
                let Some(sql) = sql else {
                    return Ok(true);
                };
                log::trace!("Running the custom SQL connection cleanup handler. {metadata:?}");
                match conn.fetch_optional(AssertSqlSafe(sql)).await? {
                    Some(row) => row.try_get::<bool, _>(0),
                    None => Ok(true),
                }
            })
        })
        .connect_with(options)
        .await
        .with_context(|| format!("Unable to open {kind:?} database pool"))
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
                    return Err(
                        anyhow::Error::new(e).context("Unable to open connection to the database")
                    );
                }
                log::warn!("Failed to connect to the database: {e:#}. Retrying in 5 seconds.");
                retries -= 1;
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

fn max_connections(config: &AppConfig, kind: DatabaseDriver) -> u32 {
    if let Some(max) = config.max_database_pool_connections {
        return max;
    }
    // Different databases have a different number of max concurrent connections allowed by default
    match kind {
        DatabaseDriver::Postgres | DatabaseDriver::Odbc => 50, // Default to PostgreSQL-like limits for Generic
        DatabaseDriver::MySql => 75,
        DatabaseDriver::Sqlite => {
            if config.database_url.contains(":memory:") {
                128
            } else {
                16
            }
        }
        DatabaseDriver::Mssql => 100,
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sqlite_pool(db: &Database) -> &sqlx::SqlitePool {
        let DatabasePool::Sqlite(pool) = &db.connection else {
            panic!("expected SQLite pool");
        };
        pool
    }

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
        let value: String = sqlx::query("SELECT value FROM on_connect_marker")
            .fetch_one(sqlite_pool(&db))
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
        let resets: i64 = sqlx::query("SELECT COUNT(*) FROM reset_log")
            .fetch_one(sqlite_pool(&db))
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
