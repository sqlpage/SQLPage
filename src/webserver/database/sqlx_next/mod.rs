mod odbc;

use std::time::Duration;

use futures_util::stream::BoxStream;
use futures_util::{StreamExt, TryStreamExt};
use serde_json::Value;
use sqlx::any::AnyKind;
use sqlx_next::database::Database;
use sqlx_next::decode::Decode;
use sqlx_next::encode::Encode;
use sqlx_next::error::BoxDynError;
use sqlx_next::pool::{PoolConnection, PoolOptions};
use sqlx_next::{
    Arguments, AssertSqlSafe, ColumnIndex, Either, Execute, Executor, Row, SqlSafeStr, SqlStr, Type,
};
use sqlx_odbc::Odbc;

use super::connect::{ConnectionHooks, max_connections};
use super::driver::FetchedItem;
use crate::app_config::AppConfig;

pub(super) use sqlx_next::Error;

#[derive(Clone, Debug)]
pub enum Pool {
    Odbc(sqlx_odbc::OdbcPool),
}

pub enum PooledConnection {
    Odbc(PoolConnection<Odbc>),
}

#[derive(Clone, Copy)]
pub(super) enum Driver {
    Odbc,
}

impl Driver {
    pub(super) fn serving(kind: AnyKind) -> Option<Self> {
        match kind {
            AnyKind::Odbc => Some(Self::Odbc),
            AnyKind::Postgres | AnyKind::MySql | AnyKind::Sqlite | AnyKind::Mssql => None,
        }
    }

    /// Returns the pool and the name the database management system reports for itself.
    pub(super) async fn connect(self, config: &AppConfig) -> anyhow::Result<(Pool, String)> {
        match self {
            Self::Odbc => odbc::connect(config).await,
        }
    }
}

impl Pool {
    pub(super) async fn acquire(&self) -> Result<PooledConnection, Error> {
        Ok(match self {
            Self::Odbc(pool) => PooledConnection::Odbc(pool.acquire().await?),
        })
    }

    pub(super) fn size(&self) -> u32 {
        match self {
            Self::Odbc(pool) => pool.size(),
        }
    }

    pub(super) fn num_idle(&self) -> usize {
        match self {
            Self::Odbc(pool) => pool.num_idle(),
        }
    }

    pub(super) async fn close(&self) {
        match self {
            Self::Odbc(pool) => pool.close().await,
        }
    }
}

impl PooledConnection {
    pub(super) fn fetch_many<'c>(
        &'c mut self,
        sql: &'c str,
        params: &'c [Option<String>],
    ) -> BoxStream<'c, Result<FetchedItem, Error>> {
        match self {
            Self::Odbc(connection) => {
                fetch_decoded::<Odbc>(&mut **connection, sql, params, odbc::decode_row)
            }
        }
    }

    pub(super) async fn execute(&mut self, sql: &str) -> Result<(), Error> {
        let sql = AssertSqlSafe(sql.to_owned());
        match self {
            Self::Odbc(connection) => connection.execute(sql).await.map(drop),
        }
    }
}

fn fetch_decoded<'c, DB>(
    connection: &'c mut DB::Connection,
    sql: &'c str,
    params: &'c [Option<String>],
    decode_row: fn(&DB::Row) -> Vec<(String, Value)>,
) -> BoxStream<'c, Result<FetchedItem, Error>>
where
    DB: Database,
    for<'e> &'e mut DB::Connection: Executor<'e, Database = DB>,
    DB::QueryResult: std::fmt::Debug,
    for<'q> Option<&'q str>: Encode<'q, DB> + Type<DB>,
{
    connection
        .fetch_many(BoundStatement { sql, params })
        .map_ok(move |result| match result {
            Either::Left(result) => {
                log::debug!("Finished query with result: {result:?}");
                FetchedItem::FinishedQuery
            }
            Either::Right(row) => FetchedItem::Row(decode_row(&row)),
        })
        .boxed()
}

struct BoundStatement<'q> {
    sql: &'q str,
    params: &'q [Option<String>],
}

impl<'q, DB> Execute<'q, DB> for BoundStatement<'q>
where
    DB: Database,
    for<'a> Option<&'a str>: Encode<'a, DB> + Type<DB>,
{
    fn sql(self) -> SqlStr {
        AssertSqlSafe(self.sql).into_sql_str()
    }

    fn statement(&self) -> Option<&DB::Statement> {
        None
    }

    fn take_arguments(&mut self) -> Result<Option<DB::Arguments>, BoxDynError> {
        if self.params.is_empty() {
            return Ok(None);
        }
        let mut arguments = DB::Arguments::default();
        for param in self.params {
            arguments.add(param.as_deref())?;
        }
        Ok(Some(arguments))
    }

    fn persistent(&self) -> bool {
        true
    }
}

fn pool_options<DB>(config: &AppConfig, kind: AnyKind) -> PoolOptions<DB>
where
    DB: Database,
    for<'c> &'c mut DB::Connection: Executor<'c, Database = DB>,
    bool: Type<DB> + for<'r> Decode<'r, DB>,
    usize: ColumnIndex<DB::Row>,
    DB::QueryResult: std::fmt::Debug,
{
    let ConnectionHooks {
        on_connect,
        on_reset,
    } = ConnectionHooks::read(config);
    PoolOptions::new()
        .max_connections(max_connections(config, kind))
        .idle_timeout(config.database_connection_idle_timeout)
        .max_lifetime(config.database_connection_max_lifetime)
        .acquire_timeout(Duration::from_secs_f64(
            config.database_connection_acquire_timeout_seconds,
        ))
        .after_connect(move |connection: &mut DB::Connection, _| {
            let sql = on_connect.clone();
            Box::pin(async move {
                if let Some(sql) = sql {
                    let result = connection.execute(AssertSqlSafe(sql)).await?;
                    log::debug!(
                        "Finished running connection handler on new connection: {result:?}"
                    );
                }
                Ok(())
            })
        })
        .after_release(move |connection: &mut DB::Connection, metadata| {
            let sql = on_reset.clone();
            Box::pin(async move {
                let Some(sql) = sql else {
                    return Ok(true);
                };
                log::trace!("Running the custom SQL connection cleanup handler. {metadata:?}");
                match connection.fetch_optional(AssertSqlSafe(sql)).await? {
                    Some(row) => row.try_get::<bool, _>(0),
                    None => Ok(true),
                }
            })
        })
}

fn decode_or_default<'r, DB, T>(value: DB::ValueRef<'r>) -> T
where
    DB: Database,
    T: Decode<'r, DB> + Default,
{
    T::decode(value).unwrap_or_else(|error| {
        let type_name = std::any::type_name::<T>();
        log::error!("Failed to decode {type_name} value: {error}");
        T::default()
    })
}
