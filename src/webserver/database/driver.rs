use std::fmt;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
use std::time::Duration;

use anyhow::Context;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_rusqlite::rusqlite;

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
}

pub struct DbConnection {
    inner: NativeConnection,
    pool: Arc<DbPoolInner>,
    _permit: OwnedSemaphorePermit,
}

enum NativeConnection {
    Sqlite(tokio_rusqlite::Connection),
}

impl Drop for DbConnection {
    fn drop(&mut self) {
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
        Self {
            inner: Arc::new(DbPoolInner {
                url: config.database_url.clone(),
                kind,
                max_size,
                acquire_timeout: Duration::from_secs_f64(
                    config.database_connection_acquire_timeout_seconds,
                ),
                semaphore: Arc::new(Semaphore::new(usize::try_from(max_size).unwrap_or(usize::MAX))),
                active: AtomicU32::new(0),
                on_connect_sql: on_connect_sql.map(Arc::new),
                sqlite_extensions: config.sqlite_extensions.clone(),
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

    pub async fn close(&self) {}

    pub async fn acquire(&self) -> Result<DbConnection, DbError> {
        let permit = tokio::time::timeout(
            self.inner.acquire_timeout,
            self.inner.semaphore.clone().acquire_owned(),
        )
        .await
        .map_err(|_| DbError::PoolTimedOut)?
        .map_err(|_| DbError::PoolTimedOut)?;
        self.inner.active.fetch_add(1, Ordering::Relaxed);
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
            other => return Err(DbError::UnsupportedBackend(other)),
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
        }
    }

    pub async fn execute(
        &mut self,
        sql: &str,
        params: &[DbParam],
    ) -> Result<Vec<DbStatementResult>, DbError> {
        self.inner.execute(sql, params).await
    }

    pub async fn execute_command(
        &mut self,
        sql: &str,
        params: &[DbParam],
    ) -> Result<(), DbError> {
        let _ = self.execute(sql, params).await?;
        Ok(())
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
        }
    }

    async fn execute(
        &mut self,
        sql: &str,
        params: &[DbParam],
    ) -> Result<Vec<DbStatementResult>, DbError> {
        match self {
            Self::Sqlite(conn) => execute_sqlite(conn, sql, params).await,
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

fn sqlite_path_from_url(url: &str) -> anyhow::Result<String> {
    let Some(rest) = url.strip_prefix("sqlite:") else {
        anyhow::bail!("not a sqlite URL: {url}");
    };
    let rest = rest.strip_prefix("//").unwrap_or(rest);
    let decoded = percent_encoding::percent_decode_str(rest)
        .decode_utf8()
        .with_context(|| format!("invalid percent encoding in sqlite URL {url:?}"))?;
    Ok(decoded.into_owned())
}

async fn configure_sqlite(
    conn: &tokio_rusqlite::Connection,
    extensions: &[String],
) -> Result<(), DbError> {
    let extensions = extensions.to_vec();
    conn.call(move |conn| {
        conn.create_collation("NOCASE", |a, b| {
            a.to_lowercase().cmp(&b.to_lowercase())
        })?;
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
            stmt.execute(rusqlite::params_from_iter(values))?;
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

fn sqlite_value_from_param(param: DbParam) -> rusqlite::types::Value {
    match param {
        DbParam::Null => rusqlite::types::Value::Null,
        DbParam::Text(s) => rusqlite::types::Value::Text(s),
        DbParam::Bytes(bytes) => rusqlite::types::Value::Blob(bytes),
        DbParam::Timestamp(ts) => rusqlite::types::Value::Text(ts.to_rfc3339()),
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
