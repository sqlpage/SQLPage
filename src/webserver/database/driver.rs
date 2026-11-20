use chrono::{DateTime, Utc};

use super::{Parameter, execute_queries::QueryEvent, sql_to_json::DecodeRow};
use futures_util::{StreamExt, future::BoxFuture, stream::BoxStream};
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::pool::PoolConnection;
use sqlx::{
    Arguments, AssertSqlSafe, ColumnIndex, Database, Either, Error, Execute, Executor, MySql, Pool,
    Postgres, Row, SqlSafeStr, SqlStr, Sqlite, Type, encode::Encode, error::BoxDynError,
};
use sqlx_odbc::Odbc;
use sqlx_sqlserver::Mssql;

/// The driver selected by the database URL, independently of the DBMS behind ODBC.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatabaseDriver {
    Postgres,
    MySql,
    Sqlite,
    Mssql,
    Odbc,
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

#[allow(clippy::large_enum_variant)]
pub enum DatabaseConnection {
    Postgres(PoolConnection<Postgres>),
    MySql(PoolConnection<MySql>),
    Sqlite(PoolConnection<Sqlite>),
    Mssql(PoolConnection<Mssql>),
    Odbc(PoolConnection<Odbc>),
}

impl DatabasePool {
    // Pool acquisition combines the futures of all the drivers. Keep that
    // state off the stack of recursively executed SQL files.
    pub fn acquire(&self) -> BoxFuture<'_, Result<DatabaseConnection, Error>> {
        Box::pin(async {
            Ok(match self {
                Self::Postgres(pool) => DatabaseConnection::Postgres(pool.acquire().await?),
                Self::MySql(pool) => DatabaseConnection::MySql(pool.acquire().await?),
                Self::Sqlite(pool) => DatabaseConnection::Sqlite(pool.acquire().await?),
                Self::Mssql(pool) => DatabaseConnection::Mssql(pool.acquire().await?),
                Self::Odbc(pool) => DatabaseConnection::Odbc(pool.acquire().await?),
            })
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

    pub(super) async fn execute_with(
        &self,
        sql: &str,
        params: &[Parameter<'_>],
    ) -> Result<(), Error> {
        self.acquire().await?.execute_with(sql, params).await
    }

    pub(super) async fn exists(&self, sql: &str, params: &[Parameter<'_>]) -> Result<bool, Error> {
        let mut conn = self.acquire().await?;
        dispatch!(DatabaseConnection, &mut conn, conn =>
            (&mut **conn).fetch_optional(BoundStatement::typed(sql, params)).await.map(|row| row.is_some())
        )
    }

    pub(super) async fn fetch_bytes(
        &self,
        sql: &str,
        params: &[Parameter<'_>],
    ) -> Result<Option<Vec<u8>>, Error> {
        match self.acquire().await? {
            DatabaseConnection::Postgres(mut conn) => {
                bytes::<Postgres>(&mut conn, sql, params).await
            }
            DatabaseConnection::MySql(mut conn) => bytes::<MySql>(&mut conn, sql, params).await,
            DatabaseConnection::Sqlite(mut conn) => bytes::<Sqlite>(&mut conn, sql, params).await,
            DatabaseConnection::Mssql(mut conn) => bytes::<Mssql>(&mut conn, sql, params).await,
            DatabaseConnection::Odbc(mut conn) => bytes::<Odbc>(&mut conn, sql, params).await,
        }
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

impl DatabaseConnection {
    pub(super) fn execute<'a>(&'a mut self, sql: &'a str) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            dispatch!(Self, self, conn => (&mut **conn).execute(AssertSqlSafe(sql)).await.map(drop))
        })
    }

    pub(super) fn execute_with<'a>(
        &'a mut self,
        sql: &'a str,
        params: &'a [Parameter<'a>],
    ) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            dispatch!(Self, self, conn =>
                (&mut **conn).execute(BoundStatement::typed(sql, params)).await.map(drop)
            )
        })
    }

    pub(super) fn fetch_many<'c>(
        &'c mut self,
        sql: &'c str,
        params: &'c [Option<String>],
        json_inputs: &'c [bool],
    ) -> BoxStream<'c, Result<QueryEvent, Error>> {
        let statement = BoundStatement {
            sql,
            params: Parameters::Text(params),
        };
        match self {
            Self::Postgres(conn) => fetch::<Postgres>(conn, statement, json_inputs),
            Self::MySql(conn) => fetch::<MySql>(conn, statement, json_inputs),
            Self::Sqlite(conn) => fetch::<Sqlite>(conn, statement, json_inputs),
            Self::Mssql(conn) => fetch::<Mssql>(conn, statement, json_inputs),
            Self::Odbc(conn) => fetch::<Odbc>(conn, statement, json_inputs),
        }
    }
}

async fn bytes<DB>(
    conn: &mut DB::Connection,
    sql: &str,
    params: &[Parameter<'_>],
) -> Result<Option<Vec<u8>>, Error>
where
    DB: Database,
    for<'c> &'c mut DB::Connection: Executor<'c, Database = DB>,
    usize: ColumnIndex<DB::Row>,
    Vec<u8>: Type<DB> + for<'r> sqlx::Decode<'r, DB>,
    for<'q> BoundStatement<'q>: Execute<'q, DB>,
{
    conn.fetch_optional(BoundStatement::typed(sql, params))
        .await?
        .map(|row| row.try_get(0))
        .transpose()
}

fn fetch<'c, DB>(
    conn: &'c mut DB::Connection,
    statement: BoundStatement<'c>,
    json_inputs: &'c [bool],
) -> BoxStream<'c, Result<QueryEvent, Error>>
where
    DB: Database,
    for<'e> &'e mut DB::Connection: Executor<'e, Database = DB>,
    BoundStatement<'c>: Execute<'c, DB>,
    DB::Row: DecodeRow,
    usize: ColumnIndex<DB::Row>,
{
    conn.fetch_many(statement)
        .map(move |result| match result? {
            Either::Left(_) => Ok(QueryEvent::FinishedQuery),
            Either::Right(row) => row.query_row(json_inputs).map(QueryEvent::Row),
        })
        .boxed()
}

struct BoundStatement<'q> {
    sql: &'q str,
    params: Parameters<'q>,
}

enum Parameters<'q> {
    Text(&'q [Option<String>]),
    Typed(&'q [Parameter<'q>]),
}

impl<'q> BoundStatement<'q> {
    fn typed(sql: &'q str, params: &'q [Parameter<'q>]) -> Self {
        Self {
            sql,
            params: Parameters::Typed(params),
        }
    }
}

impl<'q, DB> Execute<'q, DB> for BoundStatement<'q>
where
    DB: Database,
    Option<&'q str>: Encode<'q, DB> + Type<DB>,
    &'q [u8]: Encode<'q, DB> + Type<DB>,
    DateTime<Utc>: Encode<'q, DB> + Type<DB>,
{
    fn sql(self) -> SqlStr {
        AssertSqlSafe(self.sql).into_sql_str()
    }

    fn statement(&self) -> Option<&DB::Statement> {
        None
    }

    fn take_arguments(&mut self) -> Result<Option<DB::Arguments>, BoxDynError> {
        let mut arguments = DB::Arguments::default();
        match self.params {
            Parameters::Text([]) | Parameters::Typed([]) => return Ok(None),
            Parameters::Text(params) => {
                for param in params {
                    arguments.add(param.as_deref())?;
                }
            }
            Parameters::Typed(params) => {
                for param in params {
                    match param {
                        Parameter::Text(value) => arguments.add(*value)?,
                        Parameter::Bytes(value) => arguments.add(*value)?,
                        Parameter::Timestamp(value) => arguments.add(*value)?,
                    }
                }
            }
        }
        Ok(Some(arguments))
    }

    fn persistent(&self) -> bool {
        true
    }
}
