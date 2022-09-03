use futures_util::stream::{self, BoxStream, Stream};
use futures_util::StreamExt;
use serde_json::{Map, Value};
use std::fmt::Display;
use std::future::ready;
use std::path::Path;
use anyhow::Context;

use crate::utils::add_value_to_map;
use crate::MIGRATIONS_DIR;
use sqlx::any::{AnyArguments, AnyConnectOptions, AnyQueryResult, AnyRow, AnyStatement};
use sqlx::migrate::Migrator;
use sqlx::query::Query;
use sqlx::{AnyPool, Arguments, Column, ConnectOptions, Decode, Either, Row, Statement};

pub struct Database {
    connection: AnyPool,
}

pub async fn apply_migrations(db: &Database) -> anyhow::Result<()> {
    let migrations_dir = Path::new(MIGRATIONS_DIR);
    if !migrations_dir.exists() {
        log::debug!(
            "Not applying database migrations because '{}' does not exist",
            MIGRATIONS_DIR
        );
        return Ok(());
    }
    let migrator = Migrator::new(migrations_dir).await.map_err(migration_err)?;
    migrator.run(&db.connection).await.map_err(migration_err)?;
    Ok(())
}

fn migration_err(e: impl Display) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::Other,
        format!(
            "An error occurred while running the database migration.
        The path '{MIGRATIONS_DIR}' has to point to a directory, which contains valid SQL files
        with names using the format '<VERSION>_<DESCRIPTION>.sql',
        where <VERSION> is a positive number, and <DESCRIPTION> is a string.
        The current state of migrations will be stored in a table called _sqlx_migrations.\n {e}"
        ),
    )
}

pub async fn stream_query_results<'a>(
    db: &'a Database,
    sql_source: &'a [u8],
    argument: &'a str,
) -> impl Stream<Item=DbItem> + 'a {
    stream_query_results_direct(db, sql_source, argument)
        .await
        .unwrap_or_else(|e| {
            let error = sqlx::Error::Decode(e);
            stream::once(ready(Err(error))).boxed()
        })
        .map(|res| match res {
            Ok(Either::Right(r)) => DbItem::Row(row_to_json(r)),
            Ok(Either::Left(res)) => {
                log::debug!("Finished query with result: {:?}", res);
                DbItem::FinishedQuery
            }
            Err(e) => DbItem::Error(e),
        })
}

pub async fn stream_query_results_direct<'a>(
    db: &'a Database,
    sql_source: &'a [u8],
    argument: &'a str,
) -> Result<
    BoxStream<'a, Result<Either<AnyQueryResult, AnyRow>, sqlx::Error>>,
    Box<dyn std::error::Error + Sync + Send>,
> {
    let src = std::str::from_utf8(sql_source)?;
    // TODO: cache prepared statements for file
    let statements = sql::prepare_statements(db, src).await?;
    Ok(async_stream::stream! {
        for res in statements.into_iter() {
            match res {
                Ok(stmt)=>{
                    let query = bind_parameters(&stmt, argument);
                    let mut stream = query.fetch_many(&db.connection);
                    while let Some(elem) = stream.next().await {
                        yield elem
                    }
                },
                Err(e) => yield Err(e),
            }
        }
    }
        .boxed())
}

fn bind_parameters<'a>(
    stmt: &'a AnyStatement,
    argument: &'a str,
) -> Query<'a, sqlx::Any, AnyArguments<'a>> {
    let num_params = stmt
        .parameters()
        .map_or(0, |e| e.either(|args| args.len(), |n| n));
    let mut arguments = AnyArguments::default();
    for _ in 0..num_params {
        arguments.add(argument);
    }
    stmt.query_with(arguments)
}

pub enum DbItem {
    Row(Value),
    FinishedQuery,
    Error(sqlx::Error),
}

fn row_to_json(row: AnyRow) -> Value {
    use sqlx::{TypeInfo, ValueRef};
    use Value::{Null, Object};

    let columns = row.columns();
    let mut map = Map::new();
    for col in columns {
        let key = col.name().to_string();
        let value: Value = match row.try_get_raw(col.ordinal()) {
            Ok(raw_value) if !raw_value.is_null() => match raw_value.type_info().name() {
                "REAL" | "FLOAT" | "NUMERIC" | "FLOAT4" | "FLOAT8" | "DOUBLE" => {
                    <f64 as Decode<sqlx::any::Any>>::decode(raw_value)
                        .unwrap_or(f64::NAN)
                        .into()
                }
                "INT8" | "BIGINT" => <i64 as Decode<sqlx::any::Any>>::decode(raw_value)
                    .unwrap_or_default()
                    .into(),
                "INT" | "INTEGER" | "INT4" => <i32 as Decode<sqlx::any::Any>>::decode(raw_value)
                    .unwrap_or_default()
                    .into(),
                "INT2" | "SMALLINT" => <i16 as Decode<sqlx::any::Any>>::decode(raw_value)
                    .unwrap_or_default()
                    .into(),
                "BOOL" | "BOOLEAN" => <bool as Decode<sqlx::any::Any>>::decode(raw_value)
                    .unwrap_or_default()
                    .into(),
                "JSON" | "JSON[]" | "JSONB" | "JSONB[]" => {
                    <&[u8] as Decode<sqlx::any::Any>>::decode(raw_value)
                        .and_then(|rv| {
                            serde_json::from_slice::<Value>(rv).map_err(|e| {
                                Box::new(e) as Box<dyn std::error::Error + Sync + Send>
                            })
                        })
                        .unwrap_or_default()
                }
                // Deserialize as a string by default
                _ => <String as Decode<sqlx::any::Any>>::decode(raw_value)
                    .unwrap_or_default()
                    .into(),
            },
            Ok(_null) => Null,
            Err(e) => {
                log::warn!("Unable to extract value from row: {:?}", e);
                Null
            }
        };
        map = add_value_to_map(map, (key, value));
    }
    Object(map)
}

pub async fn init_database(database_url: &str) -> anyhow::Result<Database> {
    let mut connect_options: AnyConnectOptions =
        database_url.parse().expect("Invalid database URL");
    connect_options.log_statements(log::LevelFilter::Trace);
    connect_options.log_slow_statements(
        log::LevelFilter::Warn,
        std::time::Duration::from_millis(250),
    );
    let connection = AnyPool::connect_with(connect_options)
        .await
        .with_context(|| "Failed to connect to database")?;
    Ok(Database { connection })
}

mod sql {
    use sqlparser::dialect::GenericDialect;
    use sqlparser::parser::Parser;
    use sqlx::any::AnyStatement;
    use sqlx::{Executor, Statement};

    pub async fn prepare_statements(
        db: &super::Database,
        sql: &str,
    ) -> anyhow::Result<Vec<sqlx::Result<AnyStatement<'static>>>> {
        let dialect = GenericDialect {};
        let ast = Parser::parse_sql(&dialect, sql)?;
        let mut res = Vec::with_capacity(ast.len());
        for stmt in ast {
            let query = stmt.to_string();
            let s = db.connection.prepare(&query).await;
            res.push(s.map(|r| r.to_owned()));
        }
        Ok(res)
    }
}

#[actix_web::test]
async fn test_row_to_json() -> anyhow::Result<()> {
    use sqlx::Connection;
    let mut c = sqlx::AnyConnection::connect("sqlite://:memory:").await?;
    let row = sqlx::query(
        "SELECT \
        3.14159 as one_value, \
        1 as two_values, \
        2 as two_values, \
        'x' as three_values, \
        'y' as three_values, \
        'z' as three_values \
    ",
    )
        .fetch_one(&mut c)
        .await?;
    assert_eq!(
        row_to_json(row),
        serde_json::json!({
            "one_value": 3.14159,
            "two_values": [1,2],
            "three_values": ["x","y","z"],
        })
    );
    Ok(())
}
