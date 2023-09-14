mod sql;
mod sql_pseudofunctions;
mod sql_to_json;

use anyhow::{anyhow, Context};
use futures_util::stream::Stream;
use futures_util::StreamExt;
use serde_json::Value;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::time::Duration;

use crate::app_config::AppConfig;
pub use crate::file_cache::FileCache;

use crate::webserver::database::sql_pseudofunctions::extract_req_param;
use crate::webserver::http::RequestInfo;
use crate::MIGRATIONS_DIR;
pub use sql::make_placeholder;
pub use sql::ParsedSqlFile;
use sqlx::any::{
    AnyArguments, AnyConnectOptions, AnyKind, AnyQueryResult, AnyRow, AnyStatement, AnyTypeInfo,
};
use sqlx::migrate::Migrator;
use sqlx::pool::{PoolConnection, PoolOptions};
use sqlx::query::Query;
use sqlx::{Any, AnyConnection, AnyPool, Arguments, ConnectOptions, Either, Executor, Statement};

use self::sql::ParsedSQLStatement;

pub struct Database {
    pub(crate) connection: AnyPool,
}

impl Database {
    pub(crate) async fn prepare_with(
        &self,
        query: &str,
        param_types: &[AnyTypeInfo],
    ) -> anyhow::Result<AnyStatement<'static>> {
        self.connection
            .prepare_with(query, param_types)
            .await
            .map(|s| s.to_owned())
            .with_context(|| format!("Failed to prepare SQL statement: '{query}'"))
    }
}

pub async fn apply_migrations(db: &Database) -> anyhow::Result<()> {
    let migrations_dir = Path::new(MIGRATIONS_DIR);
    if !migrations_dir.exists() {
        log::info!(
            "Not applying database migrations because '{}' does not exist",
            MIGRATIONS_DIR
        );
        return Ok(());
    }
    log::info!("Applying migrations from '{MIGRATIONS_DIR}'");
    let migrator = Migrator::new(migrations_dir)
        .await
        .with_context(|| migration_err("preparing the database migration"))?;
    if migrator.migrations.is_empty() {
        log::info!("No migration found. \
        You can specify database operations to apply when the server first starts by creating files \
        in {MIGRATIONS_DIR}/<VERSION>_<DESCRIPTION>.sql \
        where <VERSION> is a number and <DESCRIPTION> is a short string.");
        return Ok(());
    }
    log::info!("Found {} migrations:", migrator.migrations.len());
    for m in migrator.iter() {
        log::info!(
            "\t[{:04}] {:?} {}",
            m.version,
            m.migration_type,
            m.description
        );
    }
    migrator
        .run(&db.connection)
        .await
        .with_context(|| migration_err("running the migration"))?;
    Ok(())
}

fn migration_err(operation: &'static str) -> String {
    format!(
        "An error occurred while {operation}.
        The path '{MIGRATIONS_DIR}' has to point to a directory, which contains valid SQL files
        with names using the format '<VERSION>_<DESCRIPTION>.sql',
        where <VERSION> is a positive number, and <DESCRIPTION> is a string.
        The current state of migrations will be stored in a table called _sqlx_migrations."
    )
}

pub fn stream_query_results<'a>(
    db: &'a Database,
    sql_file: &'a ParsedSqlFile,
    request: &'a RequestInfo,
) -> impl Stream<Item = DbItem> + 'a {
    async_stream::stream! {
        let mut connection_opt = None;
        for res in &sql_file.statements {
            match res {
                ParsedSQLStatement::Statement(stmt)=>{
                    let query = match bind_parameters(stmt, request) {
                        Ok(q) => q,
                        Err(e) => {
                            yield DbItem::Error(e);
                            continue;
                        }
                    };
                    let connection = match take_connection(db, &mut connection_opt).await {
                        Ok(c) => c,
                        Err(e) => {
                            yield DbItem::Error(e);
                            return;
                        }
                    };
                    let mut stream = query.fetch_many(connection);
                    while let Some(elem) = stream.next().await {
                        let is_err = elem.is_err();
                        yield parse_single_sql_result(elem);
                        if is_err {
                            break;
                        }
                    }
                },
                ParsedSQLStatement::StaticSimpleSelect(value) => {
                    yield DbItem::Row(value.clone().into())
                }
                ParsedSQLStatement::Error(e) => yield DbItem::Error(clone_anyhow_err(e)),
            }
        }
    }
}

async fn take_connection<'a, 'b>(
    db: &'a Database,
    conn: &'b mut Option<PoolConnection<sqlx::Any>>,
) -> anyhow::Result<&'b mut AnyConnection> {
    match conn {
        Some(c) => Ok(c),
        None => match db.connection.acquire().await {
            Ok(c) => {
                log::debug!("Acquired a database connection");
                *conn = Some(c);
                Ok(conn.as_mut().unwrap())
            }
            Err(e) => {
                let err_msg = format!("Unable to acquire a database connection to execute the SQL file. All of the {} {:?} connections are busy.", db.connection.size(), db.connection.any_kind());
                Err(anyhow::Error::new(e).context(err_msg))
            }
        },
    }
}

#[inline]
fn parse_single_sql_result(res: sqlx::Result<Either<AnyQueryResult, AnyRow>>) -> DbItem {
    match res {
        Ok(Either::Right(r)) => DbItem::Row(sql_to_json::row_to_json(&r)),
        Ok(Either::Left(res)) => {
            log::debug!("Finished query with result: {:?}", res);
            DbItem::FinishedQuery
        }
        Err(e) => DbItem::Error(e.into()),
    }
}

fn clone_anyhow_err(err: &anyhow::Error) -> anyhow::Error {
    let mut e = anyhow!("An error occurred during the preparation phase of the SQL");
    for c in err.chain().rev() {
        e = e.context(c.to_string());
    }
    e
}

fn bind_parameters<'a>(
    stmt: &'a PreparedStatement,
    request: &'a RequestInfo,
) -> anyhow::Result<Query<'a, sqlx::Any, AnyArguments<'a>>> {
    let mut arguments = AnyArguments::default();
    for param in &stmt.parameters {
        let argument = extract_req_param(param, request)
            .with_context(|| format!("Unable to extract {param:?} from the HTTP request"))?;
        log::debug!("Binding value {:?} in statement {}", &argument, stmt);
        match argument {
            None => arguments.add(None::<String>),
            Some(Cow::Owned(s)) => arguments.add(s),
            Some(Cow::Borrowed(v)) => arguments.add(v),
        }
    }
    Ok(stmt.statement.query_with(arguments))
}

#[derive(Debug)]
pub enum DbItem {
    Row(Value),
    FinishedQuery,
    Error(anyhow::Error),
}

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
        PoolOptions::new()
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
            ))
    }
}

fn set_custom_connect_options(options: &mut AnyConnectOptions, config: &AppConfig) {
    if let Some(sqlite_options) = options.as_sqlite_mut() {
        for extension_name in &config.sqlite_extensions {
            log::info!("Loading SQLite extension: {}", extension_name);
            *sqlite_options = std::mem::take(sqlite_options).extension(extension_name.clone());
        }
    }
}
struct PreparedStatement {
    statement: AnyStatement<'static>,
    parameters: Vec<sql_pseudofunctions::StmtParam>,
}

impl Display for PreparedStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.statement.sql())
    }
}
