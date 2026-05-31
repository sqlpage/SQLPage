use std::{ffi::CString, time::Duration};

use super::{Database, DatabasePool, DbInfo, DbKind, SupportedDatabase};
use crate::{ON_CONNECT_FILE, ON_RESET_FILE, app_config::AppConfig};
use anyhow::Context;
use futures_util::future::BoxFuture;
use sqlx::{
    ColumnIndex, ConnectOptions, Connection, Database as SqlxDatabase, Decode, Executor, Row, Type,
    mysql::MySqlConnectOptions, pool::PoolOptions, postgres::PgConnectOptions,
    sqlite::SqliteConnectOptions,
};
use sqlx_odbc::{OdbcConnectOptions, OdbcConnection};
use sqlx_sqlserver::MssqlConnectOptions;
use url::Url;

impl Database {
    #[allow(clippy::too_many_lines)]
    pub async fn init(config: &AppConfig) -> anyhow::Result<Self> {
        let database_url = database_url_with_password(config)?;
        let db_kind = DbKind::from_database_url(&database_url);
        log::debug!(
            "Connecting to a {db_kind:?} database on {}",
            config.database_url
        );

        let connection = match db_kind {
            DbKind::Sqlite => {
                let mut options = database_url.parse::<SqliteConnectOptions>()?;
                options = set_common_connect_options(options);
                options = set_custom_connect_options_sqlite(options, config);
                let pool = Self::create_sqlite_pool_options(config, db_kind)
                    .connect_with(options)
                    .await
                    .with_context(|| {
                        format!("Unable to open connection pool to {}", config.database_url)
                    })?;
                DatabasePool::Sqlite(pool)
            }
            DbKind::Postgres => {
                let mut options = database_url.parse::<PgConnectOptions>()?;
                if let Some(password) = &config.database_password {
                    options = options.password(password);
                }
                options = set_common_connect_options(options);
                let pool = Self::create_pool_options::<sqlx::Postgres>(config, db_kind)
                    .connect_with(options)
                    .await
                    .with_context(|| {
                        format!("Unable to open connection pool to {}", config.database_url)
                    })?;
                DatabasePool::Postgres(pool)
            }
            DbKind::MySql => {
                let mut options = database_url.parse::<MySqlConnectOptions>()?;
                if let Some(password) = &config.database_password {
                    options = options.password(password);
                }
                options = set_common_connect_options(options);
                let pool = Self::create_pool_options::<sqlx::MySql>(config, db_kind)
                    .connect_with(options)
                    .await
                    .with_context(|| {
                        format!("Unable to open connection pool to {}", config.database_url)
                    })?;
                DatabasePool::MySql(pool)
            }
            DbKind::Mssql => {
                let options = set_common_connect_options(
                    database_url
                        .parse::<MssqlConnectOptions>()
                        .with_context(|| format!("Unable to parse {}", config.database_url))?,
                );
                let pool = Self::create_pool_options::<sqlx_sqlserver::Mssql>(config, db_kind)
                    .connect_with(options)
                    .await
                    .with_context(|| {
                        format!("Unable to open connection pool to {}", config.database_url)
                    })?;
                DatabasePool::Mssql(pool)
            }
            DbKind::Odbc => {
                if config.database_password.is_some() {
                    log::warn!(
                        "Setting a password for an ODBC connection is not supported via separate config; include credentials in the DSN or connection string"
                    );
                }
                let mut options = database_url.parse::<OdbcConnectOptions>()?;
                set_custom_connect_options_odbc(&mut options, config);
                let dbms_name = detect_odbc_dbms_name(&options, config).await?;
                let database_type = SupportedDatabase::from_dbms_name(&dbms_name);
                let options = set_common_connect_options(options);
                let pool = Self::create_pool_options::<sqlx_odbc::Odbc>(config, db_kind)
                    .connect_with(options)
                    .await
                    .with_context(|| {
                        format!("Unable to open connection pool to {}", config.database_url)
                    })?;
                log::debug!("Initialized {dbms_name:?} database pool: {pool:#?}");
                return Ok(Database {
                    connection: DatabasePool::Odbc(pool),
                    info: DbInfo {
                        dbms_name,
                        database_type,
                        kind: db_kind,
                    },
                });
            }
        };

        let dbms_name = db_kind.display_name().to_owned();
        let database_type = SupportedDatabase::from(db_kind);
        log::debug!("Initialized {dbms_name:?} database pool: {connection:#?}");
        Ok(Database {
            connection,
            info: DbInfo {
                dbms_name,
                database_type,
                kind: db_kind,
            },
        })
    }

    fn create_pool_options<DB>(config: &AppConfig, kind: DbKind) -> PoolOptions<DB>
    where
        DB: SqlxDatabase,
        for<'c> &'c mut DB::Connection: Executor<'c, Database = DB>,
        for<'r> bool: Decode<'r, DB> + Type<DB>,
        usize: ColumnIndex<DB::Row>,
    {
        let max_connections = config
            .max_database_pool_connections
            .unwrap_or_else(|| default_max_connections(config, kind));
        let pool_options = PoolOptions::new()
            .max_connections(max_connections)
            .idle_timeout(config.database_connection_idle_timeout)
            .max_lifetime(config.database_connection_max_lifetime)
            .acquire_timeout(Duration::from_secs_f64(
                config.database_connection_acquire_timeout_seconds,
            ));
        let pool_options = add_on_return_to_pool(config, pool_options);
        add_on_connection_handler(config, pool_options)
    }

    fn create_sqlite_pool_options(config: &AppConfig, kind: DbKind) -> PoolOptions<sqlx::Sqlite> {
        let max_connections = config
            .max_database_pool_connections
            .unwrap_or_else(|| default_max_connections(config, kind));
        let pool_options = PoolOptions::new()
            .max_connections(max_connections)
            .idle_timeout(config.database_connection_idle_timeout)
            .max_lifetime(config.database_connection_max_lifetime)
            .acquire_timeout(Duration::from_secs_f64(
                config.database_connection_acquire_timeout_seconds,
            ));
        let pool_options = add_on_return_to_pool(config, pool_options);
        add_sqlite_on_connection_handler(config, pool_options)
    }
}

fn default_max_connections(config: &AppConfig, kind: DbKind) -> u32 {
    match kind {
        DbKind::Postgres | DbKind::Odbc => 50,
        DbKind::MySql => 75,
        DbKind::Sqlite => {
            if config.database_url.contains(":memory:") {
                128
            } else {
                16
            }
        }
        DbKind::Mssql => 100,
    }
}

fn set_common_connect_options<T>(options: T) -> T
where
    T: ConnectOptions,
{
    options
        .log_statements(log::LevelFilter::Trace)
        .log_slow_statements(log::LevelFilter::Warn, Duration::from_millis(250))
}

fn add_on_return_to_pool<DB>(config: &AppConfig, pool_options: PoolOptions<DB>) -> PoolOptions<DB>
where
    DB: SqlxDatabase,
    for<'c> &'c mut DB::Connection: Executor<'c, Database = DB>,
    for<'r> bool: Decode<'r, DB> + Type<DB>,
    usize: ColumnIndex<DB::Row>,
{
    let sql = read_optional_handler_sql(config, ON_RESET_FILE, "connection cleanup");

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

fn on_return_to_pool<DB>(
    conn: &mut DB::Connection,
    meta: sqlx::pool::PoolConnectionMetadata,
    sql: std::sync::Arc<String>,
) -> BoxFuture<'_, Result<bool, sqlx::Error>>
where
    DB: SqlxDatabase,
    for<'c> &'c mut DB::Connection: Executor<'c, Database = DB>,
    for<'r> bool: Decode<'r, DB> + Type<DB>,
    usize: ColumnIndex<DB::Row>,
{
    Box::pin(async move {
        log::trace!("Running the custom SQL connection cleanup handler. {meta:?}");
        let query_result = conn
            .fetch_optional(sqlx::AssertSqlSafe(sql.as_str()))
            .await?;
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

fn add_on_connection_handler<DB>(
    config: &AppConfig,
    pool_options: PoolOptions<DB>,
) -> PoolOptions<DB>
where
    DB: SqlxDatabase,
    for<'c> &'c mut DB::Connection: Executor<'c, Database = DB>,
{
    let sql = read_optional_handler_sql(config, ON_CONNECT_FILE, "database connection");
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
                conn.execute(sqlx::AssertSqlSafe(sql.as_str())).await?;
                log::debug!("Finished running connection handler on new connection");
            }
            Ok(())
        })
    })
}

fn add_sqlite_on_connection_handler(
    config: &AppConfig,
    pool_options: PoolOptions<sqlx::Sqlite>,
) -> PoolOptions<sqlx::Sqlite> {
    let sql = read_optional_handler_sql(config, ON_CONNECT_FILE, "database connection");
    let on_connect_file_display = config
        .configuration_directory
        .join(ON_CONNECT_FILE)
        .display()
        .to_string();

    pool_options.after_connect(move |conn, _| {
        let sql = sql.clone();
        let on_connect_file_display = on_connect_file_display.clone();
        Box::pin(async move {
            install_sqlite_unicode_functions(conn).await?;
            if let Some(sql) = sql {
                log::debug!("Running {on_connect_file_display} on new connection");
                conn.execute(sqlx::AssertSqlSafe(sql.as_str())).await?;
                log::debug!("Finished running connection handler on new connection");
            }
            Ok(())
        })
    })
}

fn read_optional_handler_sql(
    config: &AppConfig,
    file_name: &str,
    handler_name: &str,
) -> Option<std::sync::Arc<String>> {
    let handler_file = config.configuration_directory.join(file_name);
    if handler_file.exists() {
        log::info!(
            "Creating a custom SQL {handler_name} handler from {}",
            handler_file.display()
        );
        match std::fs::read_to_string(&handler_file) {
            Ok(sql) => {
                log::trace!("The custom SQL {handler_name} handler is:\n{sql}");
                Some(std::sync::Arc::new(sql))
            }
            Err(e) => {
                log::error!("Unable to read the file {}: {}", handler_file.display(), e);
                None
            }
        }
    } else {
        log::debug!(
            "Not creating a custom SQL {handler_name} handler because {} does not exist",
            handler_file.display()
        );
        None
    }
}

fn set_custom_connect_options_sqlite(
    mut sqlite_options: SqliteConnectOptions,
    config: &AppConfig,
) -> SqliteConnectOptions {
    for extension_name in &config.sqlite_extensions {
        log::info!("Loading SQLite extension: {extension_name}");
        // SAFETY: SQLPage has always treated `sqlite_extensions` as an explicit administrator
        // opt-in to load trusted native extensions from the filesystem.
        sqlite_options = unsafe { sqlite_options.extension(extension_name.clone()) };
    }
    sqlite_options.collation("NOCASE", |a, b| a.to_lowercase().cmp(&b.to_lowercase()))
}

async fn install_sqlite_unicode_functions(conn: &mut sqlx::SqliteConnection) -> sqlx::Result<()> {
    let mut handle = conn.lock_handle().await?;
    let sqlite = handle.as_raw_handle().as_ptr();
    register_sqlite_function(sqlite, b"upper\0", sqlite_upper)?;
    register_sqlite_function(sqlite, b"lower\0", sqlite_lower)?;
    Ok(())
}

fn register_sqlite_function(
    sqlite: *mut libsqlite3_sys::sqlite3,
    name: &'static [u8],
    function: unsafe extern "C" fn(
        *mut libsqlite3_sys::sqlite3_context,
        i32,
        *mut *mut libsqlite3_sys::sqlite3_value,
    ),
) -> sqlx::Result<()> {
    let result = unsafe {
        libsqlite3_sys::sqlite3_create_function_v2(
            sqlite,
            name.as_ptr().cast(),
            1,
            libsqlite3_sys::SQLITE_UTF8 | libsqlite3_sys::SQLITE_DETERMINISTIC,
            std::ptr::null_mut(),
            Some(function),
            None,
            None,
            None,
        )
    };
    if result == libsqlite3_sys::SQLITE_OK {
        Ok(())
    } else {
        Err(sqlx::Error::Protocol(format!(
            "sqlite3_create_function_v2 failed with code {result}"
        )))
    }
}

unsafe extern "C" fn sqlite_upper(
    ctx: *mut libsqlite3_sys::sqlite3_context,
    n_arg: i32,
    args: *mut *mut libsqlite3_sys::sqlite3_value,
) {
    sqlite_case_function(ctx, n_arg, args, str::to_uppercase);
}

unsafe extern "C" fn sqlite_lower(
    ctx: *mut libsqlite3_sys::sqlite3_context,
    n_arg: i32,
    args: *mut *mut libsqlite3_sys::sqlite3_value,
) {
    sqlite_case_function(ctx, n_arg, args, str::to_lowercase);
}

fn sqlite_case_function(
    ctx: *mut libsqlite3_sys::sqlite3_context,
    n_arg: i32,
    args: *mut *mut libsqlite3_sys::sqlite3_value,
    f: fn(&str) -> String,
) {
    if n_arg != 1 {
        unsafe {
            libsqlite3_sys::sqlite3_result_error_code(
                ctx,
                libsqlite3_sys::SQLITE_CONSTRAINT_FUNCTION,
            );
        }
        return;
    }
    let arg = unsafe { *args };
    let Some(input) = sqlite_value_text(arg) else {
        unsafe { libsqlite3_sys::sqlite3_result_null(ctx) };
        return;
    };
    let output = f(&input);
    match CString::new(output) {
        Ok(output) => unsafe {
            libsqlite3_sys::sqlite3_result_text(
                ctx,
                output.as_ptr(),
                -1,
                libsqlite3_sys::SQLITE_TRANSIENT(),
            );
        },
        Err(_) => unsafe {
            libsqlite3_sys::sqlite3_result_error_code(ctx, libsqlite3_sys::SQLITE_CONSTRAINT);
        },
    }
}

fn sqlite_value_text(value: *mut libsqlite3_sys::sqlite3_value) -> Option<String> {
    let text = unsafe { libsqlite3_sys::sqlite3_value_text(value) };
    if text.is_null() {
        return None;
    }
    let len = unsafe { libsqlite3_sys::sqlite3_value_bytes(value) };
    let len = usize::try_from(len).ok()?;
    let bytes = unsafe { std::slice::from_raw_parts(text.cast::<u8>(), len) };
    Some(String::from_utf8_lossy(bytes).into_owned())
}

fn set_custom_connect_options_odbc(odbc_options: &mut OdbcConnectOptions, config: &AppConfig) {
    let batch_size = config.max_pending_rows.clamp(1, 1024);
    odbc_options.batch_size(batch_size);
    log::trace!("ODBC batch size set to {batch_size}");
    odbc_options.max_column_size(None);
}

async fn detect_odbc_dbms_name(
    options: &OdbcConnectOptions,
    config: &AppConfig,
) -> anyhow::Result<String> {
    let mut retries = config.database_connection_retries;
    let conn: OdbcConnection = loop {
        match options.connect().await {
            Ok(c) => break c,
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
    };
    let dbms_name = conn.dbms_name()?;
    conn.close().await?;
    Ok(dbms_name)
}

fn database_url_with_password(config: &AppConfig) -> anyhow::Result<String> {
    let Some(password) = &config.database_password else {
        return Ok(config.database_url.clone());
    };
    let kind = DbKind::from_database_url(&config.database_url);
    if !matches!(kind, DbKind::Mssql) {
        return Ok(config.database_url.clone());
    }
    let mut url = Url::parse(&config.database_url)
        .with_context(|| format!("Unable to parse {}", config.database_url))?;
    url.set_password(Some(password))
        .map_err(|()| anyhow::anyhow!("Unable to set password in database URL"))?;
    Ok(url.to_string())
}
