use std::borrow::Cow;
use std::fmt;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Mutex;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
use std::time::Duration;

use anyhow::Context;
use chrono::{Datelike, Timelike};
use futures_util::stream::Stream;
use mysql_async::prelude::Queryable;
use odbc_api::parameter::{InputParameter, VarCharBox, WithDataType};
use odbc_api::{ConnectionOptions, Cursor, IntoParameter, ResultSetMetadata};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_rusqlite::rusqlite;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use crate::app_config::AppConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DbKind {
    Sqlite,
    Postgres,
    MySql,
    Mssql,
    Odbc,
}

#[derive(Debug, Clone)]
pub enum DbParam {
    Null,
    Bool(bool),
    Integer(i64),
    Text(String),
    Bytes(Vec<u8>),
    Timestamp(chrono::DateTime<chrono::Utc>),
}

impl From<Option<String>> for DbParam {
    fn from(value: Option<String>) -> Self {
        value.map_or(Self::Null, Self::Text)
    }
}

#[derive(Debug, Clone)]
pub enum DbValue {
    Null,
    Bool(bool),
    Integer(i64),
    Real(f64),
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct DbColumn {
    pub name: String,
    pub type_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DbRow {
    pub columns: Vec<DbColumn>,
    pub values: Vec<DbValue>,
    pub kind: DbKind,
}

#[derive(Debug, Clone)]
pub enum DbStatementResult {
    Finished,
    Row(DbRow),
}

#[derive(Debug)]
pub enum DbError {
    PoolTimedOut,
    UnsupportedBackend(DbKind),
    Database {
        message: String,
        offset: Option<usize>,
    },
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PoolTimedOut => write!(f, "database connection pool timed out"),
            Self::UnsupportedBackend(kind) => {
                write!(f, "database backend {kind:?} is not implemented yet")
            }
            Self::Database { message, .. } => f.write_str(message),
        }
    }
}

impl std::error::Error for DbError {}

impl From<rusqlite::Error> for DbError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database {
            message: error.to_string(),
            offset: None,
        }
    }
}

impl From<tokio_rusqlite::Error> for DbError {
    fn from(error: tokio_rusqlite::Error) -> Self {
        Self::Database {
            message: error.to_string(),
            offset: None,
        }
    }
}

impl From<tokio_postgres::Error> for DbError {
    fn from(error: tokio_postgres::Error) -> Self {
        if let Some(db_error) = error.as_db_error() {
            return DbError::Database {
                message: db_error.message().to_string(),
                offset: db_error.position().and_then(postgres_error_position),
            };
        }
        db_error(error)
    }
}

impl From<mysql_async::Error> for DbError {
    fn from(error: mysql_async::Error) -> Self {
        db_error(error)
    }
}

impl From<tiberius::error::Error> for DbError {
    fn from(error: tiberius::error::Error) -> Self {
        db_error(error)
    }
}

fn db_error(error: impl std::error::Error) -> DbError {
    DbError::Database {
        message: error.to_string(),
        offset: None,
    }
}

fn postgres_error_position(position: &tokio_postgres::error::ErrorPosition) -> Option<usize> {
    match position {
        tokio_postgres::error::ErrorPosition::Original(position) => usize::try_from(*position).ok(),
        tokio_postgres::error::ErrorPosition::Internal { .. } => None,
    }
}

#[derive(Clone)]
pub struct DbPool {
    inner: Arc<DbPoolInner>,
}

struct DbPoolInner {
    url: String,
    kind: DbKind,
    max_size: u32,
    acquire_timeout: Duration,
    semaphore: Arc<Semaphore>,
    active: AtomicU32,
    on_connect_sql: Option<Arc<String>>,
    sqlite_extensions: Vec<String>,
    idle_sqlite: Mutex<Option<tokio_rusqlite::Connection>>,
}

pub struct DbConnection {
    inner: NativeConnection,
    pool: Arc<DbPoolInner>,
    _permit: OwnedSemaphorePermit,
}

enum NativeConnection {
    Sqlite(tokio_rusqlite::Connection),
    Postgres(tokio_postgres::Client),
    MySql(mysql_async::Conn),
    Mssql(Box<tiberius::Client<Compat<tokio::net::TcpStream>>>),
    Odbc(odbc_api::Connection<'static>),
    Closed,
}

impl Drop for DbConnection {
    fn drop(&mut self) {
        let inner = std::mem::replace(&mut self.inner, NativeConnection::Closed);
        if let NativeConnection::Sqlite(conn) = inner
            && let Ok(mut idle) = self.pool.idle_sqlite.lock()
            && idle.is_none()
        {
            *idle = Some(conn);
        }
        self.pool.active.fetch_sub(1, Ordering::Relaxed);
    }
}

impl DbPool {
    pub fn new(
        config: &AppConfig,
        kind: DbKind,
        max_size: u32,
        on_connect_sql: Option<String>,
    ) -> Self {
        let url = if kind == DbKind::Sqlite {
            normalize_sqlite_url(&config.database_url)
        } else {
            config.database_url.clone()
        };
        Self {
            inner: Arc::new(DbPoolInner {
                url,
                kind,
                max_size,
                acquire_timeout: Duration::from_secs_f64(
                    config.database_connection_acquire_timeout_seconds,
                ),
                semaphore: Arc::new(Semaphore::new(
                    usize::try_from(max_size).unwrap_or(usize::MAX),
                )),
                active: AtomicU32::new(0),
                on_connect_sql: on_connect_sql.map(Arc::new),
                sqlite_extensions: config.sqlite_extensions.clone(),
                idle_sqlite: Mutex::new(None),
            }),
        }
    }

    #[must_use]
    pub fn kind(&self) -> DbKind {
        self.inner.kind
    }

    #[must_use]
    pub fn size(&self) -> u32 {
        self.inner.active.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn num_idle(&self) -> u32 {
        0
    }

    #[must_use]
    pub fn max_size(&self) -> u32 {
        self.inner.max_size
    }

    pub fn close(&self) {}

    pub async fn acquire(&self) -> Result<DbConnection, DbError> {
        let permit = tokio::time::timeout(
            self.inner.acquire_timeout,
            self.inner.semaphore.clone().acquire_owned(),
        )
        .await
        .map_err(|_| DbError::PoolTimedOut)?
        .map_err(|_| DbError::PoolTimedOut)?;
        self.inner.active.fetch_add(1, Ordering::Relaxed);
        if self.inner.kind == DbKind::Sqlite
            && let Ok(mut idle) = self.inner.idle_sqlite.lock()
            && let Some(conn) = idle.take()
        {
            return Ok(DbConnection {
                inner: NativeConnection::Sqlite(conn),
                pool: self.inner.clone(),
                _permit: permit,
            });
        }
        match self.inner.connect().await {
            Ok(inner) => Ok(DbConnection {
                inner,
                pool: self.inner.clone(),
                _permit: permit,
            }),
            Err(e) => {
                self.inner.active.fetch_sub(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }
}

impl DbPoolInner {
    async fn connect(&self) -> Result<NativeConnection, DbError> {
        let mut conn = match self.kind {
            DbKind::Sqlite => NativeConnection::Sqlite(open_sqlite(&self.url).await?),
            DbKind::Postgres => NativeConnection::Postgres(open_postgres(&self.url).await?),
            DbKind::MySql => NativeConnection::MySql(open_mysql(&self.url).await?),
            DbKind::Mssql => NativeConnection::Mssql(Box::new(open_mssql(&self.url).await?)),
            DbKind::Odbc => NativeConnection::Odbc(open_odbc(&self.url)?),
        };
        conn.configure(self).await?;
        if let Some(sql) = &self.on_connect_sql {
            conn.execute(sql, &[]).await?;
        }
        Ok(conn)
    }
}

impl DbConnection {
    #[must_use]
    pub fn kind(&self) -> DbKind {
        self.pool.kind
    }

    pub async fn dbms_name(&mut self) -> Result<String, DbError> {
        match &mut self.inner {
            NativeConnection::Sqlite(_) => Ok("SQLite".to_string()),
            NativeConnection::Postgres(client) => {
                let row = client.query_one("SELECT version()", &[]).await?;
                let version: String = row.try_get(0)?;
                if version.starts_with("PostgreSQL") {
                    Ok("PostgreSQL".to_string())
                } else {
                    Ok(version)
                }
            }
            NativeConnection::MySql(_) => Ok("MySQL".to_string()),
            NativeConnection::Mssql(_) => Ok("Microsoft SQL Server".to_string()),
            NativeConnection::Odbc(conn) => {
                conn.database_management_system_name().map_err(db_error)
            }
            NativeConnection::Closed => Err(DbError::Database {
                message: "database connection is closed".to_string(),
                offset: None,
            }),
        }
    }

    pub async fn execute(
        &mut self,
        sql: &str,
        params: &[DbParam],
    ) -> Result<Vec<DbStatementResult>, DbError> {
        self.inner.execute(sql, params).await
    }

    pub fn execute_stream<'a>(
        &'a mut self,
        sql: &'a str,
        params: &'a [DbParam],
    ) -> Pin<Box<dyn Stream<Item = Result<DbStatementResult, DbError>> + 'a>> {
        self.inner.execute_stream(sql, params)
    }

    pub async fn execute_command(&mut self, sql: &str, params: &[DbParam]) -> Result<(), DbError> {
        let _ = self.execute(sql, params).await?;
        Ok(())
    }

    pub async fn execute_batch(&mut self, sql: &str) -> Result<(), DbError> {
        self.inner.execute_batch(sql).await
    }

    pub async fn fetch_optional(
        &mut self,
        sql: &str,
        params: &[DbParam],
    ) -> Result<Option<DbRow>, DbError> {
        let results = self.execute(sql, params).await?;
        Ok(results.into_iter().find_map(|item| match item {
            DbStatementResult::Row(row) => Some(row),
            DbStatementResult::Finished => None,
        }))
    }
}

impl NativeConnection {
    async fn configure(&mut self, pool: &DbPoolInner) -> Result<(), DbError> {
        match self {
            Self::Sqlite(conn) => configure_sqlite(conn, &pool.sqlite_extensions).await,
            Self::Postgres(_) | Self::MySql(_) | Self::Mssql(_) | Self::Odbc(_) | Self::Closed => {
                Ok(())
            }
        }
    }

    async fn execute(
        &mut self,
        sql: &str,
        params: &[DbParam],
    ) -> Result<Vec<DbStatementResult>, DbError> {
        match self {
            Self::Sqlite(conn) => execute_sqlite(conn, sql, params).await,
            Self::Postgres(client) => execute_postgres(client, sql, params).await,
            Self::MySql(conn) => execute_mysql(conn, sql, params).await,
            Self::Mssql(client) => execute_mssql(client, sql, params).await,
            Self::Odbc(conn) => execute_odbc(conn, sql, params),
            Self::Closed => Err(DbError::Database {
                message: "database connection is closed".to_string(),
                offset: None,
            }),
        }
    }

    fn execute_stream<'a>(
        &'a mut self,
        sql: &'a str,
        params: &'a [DbParam],
    ) -> Pin<Box<dyn Stream<Item = Result<DbStatementResult, DbError>> + 'a>> {
        match self {
            Self::Sqlite(conn) => stream_sqlite(conn, sql, params),
            _ => Box::pin(async_stream::try_stream! {
                for item in self.execute(sql, params).await? {
                    yield item;
                }
            }),
        }
    }

    async fn execute_batch(&mut self, sql: &str) -> Result<(), DbError> {
        match self {
            Self::Sqlite(conn) => execute_sqlite_batch(conn, sql).await,
            Self::Postgres(client) => client.batch_execute(sql).await.map_err(Into::into),
            Self::MySql(conn) => {
                conn.query_drop(sql).await?;
                Ok(())
            }
            Self::Mssql(client) => {
                client.simple_query(sql).await?.into_results().await?;
                Ok(())
            }
            Self::Odbc(conn) => {
                let _ = conn.execute(sql, (), None).map_err(db_error)?;
                Ok(())
            }
            Self::Closed => Err(DbError::Database {
                message: "database connection is closed".to_string(),
                offset: None,
            }),
        }
    }
}

async fn open_sqlite(url: &str) -> Result<tokio_rusqlite::Connection, DbError> {
    let sqlite_path = sqlite_path_from_url(url).map_err(|e| DbError::Database {
        message: e.to_string(),
        offset: None,
    })?;
    let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
        | rusqlite::OpenFlags::SQLITE_OPEN_CREATE
        | rusqlite::OpenFlags::SQLITE_OPEN_URI;
    if sqlite_path == ":memory:" {
        tokio_rusqlite::Connection::open_in_memory_with_flags(flags)
            .await
            .map_err(Into::into)
    } else {
        tokio_rusqlite::Connection::open_with_flags(sqlite_path, flags)
            .await
            .map_err(Into::into)
    }
}

async fn open_postgres(url: &str) -> Result<tokio_postgres::Client, DbError> {
    let (client, connection) = tokio_postgres::connect(url, tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            log::debug!("PostgreSQL connection task finished with error: {error}");
        }
    });
    Ok(client)
}

async fn open_mysql(url: &str) -> Result<mysql_async::Conn, DbError> {
    mysql_async::Conn::from_url(url).await.map_err(Into::into)
}

async fn open_mssql(url: &str) -> Result<tiberius::Client<Compat<tokio::net::TcpStream>>, DbError> {
    let mut config = mssql_config_from_url(url)?;
    config.trust_cert();
    let tcp = tokio::net::TcpStream::connect(config.get_addr())
        .await
        .map_err(db_error)?;
    tcp.set_nodelay(true).map_err(db_error)?;
    tiberius::Client::connect(config, tcp.compat_write())
        .await
        .map_err(Into::into)
}

fn open_odbc(url: &str) -> Result<odbc_api::Connection<'static>, DbError> {
    let conn_str = odbc_connection_string(url);
    odbc_api::environment()
        .map_err(db_error)?
        .connect_with_connection_string(&conn_str, ConnectionOptions::default())
        .map_err(db_error)
}

fn odbc_connection_string(url: &str) -> String {
    let trimmed = url.trim().strip_prefix("odbc:").unwrap_or(url.trim());
    if trimmed.contains('=') {
        trimmed.to_string()
    } else {
        format!("DSN={trimmed}")
    }
}

fn mssql_config_from_url(url: &str) -> Result<tiberius::Config, DbError> {
    if url.starts_with("jdbc:") {
        return tiberius::Config::from_jdbc_string(url).map_err(Into::into);
    }
    if url.contains('=') {
        return tiberius::Config::from_ado_string(url).map_err(Into::into);
    }

    let without_scheme = url
        .strip_prefix("mssql://")
        .or_else(|| url.strip_prefix("sqlserver://"))
        .ok_or_else(|| DbError::Database {
            message: format!("not a SQL Server URL: {url}"),
            offset: None,
        })?;
    let (authority, database) = without_scheme
        .split_once('/')
        .map_or((without_scheme, ""), |(authority, rest)| {
            (authority, rest.split(['?', '#']).next().unwrap_or(""))
        });
    let (credentials, host_port) = authority
        .rsplit_once('@')
        .map_or(("", authority), |(credentials, host_port)| {
            (credentials, host_port)
        });
    let (user, password) = credentials.split_once(':').unwrap_or((credentials, ""));
    let (host, port) = parse_host_port(host_port);

    let mut config = tiberius::Config::new();
    config.host(decode_url_part(host)?);
    if let Some(port) = port {
        config.port(port);
    }
    if !database.is_empty() {
        config.database(decode_url_part(database)?);
    }
    if !user.is_empty() {
        config.authentication(tiberius::AuthMethod::sql_server(
            decode_url_part(user)?,
            decode_url_part(password)?,
        ));
    }
    Ok(config)
}

fn parse_host_port(host_port: &str) -> (&str, Option<u16>) {
    let Some((host, port)) = host_port.rsplit_once(':') else {
        return (host_port, None);
    };
    match port.parse::<u16>() {
        Ok(port) => (host, Some(port)),
        Err(_) => (host_port, None),
    }
}

fn decode_url_part(value: &str) -> Result<String, DbError> {
    percent_encoding::percent_decode_str(value)
        .decode_utf8()
        .map(std::borrow::Cow::into_owned)
        .map_err(db_error)
}

fn sqlite_path_from_url(url: &str) -> anyhow::Result<String> {
    let Some(rest) = url.strip_prefix("sqlite:") else {
        anyhow::bail!("not a sqlite URL: {url}");
    };
    let rest = rest.strip_prefix("//").unwrap_or(rest);
    let decoded = percent_encoding::percent_decode_str(rest)
        .decode_utf8()
        .with_context(|| format!("invalid percent encoding in sqlite URL {url:?}"))?;
    let decoded = decoded.into_owned();
    if decoded.contains('?') && !decoded.starts_with("file:") {
        Ok(format!("file:{decoded}"))
    } else {
        Ok(decoded)
    }
}

fn normalize_sqlite_url(url: &str) -> String {
    static MEMORY_DB_ID: AtomicU32 = AtomicU32::new(0);
    let Ok(path) = sqlite_path_from_url(url) else {
        return url.to_string();
    };
    if path == ":memory:" || path.starts_with("file::memory:") {
        let id = MEMORY_DB_ID.fetch_add(1, Ordering::Relaxed);
        format!("sqlite://file:sqlpage_memory_{id}?mode=memory&cache=shared")
    } else {
        url.to_string()
    }
}

async fn configure_sqlite(
    conn: &tokio_rusqlite::Connection,
    extensions: &[String],
) -> Result<(), DbError> {
    let extensions = extensions.to_vec();
    conn.call(move |conn| {
        conn.create_collation("NOCASE", |a, b| a.to_lowercase().cmp(&b.to_lowercase()))?;
        conn.create_scalar_function(
            "upper",
            1,
            rusqlite::functions::FunctionFlags::SQLITE_UTF8
                | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC,
            |ctx| {
                let arg = ctx.get::<Option<String>>(0)?;
                Ok(arg.map(|s| s.to_uppercase()))
            },
        )?;
        conn.create_scalar_function(
            "lower",
            1,
            rusqlite::functions::FunctionFlags::SQLITE_UTF8
                | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC,
            |ctx| {
                let arg = ctx.get::<Option<String>>(0)?;
                Ok(arg.map(|s| s.to_lowercase()))
            },
        )?;
        if !extensions.is_empty() {
            unsafe { conn.load_extension_enable()? };
            for extension in extensions {
                unsafe { conn.load_extension(PathBuf::from(extension), None::<&str>)? };
            }
            conn.load_extension_disable()?;
        }
        Ok::<_, rusqlite::Error>(())
    })
    .await
    .map_err(Into::into)
}

async fn execute_sqlite(
    conn: &tokio_rusqlite::Connection,
    sql: &str,
    params: &[DbParam],
) -> Result<Vec<DbStatementResult>, DbError> {
    let sql = sql.to_string();
    let params = params.to_vec();
    conn.call(move |conn| {
        let mut stmt = conn.prepare(&sql)?;
        let values = params
            .into_iter()
            .map(sqlite_value_from_param)
            .collect::<Vec<_>>();
        let column_count = stmt.column_count();
        if column_count == 0 {
            if values.is_empty() {
                drop(stmt);
                conn.execute_batch(&sql)?;
            } else {
                stmt.execute(rusqlite::params_from_iter(values))?;
            }
            return Ok(vec![DbStatementResult::Finished]);
        }
        let columns = (0..column_count)
            .map(|idx| DbColumn {
                name: stmt.column_name(idx).unwrap_or("").to_string(),
                type_name: None,
            })
            .collect::<Vec<_>>();
        let mut rows = stmt.query(rusqlite::params_from_iter(values))?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            let mut values = Vec::with_capacity(column_count);
            for idx in 0..column_count {
                values.push(sqlite_value(row.get_ref(idx)?));
            }
            result.push(DbStatementResult::Row(DbRow {
                columns: columns.clone(),
                values,
                kind: DbKind::Sqlite,
            }));
        }
        Ok(result)
    })
    .await
    .map_err(Into::into)
}

fn stream_sqlite<'a>(
    conn: &'a tokio_rusqlite::Connection,
    sql: &str,
    params: &[DbParam],
) -> Pin<Box<dyn Stream<Item = Result<DbStatementResult, DbError>> + 'a>> {
    let sql = sql.to_string();
    let params = params.to_vec();
    Box::pin(async_stream::try_stream! {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let call = conn.call(move |conn| {
            let mut stmt = conn.prepare(&sql).map_err(DbError::from)?;
            let values = params
                .into_iter()
                .map(sqlite_value_from_param)
                .collect::<Vec<_>>();
            let column_count = stmt.column_count();
            if column_count == 0 {
                if values.is_empty() {
                    drop(stmt);
                    conn.execute_batch(&sql).map_err(DbError::from)?;
                } else {
                    stmt.execute(rusqlite::params_from_iter(values))
                        .map_err(DbError::from)?;
                }
                let _ = tx.blocking_send(DbStatementResult::Finished);
                return Ok(());
            }
            let columns = (0..column_count)
                .map(|idx| DbColumn {
                    name: stmt.column_name(idx).unwrap_or("").to_string(),
                    type_name: None,
                })
                .collect::<Vec<_>>();
            let mut rows = stmt
                .query(rusqlite::params_from_iter(values))
                .map_err(DbError::from)?;
            while let Some(row) = rows.next().map_err(DbError::from)? {
                let mut values = Vec::with_capacity(column_count);
                for idx in 0..column_count {
                    values.push(sqlite_value(row.get_ref(idx).map_err(DbError::from)?));
                }
                if tx
                    .blocking_send(DbStatementResult::Row(DbRow {
                        columns: columns.clone(),
                        values,
                        kind: DbKind::Sqlite,
                    }))
                    .is_err()
                {
                    return Ok(());
                }
            }
            Ok(())
        });
        tokio::pin!(call);
        let mut call_finished = false;
        let mut call_result = None;
        loop {
            tokio::select! {
                result = &mut call, if !call_finished => {
                    call_finished = true;
                    call_result = Some(sqlite_call_result(result));
                }
                maybe_item = rx.recv() => {
                    match maybe_item {
                        Some(item) => yield item,
                        None => break,
                    }
                }
            }
        }
        if !call_finished {
            call_result = Some(sqlite_call_result(call.await));
        }
        if let Some(result) = call_result {
            result?;
        }
    })
}

fn sqlite_call_result(result: Result<(), tokio_rusqlite::Error<DbError>>) -> Result<(), DbError> {
    match result {
        Ok(()) => Ok(()),
        Err(tokio_rusqlite::Error::Error(error)) => Err(error),
        Err(error) => Err(DbError::Database {
            message: error.to_string(),
            offset: None,
        }),
    }
}

async fn execute_sqlite_batch(conn: &tokio_rusqlite::Connection, sql: &str) -> Result<(), DbError> {
    let sql = sql.to_string();
    conn.call(move |conn| conn.execute_batch(&sql))
        .await
        .map_err(Into::into)
}

async fn execute_postgres(
    client: &tokio_postgres::Client,
    sql: &str,
    params: &[DbParam],
) -> Result<Vec<DbStatementResult>, DbError> {
    let params = params.iter().map(PgParam::from).collect::<Vec<_>>();
    let param_refs = params
        .iter()
        .map(|param| param as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect::<Vec<_>>();
    let rows = client.query(sql, &param_refs).await?;
    if rows.is_empty() {
        return Ok(vec![DbStatementResult::Finished]);
    }
    Ok(rows
        .into_iter()
        .map(|row| DbStatementResult::Row(postgres_row(&row)))
        .collect())
}

#[derive(Debug)]
enum PgParam {
    Null(Option<String>),
    Bool(bool),
    Integer(i64),
    Text(String),
    Bytes(Vec<u8>),
    Timestamp(chrono::DateTime<chrono::Utc>),
}

impl From<&DbParam> for PgParam {
    fn from(value: &DbParam) -> Self {
        match value {
            DbParam::Null => Self::Null(None),
            DbParam::Bool(value) => Self::Bool(*value),
            DbParam::Integer(value) => Self::Integer(*value),
            DbParam::Text(value) => Self::Text(value.clone()),
            DbParam::Bytes(value) => Self::Bytes(value.clone()),
            DbParam::Timestamp(value) => Self::Timestamp(*value),
        }
    }
}

impl tokio_postgres::types::ToSql for PgParam {
    fn to_sql(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        match self {
            Self::Null(value) => value.to_sql(ty, out),
            Self::Bool(value) => value.to_sql(ty, out),
            Self::Integer(value) => value.to_sql(ty, out),
            Self::Text(value) => value.to_sql(ty, out),
            Self::Bytes(value) => value.to_sql(ty, out),
            Self::Timestamp(value) => value.to_sql(ty, out),
        }
    }

    fn accepts(_ty: &tokio_postgres::types::Type) -> bool
    where
        Self: Sized,
    {
        true
    }

    tokio_postgres::types::to_sql_checked!();
}

fn postgres_row(row: &tokio_postgres::Row) -> DbRow {
    let columns = row
        .columns()
        .iter()
        .map(|column| DbColumn {
            name: column.name().to_string(),
            type_name: Some(column.type_().name().to_string()),
        })
        .collect::<Vec<_>>();
    let values = row
        .columns()
        .iter()
        .enumerate()
        .map(|(idx, column)| postgres_value(row, idx, column.type_()))
        .collect::<Vec<_>>();
    DbRow {
        columns,
        values,
        kind: DbKind::Postgres,
    }
}

fn postgres_value(
    row: &tokio_postgres::Row,
    idx: usize,
    ty: &tokio_postgres::types::Type,
) -> DbValue {
    use tokio_postgres::types::Type;
    if row.try_get::<_, Option<String>>(idx).ok() == Some(None) {
        return DbValue::Null;
    }
    match *ty {
        Type::BOOL => row
            .try_get::<_, bool>(idx)
            .map_or(DbValue::Null, DbValue::Bool),
        Type::INT2 => row
            .try_get::<_, i16>(idx)
            .map_or(DbValue::Null, |value| DbValue::Integer(i64::from(value))),
        Type::INT4 => row
            .try_get::<_, i32>(idx)
            .map_or(DbValue::Null, |value| DbValue::Integer(i64::from(value))),
        Type::INT8 => row
            .try_get::<_, i64>(idx)
            .map_or(DbValue::Null, DbValue::Integer),
        Type::FLOAT4 => row
            .try_get::<_, f32>(idx)
            .map_or(DbValue::Null, |value| DbValue::Real(f64::from(value))),
        Type::FLOAT8 => row
            .try_get::<_, f64>(idx)
            .map_or(DbValue::Null, DbValue::Real),
        Type::NUMERIC => row
            .try_get::<_, PgNumericValue>(idx)
            .map_or(DbValue::Null, |value| value.0),
        Type::BYTEA => row
            .try_get::<_, Vec<u8>>(idx)
            .map_or(DbValue::Null, DbValue::Bytes),
        Type::TIMESTAMPTZ => row
            .try_get::<_, chrono::DateTime<chrono::Utc>>(idx)
            .map_or(DbValue::Null, |value| DbValue::Text(value.to_rfc3339())),
        Type::TIMESTAMP => row
            .try_get::<_, chrono::NaiveDateTime>(idx)
            .map_or(DbValue::Null, |value| DbValue::Text(value.to_string())),
        Type::DATE => row
            .try_get::<_, chrono::NaiveDate>(idx)
            .map_or(DbValue::Null, |value| DbValue::Text(value.to_string())),
        Type::JSON | Type::JSONB => row
            .try_get::<_, serde_json::Value>(idx)
            .map_or(DbValue::Null, |value| DbValue::Text(value.to_string())),
        Type::UUID => row
            .try_get::<_, uuid::Uuid>(idx)
            .map_or(DbValue::Null, |value| DbValue::Text(value.to_string())),
        _ => row
            .try_get::<_, String>(idx)
            .map_or(DbValue::Null, DbValue::Text),
    }
}

struct PgNumericValue(DbValue);

impl<'a> tokio_postgres::types::FromSql<'a> for PgNumericValue {
    fn from_sql(
        ty: &tokio_postgres::types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        if *ty != tokio_postgres::types::Type::NUMERIC {
            return Err("PgNumericValue only supports NUMERIC".into());
        }
        Ok(Self(numeric_text_value(&postgres_numeric_to_string(raw)?)))
    }

    fn accepts(ty: &tokio_postgres::types::Type) -> bool {
        *ty == tokio_postgres::types::Type::NUMERIC
    }
}

fn postgres_numeric_to_string(
    raw: &[u8],
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    use std::fmt::Write as _;

    const SIGN_POSITIVE: u16 = 0x0000;
    const SIGN_NEGATIVE: u16 = 0x4000;
    const SIGN_NAN: u16 = 0xC000;
    const SIGN_POSITIVE_INFINITY: u16 = 0xD000;
    const SIGN_NEGATIVE_INFINITY: u16 = 0xF000;

    if raw.len() < 8 {
        return Err("invalid PostgreSQL NUMERIC payload".into());
    }
    let digit_count = usize::from(read_u16_be(raw, 0));
    let weight = read_i16_be(raw, 2);
    let sign = read_u16_be(raw, 4);
    let decimal_scale = usize::from(read_u16_be(raw, 6));
    if raw.len() < 8 + digit_count * 2 {
        return Err("truncated PostgreSQL NUMERIC payload".into());
    }
    match sign {
        SIGN_NAN => return Ok("NaN".to_string()),
        SIGN_POSITIVE_INFINITY => return Ok("Infinity".to_string()),
        SIGN_NEGATIVE_INFINITY => return Ok("-Infinity".to_string()),
        SIGN_POSITIVE | SIGN_NEGATIVE => {}
        _ => return Err("invalid PostgreSQL NUMERIC sign".into()),
    }

    let digits = (0..digit_count)
        .map(|idx| read_u16_be(raw, 8 + idx * 2))
        .collect::<Vec<_>>();
    let integer_group_count = i32::from(weight) + 1;

    let mut integer_part = String::new();
    if integer_group_count <= 0 {
        integer_part.push('0');
    } else {
        for group_idx in 0..usize::try_from(integer_group_count)? {
            let digit = digits.get(group_idx).copied().unwrap_or_default();
            if group_idx == 0 {
                integer_part.push_str(&digit.to_string());
            } else {
                write!(integer_part, "{digit:04}")?;
            }
        }
    }

    let mut fraction_part = String::new();
    for _ in 0..(-integer_group_count).max(0) {
        fraction_part.push_str("0000");
    }
    let first_fraction_group = usize::try_from(integer_group_count.max(0))?;
    for digit in digits.iter().skip(first_fraction_group) {
        write!(fraction_part, "{digit:04}")?;
    }
    fraction_part.truncate(decimal_scale);
    while fraction_part.len() < decimal_scale {
        fraction_part.push('0');
    }

    let prefix = if sign == SIGN_NEGATIVE { "-" } else { "" };
    if decimal_scale == 0 {
        Ok(format!("{prefix}{integer_part}"))
    } else {
        Ok(format!("{prefix}{integer_part}.{fraction_part}"))
    }
}

fn read_i16_be(bytes: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u16_be(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

async fn execute_mysql(
    conn: &mut mysql_async::Conn,
    sql: &str,
    params: &[DbParam],
) -> Result<Vec<DbStatementResult>, DbError> {
    let values = params
        .iter()
        .map(mysql_value_from_param)
        .collect::<Vec<_>>();
    if values.is_empty() {
        collect_mysql_results(conn.query_iter(sql).await?).await
    } else {
        collect_mysql_results(
            conn.exec_iter(sql, mysql_async::Params::Positional(values))
                .await?,
        )
        .await
    }
}

async fn collect_mysql_results<P>(
    mut query_result: mysql_async::QueryResult<'_, 'static, P>,
) -> Result<Vec<DbStatementResult>, DbError>
where
    P: mysql_async::prelude::Protocol,
{
    let mut result = Vec::new();
    loop {
        let columns = query_result
            .columns_ref()
            .iter()
            .map(|column| DbColumn {
                name: column.name_str().into_owned(),
                type_name: Some(format!("{:?}", column.column_type())),
            })
            .collect::<Vec<_>>();
        while let Some(row) = query_result.next().await? {
            let values = row
                .unwrap_raw()
                .into_iter()
                .zip(columns.iter())
                .map(|(value, column)| mysql_value(value, column.type_name.as_deref()))
                .collect::<Vec<_>>();
            result.push(DbStatementResult::Row(DbRow {
                columns: columns.clone(),
                values,
                kind: DbKind::MySql,
            }));
        }
        if query_result.is_empty() {
            break;
        }
    }
    query_result.drop_result().await?;
    if result.is_empty() {
        result.push(DbStatementResult::Finished);
    }
    Ok(result)
}

fn mysql_value_from_param(param: &DbParam) -> mysql_async::Value {
    match param {
        DbParam::Null => mysql_async::Value::NULL,
        DbParam::Bool(value) => mysql_async::Value::Int(i64::from(*value)),
        DbParam::Integer(value) => mysql_async::Value::Int(*value),
        DbParam::Text(value) => mysql_async::Value::Bytes(value.clone().into_bytes()),
        DbParam::Bytes(value) => mysql_async::Value::Bytes(value.clone()),
        DbParam::Timestamp(value) => {
            let value = value.naive_utc();
            mysql_async::Value::Date(
                u16::try_from(value.year()).unwrap_or_default(),
                u8::try_from(value.month()).unwrap_or_default(),
                u8::try_from(value.day()).unwrap_or_default(),
                u8::try_from(value.hour()).unwrap_or_default(),
                u8::try_from(value.minute()).unwrap_or_default(),
                u8::try_from(value.second()).unwrap_or_default(),
                value.nanosecond() / 1000,
            )
        }
    }
}

fn mysql_value(value: Option<mysql_async::Value>, type_name: Option<&str>) -> DbValue {
    use mysql_async::Value;
    match value {
        None | Some(Value::NULL) => DbValue::Null,
        Some(Value::Int(value)) => DbValue::Integer(value),
        Some(Value::UInt(value)) => {
            i64::try_from(value).map_or_else(|_| DbValue::Text(value.to_string()), DbValue::Integer)
        }
        Some(Value::Float(value)) => DbValue::Real(f64::from(value)),
        Some(Value::Double(value)) => DbValue::Real(value),
        Some(Value::Bytes(bytes)) => String::from_utf8(bytes).map_or_else(
            |err| DbValue::Bytes(err.into_bytes()),
            |value| typed_text_value(value, type_name),
        ),
        Some(Value::Date(year, month, day, hour, minute, second, micros)) => DbValue::Text(
            format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}.{micros:06}"),
        ),
        Some(Value::Time(negative, days, hours, minutes, seconds, micros)) => {
            DbValue::Text(format!(
                "{}{:03}:{minutes:02}:{seconds:02}.{micros:06}",
                if negative { "-" } else { "" },
                days * 24 + u32::from(hours)
            ))
        }
    }
}

async fn execute_mssql(
    client: &mut tiberius::Client<Compat<tokio::net::TcpStream>>,
    sql: &str,
    params: &[DbParam],
) -> Result<Vec<DbStatementResult>, DbError> {
    if params.is_empty() {
        let rows = client.simple_query(sql).await?.into_results().await?;
        return Ok(mssql_results(rows));
    }
    let params = params.iter().map(MssqlParam::from).collect::<Vec<_>>();
    let param_refs = params
        .iter()
        .map(|param| param as &dyn tiberius::ToSql)
        .collect::<Vec<_>>();
    let rows = client.query(sql, &param_refs).await?.into_results().await?;
    Ok(mssql_results(rows))
}

fn mssql_results(rows: Vec<Vec<tiberius::Row>>) -> Vec<DbStatementResult> {
    let mut result = Vec::new();
    for set in rows {
        for row in set {
            result.push(DbStatementResult::Row(mssql_row(row)));
        }
    }
    if result.is_empty() {
        result.push(DbStatementResult::Finished);
    }
    result
}

enum MssqlParam {
    Text(Option<String>),
    Bool(bool),
    Integer(i64),
    Bytes(Vec<u8>),
    Timestamp(chrono::NaiveDateTime),
}

impl From<&DbParam> for MssqlParam {
    fn from(value: &DbParam) -> Self {
        match value {
            DbParam::Null => Self::Text(None),
            DbParam::Bool(value) => Self::Bool(*value),
            DbParam::Integer(value) => Self::Integer(*value),
            DbParam::Text(value) => Self::Text(Some(value.clone())),
            DbParam::Bytes(value) => Self::Bytes(value.clone()),
            DbParam::Timestamp(value) => Self::Timestamp(value.naive_utc()),
        }
    }
}

impl tiberius::ToSql for MssqlParam {
    fn to_sql(&self) -> tiberius::ColumnData<'_> {
        match self {
            Self::Text(value) => tiberius::ColumnData::String(value.as_ref().map(Cow::from)),
            Self::Bool(value) => tiberius::ColumnData::Bit(Some(*value)),
            Self::Integer(value) => tiberius::ColumnData::I64(Some(*value)),
            Self::Bytes(value) => tiberius::ColumnData::Binary(Some(Cow::from(value))),
            Self::Timestamp(value) => value.to_sql(),
        }
    }
}

fn mssql_row(row: tiberius::Row) -> DbRow {
    let columns = row
        .columns()
        .iter()
        .map(|column| DbColumn {
            name: column.name().to_string(),
            type_name: Some(format!("{:?}", column.column_type())),
        })
        .collect::<Vec<_>>();
    let values = row.into_iter().map(mssql_value).collect::<Vec<_>>();
    DbRow {
        columns,
        values,
        kind: DbKind::Mssql,
    }
}

fn mssql_value(value: tiberius::ColumnData<'static>) -> DbValue {
    use tiberius::ColumnData;
    match value {
        ColumnData::U8(value) => {
            value.map_or(DbValue::Null, |value| DbValue::Integer(i64::from(value)))
        }
        ColumnData::I16(value) => {
            value.map_or(DbValue::Null, |value| DbValue::Integer(i64::from(value)))
        }
        ColumnData::I32(value) => {
            value.map_or(DbValue::Null, |value| DbValue::Integer(i64::from(value)))
        }
        ColumnData::I64(value) => value.map_or(DbValue::Null, DbValue::Integer),
        ColumnData::F32(value) => {
            value.map_or(DbValue::Null, |value| DbValue::Real(f64::from(value)))
        }
        ColumnData::F64(value) => value.map_or(DbValue::Null, DbValue::Real),
        ColumnData::Bit(value) => value.map_or(DbValue::Null, DbValue::Bool),
        ColumnData::String(value) => {
            value.map_or(DbValue::Null, |value| DbValue::Text(value.into_owned()))
        }
        ColumnData::Binary(value) => {
            value.map_or(DbValue::Null, |value| DbValue::Bytes(value.into_owned()))
        }
        ColumnData::Guid(value) => {
            value.map_or(DbValue::Null, |value| DbValue::Text(value.to_string()))
        }
        ColumnData::Numeric(value) => value.map_or(DbValue::Null, |value| {
            numeric_text_value(&value.to_string())
        }),
        other => DbValue::Text(format!("{other:?}")),
    }
}

fn execute_odbc(
    conn: &mut odbc_api::Connection<'static>,
    sql: &str,
    params: &[DbParam],
) -> Result<Vec<DbStatementResult>, DbError> {
    let parameters = OdbcParameters::from_params(params);
    let cursor = if parameters.is_empty() {
        conn.execute(sql, (), None).map_err(db_error)?
    } else {
        conn.execute(sql, parameters.as_slice(), None)
            .map_err(db_error)?
    };
    let Some(cursor) = cursor else {
        return Ok(vec![DbStatementResult::Finished]);
    };
    collect_odbc_rows(cursor)
}

struct OdbcParameters {
    values: Vec<Box<dyn InputParameter>>,
}

impl OdbcParameters {
    fn from_params(params: &[DbParam]) -> Self {
        Self {
            values: params.iter().map(odbc_parameter).collect(),
        }
    }

    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn as_slice(&self) -> &[Box<dyn InputParameter>] {
        &self.values
    }
}

fn odbc_parameter(param: &DbParam) -> Box<dyn InputParameter> {
    match param {
        DbParam::Null => Box::new(WithDataType::new(
            VarCharBox::null(),
            odbc_api::DataType::Varchar { length: None },
        )),
        DbParam::Text(value) => Box::new(value.clone().into_parameter()),
        DbParam::Bool(value) => Box::new(i32::from(*value).into_parameter()),
        DbParam::Integer(value) => Box::new((*value).into_parameter()),
        DbParam::Bytes(value) => Box::new(value.clone().into_parameter()),
        DbParam::Timestamp(value) => Box::new(
            WithDataType::new(
                odbc_api::Nullable::new(odbc_api::sys::Timestamp {
                    year: i16::try_from(value.year()).unwrap_or_default(),
                    month: u16::try_from(value.month()).unwrap_or_default(),
                    day: u16::try_from(value.day()).unwrap_or_default(),
                    hour: u16::try_from(value.hour()).unwrap_or_default(),
                    minute: u16::try_from(value.minute()).unwrap_or_default(),
                    second: u16::try_from(value.second()).unwrap_or_default(),
                    fraction: value.nanosecond(),
                }),
                odbc_api::DataType::Timestamp { precision: 6 },
            )
            .into_parameter(),
        ),
    }
}

fn collect_odbc_rows<C>(mut cursor: C) -> Result<Vec<DbStatementResult>, DbError>
where
    C: Cursor + ResultSetMetadata,
{
    let column_count = cursor.num_result_cols().map_err(db_error)?;
    let column_count = usize::try_from(column_count).map_err(db_error)?;
    let mut columns = Vec::with_capacity(column_count);
    for idx in 0..column_count {
        let mut description = odbc_api::ColumnDescription::default();
        let column_number = u16::try_from(idx + 1).map_err(db_error)?;
        cursor
            .describe_col(column_number, &mut description)
            .map_err(db_error)?;
        columns.push(DbColumn {
            name: description
                .name_to_string()
                .unwrap_or_else(|_| format!("col{idx}")),
            type_name: Some(format!("{:?}", description.data_type)),
        });
    }
    let mut result = Vec::new();
    while let Some(mut row) = cursor.next_row().map_err(db_error)? {
        let mut values = Vec::with_capacity(column_count);
        for (idx, column) in columns.iter().enumerate() {
            let column_number = u16::try_from(idx + 1).map_err(db_error)?;
            let mut value = Vec::new();
            if is_binary_type_name(column.type_name.as_deref()) {
                if row
                    .get_binary(column_number, &mut value)
                    .map_err(db_error)?
                {
                    values.push(DbValue::Bytes(value));
                } else {
                    values.push(DbValue::Null);
                }
            } else if row.get_text(column_number, &mut value).map_err(db_error)? {
                values.push(String::from_utf8(value).map_or_else(
                    |err| DbValue::Bytes(err.into_bytes()),
                    |value| typed_text_value(trim_odbc_text(value), column.type_name.as_deref()),
                ));
            } else {
                values.push(DbValue::Null);
            }
        }
        result.push(DbStatementResult::Row(DbRow {
            columns: columns.clone(),
            values,
            kind: DbKind::Odbc,
        }));
    }
    if result.is_empty() {
        result.push(DbStatementResult::Finished);
    }
    Ok(result)
}

fn sqlite_value_from_param(param: DbParam) -> rusqlite::types::Value {
    match param {
        DbParam::Null => rusqlite::types::Value::Null,
        DbParam::Bool(value) => rusqlite::types::Value::Integer(i64::from(value)),
        DbParam::Integer(value) => rusqlite::types::Value::Integer(value),
        DbParam::Text(s) => rusqlite::types::Value::Text(s),
        DbParam::Bytes(bytes) => rusqlite::types::Value::Blob(bytes),
        DbParam::Timestamp(ts) => {
            rusqlite::types::Value::Text(ts.naive_utc().format("%F %T").to_string())
        }
    }
}

fn sqlite_value(value: rusqlite::types::ValueRef<'_>) -> DbValue {
    match value {
        rusqlite::types::ValueRef::Null => DbValue::Null,
        rusqlite::types::ValueRef::Integer(i) => DbValue::Integer(i),
        rusqlite::types::ValueRef::Real(f) => DbValue::Real(f),
        rusqlite::types::ValueRef::Text(s) => {
            DbValue::Text(String::from_utf8_lossy(s).into_owned())
        }
        rusqlite::types::ValueRef::Blob(b) => DbValue::Bytes(b.to_vec()),
    }
}

fn typed_text_value(value: String, type_name: Option<&str>) -> DbValue {
    let Some(type_name) = type_name else {
        return DbValue::Text(value);
    };
    if is_numeric_type_name(type_name) {
        numeric_text_value(&value)
    } else if is_bool_type_name(type_name) {
        bool_text_value(&value).unwrap_or(DbValue::Text(value))
    } else {
        DbValue::Text(value)
    }
}

fn trim_odbc_text(mut value: String) -> String {
    if value.ends_with('\0') {
        value.pop();
    }
    value
}

fn is_binary_type_name(type_name: Option<&str>) -> bool {
    type_name.is_some_and(|type_name| {
        type_name.starts_with("Binary")
            || type_name.starts_with("Varbinary")
            || type_name.starts_with("LongVarbinary")
    })
}

fn is_numeric_type_name(type_name: &str) -> bool {
    type_name.contains("DECIMAL")
        || type_name.contains("NEWDECIMAL")
        || type_name.starts_with("Decimal")
        || type_name.starts_with("Numeric")
        || type_name.starts_with("Integer")
        || type_name.starts_with("SmallInt")
        || type_name.starts_with("TinyInt")
        || type_name.starts_with("BigInt")
        || type_name.starts_with("Float")
        || type_name.starts_with("Real")
        || type_name.starts_with("Double")
}

fn is_bool_type_name(type_name: &str) -> bool {
    type_name.starts_with("Bit")
}

fn numeric_text_value(value: &str) -> DbValue {
    let trimmed = value.trim();
    if let Some(integer) = zero_fraction_integer_text(trimmed)
        && let Ok(value) = integer.parse::<i64>()
    {
        return DbValue::Integer(value);
    }
    if let Ok(value) = trimmed.parse::<i64>() {
        return DbValue::Integer(value);
    }
    if let Ok(value) = trimmed.parse::<f64>()
        && value.is_finite()
    {
        return DbValue::Real(value);
    }
    DbValue::Text(value.to_string())
}

fn zero_fraction_integer_text(value: &str) -> Option<String> {
    let (integer, fraction) = value.split_once('.')?;
    if !fraction.chars().all(|c| c == '0') {
        return None;
    }
    let (sign, digits) = integer
        .strip_prefix('-')
        .map_or(("", integer), |digits| ("-", digits));
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("{sign}{digits}"))
}

fn bool_text_value(value: &str) -> Option<DbValue> {
    match value.trim() {
        "0" | "false" | "FALSE" => Some(DbValue::Bool(false)),
        "1" | "true" | "TRUE" => Some(DbValue::Bool(true)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use std::time::Duration;

    #[tokio::test]
    async fn sqlite_stream_yields_first_row_before_statement_finishes() {
        let conn = tokio_rusqlite::Connection::open_in_memory().await.unwrap();
        conn.call(|conn| {
            conn.create_scalar_function(
                "wait_ms",
                1,
                rusqlite::functions::FunctionFlags::SQLITE_UTF8,
                |ctx| {
                    let millis = ctx.get::<i64>(0)?;
                    std::thread::sleep(Duration::from_millis(millis.cast_unsigned()));
                    Ok(millis)
                },
            )
        })
        .await
        .unwrap();

        let mut stream = stream_sqlite(&conn, "SELECT 1 AS n UNION ALL SELECT wait_ms(500)", &[]);

        let first = tokio::time::timeout(Duration::from_millis(100), stream.next())
            .await
            .expect("SQLite should yield the first row before the second row has been computed")
            .unwrap()
            .unwrap();
        let DbStatementResult::Row(row) = first else {
            panic!("expected first SQLite stream item to be a row");
        };
        assert!(matches!(row.values.first(), Some(DbValue::Integer(1))));
    }

    #[test]
    fn decimal_text_without_fraction_becomes_integer() {
        assert!(matches!(numeric_text_value("2"), DbValue::Integer(2)));
        assert!(matches!(numeric_text_value("2.000"), DbValue::Integer(2)));
    }

    #[test]
    fn decimal_text_with_fraction_becomes_real() {
        match numeric_text_value("123.45") {
            DbValue::Real(value) => assert!((value - 123.45).abs() < f64::EPSILON),
            value => panic!("expected real value, got {value:?}"),
        }
    }

    #[test]
    fn typed_text_keeps_text_columns_as_text() {
        assert!(matches!(
            typed_text_value("2".to_string(), Some("VarChar { length: Some(255) }")),
            DbValue::Text(value) if value == "2"
        ));
    }

    #[test]
    fn typed_text_parses_decimal_columns() {
        assert!(matches!(
            typed_text_value("2.00".to_string(), Some("MYSQL_TYPE_NEWDECIMAL")),
            DbValue::Integer(2)
        ));
    }

    #[test]
    fn odbc_text_trims_trailing_nul() {
        assert_eq!(trim_odbc_text("hello\0".to_string()), "hello");
        assert_eq!(trim_odbc_text("hello".to_string()), "hello");
    }

    #[test]
    fn odbc_binary_type_detection() {
        assert!(is_binary_type_name(Some("Binary { length: Some(4) }")));
        assert!(is_binary_type_name(Some("Varbinary { length: None }")));
        assert!(is_binary_type_name(Some("LongVarbinary { length: None }")));
        assert!(!is_binary_type_name(Some("Integer")));
    }
}
