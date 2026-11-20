//! Native pools share an executor that decodes each row before erasing its driver type.

use std::any::Any;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::{StreamExt, future::BoxFuture, stream, stream::BoxStream};
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::pool::PoolConnection;
use sqlx::{
    Arguments, AssertSqlSafe, ColumnIndex, Database, Decode, Either, Encode, Error, Executor,
    MySql, Pool, Postgres, Row, Sqlite, Type,
};

use sqlx_odbc::Odbc;
use sqlx_sqlserver::Mssql;

use super::{execute_queries::QueryEvent, sql_to_json::DecodeRow};

/// The driver selected by the database URL, independently of the DBMS behind ODBC.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatabaseDriver {
    Postgres,
    MySql,
    Sqlite,
    Mssql,
    Odbc,
}

#[derive(Clone, Copy)]
pub enum Parameter<'a> {
    Text(Option<&'a str>),
    Bytes(&'a [u8]),
    Timestamp(DateTime<Utc>),
}

// Use the same dispatch for every operation that is independent of the selected driver.
macro_rules! dispatch {
    ($kind:ident, $value:expr, $binding:ident => $operation:expr) => {
        match $value {
            $kind::Postgres($binding) => $operation,
            $kind::MySql($binding) => $operation,
            $kind::Sqlite($binding) => $operation,
            $kind::Mssql($binding) => $operation,
            $kind::Odbc($binding) => $operation,
        }
    };
}

#[derive(Clone)]
pub enum DatabasePool {
    Postgres(Pool<Postgres>),
    MySql(Pool<MySql>),
    Sqlite(Pool<Sqlite>),
    Mssql(Pool<Mssql>),
    Odbc(Pool<Odbc>),
}

impl DatabasePool {
    pub fn acquire(&self) -> BoxFuture<'_, Result<Box<dyn DatabaseConnection>, Error>> {
        Box::pin(async {
            let conn: Box<dyn DatabaseConnection> =
                dispatch!(Self, self, pool => Box::new(pool.acquire().await?));
            Ok(conn)
        })
    }

    #[must_use]
    pub fn size(&self) -> u32 {
        dispatch!(Self, self, pool => pool.size())
    }

    #[must_use]
    pub fn num_idle(&self) -> usize {
        dispatch!(Self, self, pool => pool.num_idle())
    }

    pub(super) async fn close(&self) {
        dispatch!(Self, self, pool => pool.close().await);
    }

    pub async fn execute(&self, sql: &str) -> Result<(), Error> {
        dispatch!(Self, self, pool => pool.execute(AssertSqlSafe(sql)).await.map(drop))
    }

    pub(super) async fn migrate(&self, migrator: &Migrator) -> Result<(), MigrateError> {
        match self {
            Self::Postgres(pool) => migrator.run(pool).await,
            Self::MySql(pool) => migrator.run(pool).await,
            Self::Sqlite(pool) => migrator.run(pool).await,
            Self::Mssql(pool) => migrator.run(pool).await,
            Self::Odbc(_) => {
                Err(Error::Protocol("SQL migrations are unavailable over ODBC".into()).into())
            }
        }
    }
}

#[async_trait]
pub trait DatabaseConnection: Send {
    fn native(&mut self) -> &mut dyn Any;
    async fn execute_with(&mut self, sql: &str, params: &[Parameter<'_>]) -> Result<(), Error>;
    async fn exists(&mut self, sql: &str, params: &[Parameter<'_>]) -> Result<bool, Error>;
    async fn fetch_bytes(
        &mut self,
        sql: &str,
        params: &[Parameter<'_>],
    ) -> Result<Option<Vec<u8>>, Error>;

    fn fetch_many<'a>(
        &'a mut self,
        sql: &'a str,
        params: &[Option<String>],
        json_inputs: &'a [bool],
    ) -> BoxStream<'a, Result<QueryEvent, Error>>;
}

impl dyn DatabaseConnection + '_ {
    pub async fn execute(&mut self, sql: &str) -> Result<(), Error> {
        self.execute_with(sql, &[]).await
    }
}

fn bind<'a, DB: Database>(
    params: impl IntoIterator<Item = Parameter<'a>>,
) -> Result<Option<DB::Arguments>, Error>
where
    Option<&'a str>: Encode<'a, DB> + Type<DB>,
    &'a [u8]: Encode<'a, DB> + Type<DB>,
    DateTime<Utc>: Encode<'a, DB> + Type<DB>,
{
    let mut args = DB::Arguments::default();
    for param in params {
        match param {
            Parameter::Text(value) => args.add(value),
            Parameter::Bytes(value) => args.add(value),
            Parameter::Timestamp(value) => args.add(value),
        }
        .map_err(Error::Encode)?;
    }
    // None selects the simple query protocol, which also supports SQL batches.
    Ok((args.len() > 0).then_some(args))
}

#[async_trait]
impl<DB> DatabaseConnection for PoolConnection<DB>
where
    DB: Database,
    DB::Row: DecodeRow,
    usize: ColumnIndex<DB::Row>,
    for<'c> &'c mut DB::Connection: Executor<'c, Database = DB>,
    for<'a> Option<&'a str>: Encode<'a, DB> + Type<DB>,
    for<'a> &'a [u8]: Encode<'a, DB> + Type<DB>,
    DateTime<Utc>: for<'a> Encode<'a, DB> + Type<DB>,
    Vec<u8>: for<'r> Decode<'r, DB> + Type<DB>,
{
    fn native(&mut self) -> &mut dyn Any {
        &mut **self
    }

    async fn execute_with(&mut self, sql: &str, params: &[Parameter<'_>]) -> Result<(), Error> {
        let args = bind::<DB>(params.iter().copied())?;
        (&mut **self)
            .execute((AssertSqlSafe(sql), args))
            .await
            .map(drop)
    }

    async fn exists(&mut self, sql: &str, params: &[Parameter<'_>]) -> Result<bool, Error> {
        let args = bind::<DB>(params.iter().copied())?;
        Ok((&mut **self)
            .fetch_optional((AssertSqlSafe(sql), args))
            .await?
            .is_some())
    }

    async fn fetch_bytes(
        &mut self,
        sql: &str,
        params: &[Parameter<'_>],
    ) -> Result<Option<Vec<u8>>, Error> {
        let args = bind::<DB>(params.iter().copied())?;
        (&mut **self)
            .fetch_optional((AssertSqlSafe(sql), args))
            .await?
            .map(|row| row.try_get(0))
            .transpose()
    }

    fn fetch_many<'a>(
        &'a mut self,
        sql: &'a str,
        params: &[Option<String>],
        json_inputs: &'a [bool],
    ) -> BoxStream<'a, Result<QueryEvent, Error>> {
        let args = match bind::<DB>(params.iter().map(|p| Parameter::Text(p.as_deref()))) {
            Ok(args) => args,
            Err(err) => return stream::once(async { Err(err) }).boxed(),
        };
        (&mut **self)
            .fetch_many((AssertSqlSafe(sql), args))
            .map(move |item| match item? {
                Either::Left(_) => Ok(QueryEvent::FinishedQuery),
                Either::Right(row) => row.query_row(json_inputs).map(QueryEvent::Row),
            })
            .boxed()
    }
}
