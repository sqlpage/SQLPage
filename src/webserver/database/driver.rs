//! Native pools share an executor that decodes each row before erasing its driver type.

use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::{StreamExt, future::BoxFuture, stream, stream::BoxStream};
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::pool::PoolConnection;
use sqlx::{
    Arguments, AssertSqlSafe, ColumnIndex, Database, Decode, Either, Encode, Error, Executor, Pool,
    Row, Type,
};

use super::{execute_queries::QueryEvent, sql_to_json::DecodeRow};

#[derive(Clone, Copy)]
pub enum Parameter<'a> {
    Text(Option<&'a str>),
    Bytes(&'a [u8]),
    Timestamp(DateTime<Utc>),
}

#[async_trait]
pub trait DatabasePool: Send + Sync {
    async fn acquire(&self) -> Result<Box<dyn DatabaseConnection>, Error>;
    async fn close(&self);
    fn size(&self) -> u32;
    fn num_idle(&self) -> usize;
    async fn migrate(&self, migrator: &Migrator) -> Result<(), MigrateError>;
}

type Migrate<DB> =
    for<'a> fn(&'a Pool<DB>, &'a Migrator) -> BoxFuture<'a, Result<(), MigrateError>>;

pub(super) struct SqlxPool<DB: Database> {
    pool: Pool<DB>,
    migrate: Option<Migrate<DB>>,
}

impl<DB: Database> SqlxPool<DB>
where
    PoolConnection<DB>: DatabaseConnection,
    DB::Connection: sqlx::migrate::Migrate,
{
    pub(super) fn native(pool: Pool<DB>) -> Arc<dyn DatabasePool> {
        Arc::new(Self {
            pool,
            migrate: Some(|pool, migrator| Box::pin(migrator.run(pool))),
        })
    }
}

impl SqlxPool<sqlx_odbc::Odbc> {
    pub(super) fn odbc(pool: Pool<sqlx_odbc::Odbc>) -> Arc<dyn DatabasePool> {
        Arc::new(Self {
            pool,
            migrate: None,
        })
    }
}

#[async_trait]
impl<DB: Database> DatabasePool for SqlxPool<DB>
where
    PoolConnection<DB>: DatabaseConnection,
{
    async fn acquire(&self) -> Result<Box<dyn DatabaseConnection>, Error> {
        Ok(Box::new(self.pool.acquire().await?))
    }
    async fn close(&self) {
        self.pool.close().await;
    }
    fn size(&self) -> u32 {
        self.pool.size()
    }
    fn num_idle(&self) -> usize {
        self.pool.num_idle()
    }

    async fn migrate(&self, migrator: &Migrator) -> Result<(), MigrateError> {
        let migrate = self
            .migrate
            .ok_or_else(|| Error::Protocol("SQL migrations are unavailable over ODBC".into()))?;
        migrate(&self.pool, migrator).await
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

impl dyn DatabasePool {
    pub async fn execute(&self, sql: &str) -> Result<(), Error> {
        self.acquire().await?.execute(sql).await
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
