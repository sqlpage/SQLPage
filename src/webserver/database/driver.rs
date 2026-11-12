use futures_util::StreamExt;
use futures_util::stream::BoxStream;
use serde_json::Value;
use sqlx::Either;
use sqlx::any::{Any, AnyArguments, AnyPool, AnyQueryResult, AnyRow};
use sqlx::arguments::Arguments;
use sqlx::column::Column;
use sqlx::executor::{Execute, Executor};
use sqlx::pool::PoolConnection;
use sqlx::row::Row;
use sqlx::value::ValueRef;

#[cfg(feature = "unstable-sqlx-next")]
use super::sqlx_next;

#[derive(Clone, Debug)]
pub enum DatabasePool {
    OldApi(AnyPool),
    #[cfg(feature = "unstable-sqlx-next")]
    SqlxNext(sqlx_next::Pool),
}

#[cfg_attr(
    feature = "unstable-sqlx-next",
    allow(
        clippy::large_enum_variant,
        reason = "boxing the default driver's connection would allocate on every acquire"
    )
)]
pub enum PooledConnection {
    OldApi(PoolConnection<Any>),
    #[cfg(feature = "unstable-sqlx-next")]
    SqlxNext(sqlx_next::PooledConnection),
}

/// One physical result of a statement, with each column already decoded to JSON.
pub(super) enum FetchedItem {
    Row(Vec<(String, Value)>),
    FinishedQuery,
}

#[derive(Debug)]
pub enum DriverError {
    OldApi(sqlx::error::Error),
    #[cfg(feature = "unstable-sqlx-next")]
    SqlxNext(sqlx_next::Error),
}

impl DatabasePool {
    pub(super) async fn acquire(&self) -> anyhow::Result<PooledConnection> {
        match self {
            Self::OldApi(pool) => Ok(PooledConnection::OldApi(pool.acquire().await?)),
            #[cfg(feature = "unstable-sqlx-next")]
            Self::SqlxNext(pool) => Ok(PooledConnection::SqlxNext(pool.acquire().await?)),
        }
    }

    #[must_use]
    pub fn size(&self) -> u32 {
        match self {
            Self::OldApi(pool) => pool.size(),
            #[cfg(feature = "unstable-sqlx-next")]
            Self::SqlxNext(pool) => pool.size(),
        }
    }

    #[must_use]
    pub fn num_idle(&self) -> usize {
        match self {
            Self::OldApi(pool) => pool.num_idle(),
            #[cfg(feature = "unstable-sqlx-next")]
            Self::SqlxNext(pool) => pool.num_idle(),
        }
    }

    pub(super) async fn close(&self) {
        match self {
            Self::OldApi(pool) => pool.close().await,
            #[cfg(feature = "unstable-sqlx-next")]
            Self::SqlxNext(pool) => pool.close().await,
        }
    }

    #[must_use]
    pub fn as_old_api(&self) -> Option<&AnyPool> {
        match self {
            Self::OldApi(pool) => Some(pool),
            #[cfg(feature = "unstable-sqlx-next")]
            Self::SqlxNext(_) => None,
        }
    }
}

impl PooledConnection {
    /// Statements without parameters use the simple query protocol, so they may hold several statements.
    pub(super) fn fetch_many<'c>(
        &'c mut self,
        sql: &'c str,
        params: &'c [Option<String>],
    ) -> BoxStream<'c, Result<FetchedItem, DriverError>> {
        match self {
            Self::OldApi(connection) => connection
                .fetch_many(OldApiStatement { sql, params })
                .map(|result| {
                    result
                        .map(fetched_from_old_api)
                        .map_err(DriverError::OldApi)
                })
                .boxed(),
            #[cfg(feature = "unstable-sqlx-next")]
            Self::SqlxNext(connection) => connection
                .fetch_many(sql, params)
                .map(|result| result.map_err(DriverError::SqlxNext))
                .boxed(),
        }
    }

    pub(super) async fn execute(&mut self, sql: &str) -> Result<(), DriverError> {
        match self {
            Self::OldApi(connection) => connection
                .execute(sql)
                .await
                .map(drop)
                .map_err(DriverError::OldApi),
            #[cfg(feature = "unstable-sqlx-next")]
            Self::SqlxNext(connection) => {
                connection.execute(sql).await.map_err(DriverError::SqlxNext)
            }
        }
    }

    pub fn as_old_api(&mut self) -> Option<&mut PoolConnection<Any>> {
        match self {
            Self::OldApi(connection) => Some(connection),
            #[cfg(feature = "unstable-sqlx-next")]
            Self::SqlxNext(_) => None,
        }
    }
}

pub(crate) fn unsupported_by_sqlx_next(capability: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "The unstable sqlx 0.9 database drivers do not support {capability} yet. \
        Set unstable_sqlx_next to false to use it."
    )
}

#[must_use]
pub fn is_pool_timeout(error: &anyhow::Error) -> bool {
    #[cfg(feature = "unstable-sqlx-next")]
    if let Some(sqlx_next::Error::PoolTimedOut) = error.downcast_ref() {
        return true;
    }
    matches!(error.downcast_ref(), Some(sqlx::error::Error::PoolTimedOut))
}

impl DriverError {
    /// The byte offset in the SQL text where the database located the error.
    pub(super) fn offset(&self) -> Option<usize> {
        match self {
            Self::OldApi(sqlx::error::Error::Database(error)) => error.offset(),
            Self::OldApi(_) => None,
            #[cfg(feature = "unstable-sqlx-next")]
            Self::SqlxNext(_) => None,
        }
    }
}

impl From<sqlx::error::Error> for DriverError {
    fn from(error: sqlx::error::Error) -> Self {
        Self::OldApi(error)
    }
}

impl std::fmt::Display for DriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OldApi(error) => error.fmt(f),
            #[cfg(feature = "unstable-sqlx-next")]
            Self::SqlxNext(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for DriverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OldApi(error) => error.source(),
            #[cfg(feature = "unstable-sqlx-next")]
            Self::SqlxNext(error) => error.source(),
        }
    }
}

fn fetched_from_old_api(result: Either<AnyQueryResult, AnyRow>) -> FetchedItem {
    match result {
        Either::Left(result) => {
            log::debug!("Finished query with result: {result:?}");
            FetchedItem::FinishedQuery
        }
        Either::Right(row) => {
            if log::log_enabled!(log::Level::Trace) {
                trace_old_api_row(&row);
            }
            FetchedItem::Row(super::sql_to_json::decode_columns(&row))
        }
    }
}

fn trace_old_api_row(r: &AnyRow) {
    use std::fmt::Write;
    let columns = r.columns();
    let mut row_str = String::new();
    for (i, col) in columns.iter().enumerate() {
        if let Ok(value) = r.try_get_raw(i) {
            write!(
                &mut row_str,
                "[{:?} ({}): {:?}: {:?}]",
                col.name(),
                if value.is_null() { "NULL" } else { "NOT NULL" },
                col,
                value.type_info()
            )
            .unwrap();
        }
    }
    log::trace!("Received db row: {row_str}");
}

struct OldApiStatement<'q> {
    sql: &'q str,
    params: &'q [Option<String>],
}

impl<'q> Execute<'q, Any> for OldApiStatement<'q> {
    fn sql(&self) -> &'q str {
        self.sql
    }

    fn statement(&self) -> Option<&<Any as sqlx::database::HasStatement<'q>>::Statement> {
        None
    }

    fn take_arguments(&mut self) -> Option<<Any as sqlx::database::HasArguments<'q>>::Arguments> {
        if self.params.is_empty() {
            return None;
        }
        let mut arguments = AnyArguments::default();
        for param in self.params {
            match param {
                None => arguments.add(None::<String>),
                Some(value) => arguments.add(value.as_str()),
            }
        }
        Some(arguments)
    }

    fn persistent(&self) -> bool {
        // Let sqlx create a prepared statement the first time it is executed, and then reuse it.
        true
    }
}
