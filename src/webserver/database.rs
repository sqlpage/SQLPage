use anyhow::Context;
use futures_util::stream::{self, BoxStream, Stream};
use futures_util::StreamExt;
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};
use std::future::ready;
use std::path::Path;

use crate::utils::add_value_to_map;
use crate::webserver::http::RequestInfo;
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
    let migrator = Migrator::new(migrations_dir)
        .await
        .with_context(|| migration_err("preparing the database migration"))?;
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
    sql_source: &'a [u8],
    request: &'a RequestInfo,
) -> impl Stream<Item = DbItem> + 'a {
    stream_query_results_direct(db, sql_source, request)
        .await
        .unwrap_or_else(|error| stream::once(ready(Err(error))).boxed())
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
    request: &'a RequestInfo,
) -> anyhow::Result<BoxStream<'a, anyhow::Result<Either<AnyQueryResult, AnyRow>>>> {
    let src = std::str::from_utf8(sql_source)?;
    // TODO: cache prepared statements for file
    let statements = sql::prepare_statements(db, src).await?;
    Ok(async_stream::stream! {
        for res in statements.into_iter() {
            match res {
                Ok(stmt)=>{
                    let query = bind_parameters(&stmt, request);
                    let mut stream = query.fetch_many(&db.connection);
                    while let Some(elem) = stream.next().await {
                        yield elem.with_context(|| format!("Error while running SQL: {}", stmt))
                    }
                },
                Err(e) => yield Err(anyhow::Error::from(e)),
            }
        }
    }
    .boxed())
}

fn bind_parameters<'a>(
    stmt: &'a PreparedStatement,
    request: &RequestInfo,
) -> Query<'a, sqlx::Any, AnyArguments<'a>> {
    let mut arguments = AnyArguments::default();
    for param in &stmt.parameters {
        let argument = match param {
            StmtParam::GetParam(x) => request.get_variables.get(x).cloned(),
            StmtParam::PostParam(x) => request.post_variables.get(x).cloned(),
            StmtParam::GetOrPostParam(x) => request
                .post_variables
                .get(x)
                .or_else(|| request.get_variables.get(x))
                .cloned(),
        };
        log::debug!(
            "Binding value {} in statement {}",
            argument.clone().unwrap_or_default(),
            stmt
        );
        match argument {
            None | Some(Value::Null) => arguments.add(None::<bool>),
            Some(Value::Bool(b)) => arguments.add(b),
            Some(Value::Number(n)) => {
                if let Some(int_n) = n.as_i64() {
                    arguments.add(int_n)
                } else {
                    arguments.add(n.as_f64().unwrap_or(f64::NAN))
                }
            }
            Some(Value::String(s)) => arguments.add(s),
            Some(other) => arguments.add(other.to_string()),
        }
    }
    stmt.statement.query_with(arguments)
}

pub enum DbItem {
    Row(Value),
    FinishedQuery,
    Error(anyhow::Error),
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
    log::debug!(
        "Connecting to a {:?} database on {}",
        connect_options.kind(),
        database_url
    );
    let connection = AnyPool::connect_with(connect_options)
        .await
        .with_context(|| "Failed to connect to database")?;
    Ok(Database { connection })
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
    GetParam(String),
    PostParam(String),
    GetOrPostParam(String),
}

mod sql {
    use super::PreparedStatement;
    use crate::webserver::database::StmtParam;
    use anyhow::Context;
    use sqlparser::ast::{visitor_fn_mut, DataType, DriveMut, Expr, Value, VisitorEvent};
    use sqlparser::dialect::GenericDialect;
    use sqlparser::parser::Parser;
    use sqlx::any::{AnyKind, AnyTypeInfo};
    use sqlx::postgres::types::Oid;
    use sqlx::postgres::PgTypeInfo;
    use sqlx::{Executor, Statement};

    pub(super) async fn prepare_statements(
        db: &super::Database,
        sql: &str,
    ) -> anyhow::Result<Vec<anyhow::Result<PreparedStatement>>> {
        let dialect = GenericDialect {};
        let ast = Parser::parse_sql(&dialect, sql)?;
        let db_kind = db.connection.any_kind();
        let mut res = Vec::with_capacity(ast.len());
        for mut stmt in ast {
            let param_names = extract_parameters(&mut stmt, db_kind);
            let parameters = map_params(param_names);
            let query = stmt.to_string();
            let param_types = get_param_types(&parameters);
            let stmt_res = db
                .connection
                .prepare_with(&query, &param_types)
                .await
                .with_context(|| format!("Parsing SQL string: '{}'", query));
            res.push(stmt_res.map(|statement| PreparedStatement {
                statement: statement.to_owned(),
                parameters,
            }));
        }
        Ok(res)
    }

    fn get_param_types(parameters: &[StmtParam]) -> Vec<AnyTypeInfo> {
        parameters
            .iter()
            .map(|_p| PgTypeInfo::with_oid(Oid(25)).into())
            .collect()
    }

    fn map_params(names: Vec<String>) -> Vec<StmtParam> {
        names
            .into_iter()
            .map(|name| {
                let (prefix, name) = name.split_at(1);
                let name = name.to_owned();
                match prefix {
                    "$" => StmtParam::GetOrPostParam(name),
                    ":" => StmtParam::PostParam(name),
                    _ => StmtParam::GetParam(name),
                }
            })
            .collect()
    }

    fn extract_parameters(sql_ast: &mut sqlparser::ast::Statement, db: AnyKind) -> Vec<String> {
        let mut parameters: Vec<String> = Vec::new();
        sql_ast.drive_mut(&mut visitor_fn_mut(|value: &mut Expr, event| {
            // Only update the nodes AFTER they have been visited
            if let VisitorEvent::Enter = event {
                return;
            }
            if let Expr::Value(Value::Placeholder(param)) = value {
                let new_expr = make_placeholder(db, parameters.len());
                let name = std::mem::take(param);
                parameters.push(name);
                *value = new_expr
            }
        }));
        parameters
    }

    fn make_placeholder(db: AnyKind, current_count: usize) -> Expr {
        let name = match db {
            // Postgres only supports numbered parameters
            AnyKind::Postgres => format!("${}", current_count + 1),
            _ => format!("?"),
        };
        let data_type = match db {
            // MySQL requires CAST(? AS CHAR) and does not understand CAST(? AS TEXT)
            AnyKind::MySql => DataType::Char(None),
            _ => DataType::Text,
        };
        let value = Expr::Value(Value::Placeholder(name));
        Expr::Cast {
            expr: Box::new(value),
            data_type,
        }
    }

    #[test]
    fn test_statement_rewrite() {
        let sql = "select $a from t where $x > $a OR $x = 0";
        let mut ast = Parser::parse_sql(&GenericDialect, sql).unwrap();
        let parameters = extract_parameters(&mut ast[0], AnyKind::Postgres);
        assert_eq!(
            ast[0].to_string(),
            "SELECT CAST($1 AS TEXT) FROM t WHERE CAST($2 AS TEXT) > CAST($3 AS TEXT) OR CAST($4 AS TEXT) = 0"
        );
        assert_eq!(parameters, ["$a", "$x", "$a", "$x"]);
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
