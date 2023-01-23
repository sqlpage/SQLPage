mod sql;

use anyhow::{anyhow, Context};
use futures_util::stream::{self, BoxStream, Stream};
use futures_util::StreamExt;
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};
use std::future::ready;
use std::path::Path;

use crate::utils::add_value_to_map;
use crate::webserver::http::{RequestInfo, SingleOrVec};
use crate::MIGRATIONS_DIR;
use sqlx::any::{
    AnyArguments, AnyConnectOptions, AnyQueryResult, AnyRow, AnyStatement, AnyTypeInfo,
};
use sqlx::migrate::Migrator;
use sqlx::query::Query;
use sqlx::{AnyPool, Arguments, Column, ConnectOptions, Decode, Either, Executor, Row, Statement};

pub use crate::file_cache::FileCache;
pub use sql::make_placeholder;
pub use sql::ParsedSqlFile;

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

pub async fn stream_query_results<'a>(
    db: &'a Database,
    sql_file: &'a ParsedSqlFile,
    request: &'a RequestInfo,
) -> impl Stream<Item = DbItem> + 'a {
    stream_query_results_direct(db, sql_file, request)
        .await
        .unwrap_or_else(|error| stream::once(ready(Err(error))).boxed())
        .map(|res| match res {
            Ok(Either::Right(r)) => DbItem::Row(row_to_json(&r)),
            Ok(Either::Left(res)) => {
                log::debug!("Finished query with result: {:?}", res);
                DbItem::FinishedQuery
            }
            Err(e) => DbItem::Error(e),
        })
}

pub async fn stream_query_results_direct<'a>(
    db: &'a Database,
    sql_file: &'a ParsedSqlFile,
    request: &'a RequestInfo,
) -> anyhow::Result<BoxStream<'a, anyhow::Result<Either<AnyQueryResult, AnyRow>>>> {
    Ok(async_stream::stream! {
        for res in sql_file.statements.iter() {
            match res {
                Ok(stmt)=>{
                    let query = bind_parameters(stmt, request);
                    let mut stream = query.fetch_many(&db.connection);
                    while let Some(elem) = stream.next().await {
                        yield elem.with_context(|| format!("Error while running SQL: {}", stmt))
                    }
                },
                Err(e) => yield Err(clone_anyhow_err(e)),
            }
        }
    }
    .boxed())
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
) -> Query<'a, sqlx::Any, AnyArguments<'a>> {
    let mut arguments = AnyArguments::default();
    for param in &stmt.parameters {
        let argument = match param {
            StmtParam::Get(x) => request.get_variables.get(x),
            StmtParam::Post(x) => request.post_variables.get(x),
            StmtParam::GetOrPost(x) => request
                .post_variables
                .get(x)
                .or_else(|| request.get_variables.get(x)),
        };
        log::debug!("Binding value {:?} in statement {}", &argument, stmt);
        match argument {
            None => arguments.add(None::<String>),
            Some(SingleOrVec::Single(s)) => arguments.add(s),
            Some(SingleOrVec::Vec(v)) => {
                arguments.add(serde_json::to_string(v).unwrap_or_default());
            }
        }
    }
    stmt.statement.query_with(arguments)
}

pub enum DbItem {
    Row(Value),
    FinishedQuery,
    Error(anyhow::Error),
}

fn row_to_json(row: &AnyRow) -> Value {
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

impl Database {
    pub async fn init(database_url: &str) -> anyhow::Result<Self> {
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
        let connection = AnyPool::connect_with(connect_options)
            .await
            .with_context(|| format!("Unable to open connection to {database_url}"))?;
        Ok(Database { connection })
    }
}

struct PreparedStatement {
    statement: AnyStatement<'static>,
    parameters: Vec<StmtParam>,
}

impl Display for PreparedStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.statement.sql())
    }
}

enum StmtParam {
    Get(String),
    Post(String),
    GetOrPost(String),
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
        row_to_json(&row),
        serde_json::json!({
            "one_value": 3.14159,
            "two_values": [1,2],
            "three_values": ["x","y","z"],
        })
    );
    Ok(())
}
