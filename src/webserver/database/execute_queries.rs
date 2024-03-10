use anyhow::{anyhow, Context};
use futures_util::stream::Stream;
use futures_util::StreamExt;
use std::borrow::Cow;
use std::collections::HashMap;
use std::pin::Pin;

use super::csv_import::run_csv_import;
use super::sql::{ParsedSqlFile, ParsedStatement, StmtWithParams};
use crate::webserver::database::sql_pseudofunctions::extract_req_param;
use crate::webserver::database::sql_to_json::row_to_string;
use crate::webserver::http::SingleOrVec;
use crate::webserver::http_request_info::RequestInfo;

use sqlx::any::{AnyArguments, AnyQueryResult, AnyRow, AnyStatement, AnyTypeInfo};
use sqlx::pool::PoolConnection;
use sqlx::{Any, AnyConnection, Arguments, Either, Executor, Statement};

use serde_json::Value as JsonValue;

use super::sql_pseudofunctions::StmtParam;
use super::{highlight_sql_error, Database, DbItem};

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
            .map_err(|e| highlight_sql_error("Failed to prepare SQL statement", query, e))
    }
}

/// the raw query results can include (potentially nested) rows with a 'component' column that has the value 'dynamic'.
/// in that case we need to parse the JSON in the 'properties' column, and emit a row for each value in the resulting json array.
pub fn parse_dynamic_rows(db_item: DbItem) -> Box<dyn Iterator<Item = DbItem>> {
    if let DbItem::Row(mut row) = db_item {
        if let Some(properties) = extract_dynamic_properties(&mut row) {
            match dynamic_properties_to_iter(properties) {
                Ok(iter) => Box::new(iter.map(DbItem::Row)),
                Err(e) => Box::new(std::iter::once(DbItem::Error(e))),
            }
        } else {
            Box::new(std::iter::once(DbItem::Row(row)))
        }
    } else {
        Box::new(std::iter::once(db_item))
    }
}

/// if row.component == 'dynamic', return Some(row.properties), otherwise return None
fn extract_dynamic_properties(data: &mut JsonValue) -> Option<JsonValue> {
    let component = data.get("component").and_then(|v| v.as_str());
    if component == Some("dynamic") {
        let properties = data.get_mut("properties").map(JsonValue::take);
        Some(properties.unwrap_or_default())
    } else {
        None
    }
}

fn dynamic_properties_to_iter(
    mut properties_obj: JsonValue,
) -> anyhow::Result<Box<dyn Iterator<Item = JsonValue>>> {
    if let JsonValue::String(s) = properties_obj {
        properties_obj = serde_json::from_str::<JsonValue>(&s).with_context(|| {
            format!(
                "Unable to parse the 'properties' property of the dynamic component as JSON.\n\
                    Invalid json: {s}"
            )
        })?;
    }
    match properties_obj {
        obj @ JsonValue::Object(_) => Ok(Box::new(std::iter::once(obj))),
        JsonValue::Array(values) => Ok(Box::new(values.into_iter())),
        other => anyhow::bail!(
            "Dynamic component expected properties of type array or object, got {other} instead."
        ),
    }
}

pub fn stream_query_results<'a>(
    db: &'a Database,
    sql_file: &'a ParsedSqlFile,
    request: &'a mut RequestInfo,
) -> impl Stream<Item = DbItem> + 'a {
    async_stream::try_stream! {
        let mut connection_opt = None;
        for res in &sql_file.statements {
            match res {
                ParsedStatement::CsvImport(csv_import) => {
                    let connection = take_connection(db, &mut connection_opt).await?;
                    log::debug!("Executing CSV import: {:?}", csv_import);
                    run_csv_import(connection, csv_import, request).await?;
                },
                ParsedStatement::StmtWithParams(stmt) => {
                    let query = bind_parameters(stmt, request).await?;
                    let connection = take_connection(db, &mut connection_opt).await?;
                    log::trace!("Executing query {:?}", query.sql);
                    let mut stream = connection.fetch_many(query);
                    while let Some(elem) = stream.next().await {
                        let is_err = elem.is_err();
                        for i in parse_dynamic_rows(parse_single_sql_result(&stmt.query, elem)) {
                            yield i;
                        }
                        if is_err {
                            break;
                        }
                    }
                },
                ParsedStatement::SetVariable { variable, value} => {
                    execute_set_variable_query(db, &mut connection_opt, request, variable, value).await
                    .with_context(||
                        format!("Failed to set the {variable:?} variable to {value:?}")
                    )?;
                },
                ParsedStatement::StaticSimpleSelect(value) => {
                    for i in parse_dynamic_rows(DbItem::Row(value.clone().into())) {
                        yield i;
                    }
                }
                ParsedStatement::Error(e) => yield DbItem::Error(clone_anyhow_err(e)),
            }
        }
    }
    .map(|res| res.unwrap_or_else(DbItem::Error))
}

/// This function is used to create a pinned boxed stream of query results.
/// This allows recursive calls.
pub fn stream_query_results_boxed<'a>(
    db: &'a Database,
    sql_file: &'a ParsedSqlFile,
    request: &'a mut RequestInfo,
) -> Pin<Box<dyn Stream<Item = DbItem> + 'a>> {
    Box::pin(stream_query_results(db, sql_file, request))
}

async fn execute_set_variable_query<'a>(
    db: &'a Database,
    connection_opt: &mut Option<PoolConnection<sqlx::Any>>,
    request: &'a mut RequestInfo,
    variable: &StmtParam,
    statement: &StmtWithParams,
) -> anyhow::Result<()> {
    let query = bind_parameters(statement, request).await?;
    let connection = take_connection(db, connection_opt).await?;
    log::debug!(
        "Executing query to set the {variable:?} variable: {:?}",
        query.sql
    );
    let value: Option<String> = connection
        .fetch_optional(query)
        .await?
        .as_ref()
        .and_then(row_to_string);
    let (vars, name) = vars_and_name(request, variable)?;
    if let Some(value) = value {
        log::debug!("Setting variable {name} to {value:?}");
        vars.insert(name.clone(), SingleOrVec::Single(value));
    } else {
        log::debug!("Removing variable {name}");
        vars.remove(&name);
    }
    Ok(())
}

fn vars_and_name<'a>(
    request: &'a mut RequestInfo,
    variable: &StmtParam,
) -> anyhow::Result<(&'a mut HashMap<String, SingleOrVec>, String)> {
    match variable {
        StmtParam::Get(name) | StmtParam::GetOrPost(name) => {
            let vars = &mut request.get_variables;
            Ok((vars, name.clone()))
        }
        StmtParam::Post(name) => {
            let vars = &mut request.post_variables;
            Ok((vars, name.clone()))
        }
        _ => Err(anyhow!(
            "Only GET and POST variables can be set, not {variable:?}"
        )),
    }
}

async fn take_connection<'a, 'b>(
    db: &'a Database,
    conn: &'b mut Option<PoolConnection<sqlx::Any>>,
) -> anyhow::Result<&'b mut AnyConnection> {
    match conn {
        Some(c) => Ok(c),
        None => match db.connection.acquire().await {
            Ok(c) => {
                log::debug!("Acquired a database connection");
                *conn = Some(c);
                Ok(conn.as_mut().unwrap())
            }
            Err(e) => {
                let err_msg = format!("Unable to acquire a database connection to execute the SQL file. All of the {} {:?} connections are busy.", db.connection.size(), db.connection.any_kind());
                Err(anyhow::Error::new(e).context(err_msg))
            }
        },
    }
}

#[inline]
fn parse_single_sql_result(sql: &str, res: sqlx::Result<Either<AnyQueryResult, AnyRow>>) -> DbItem {
    match res {
        Ok(Either::Right(r)) => DbItem::Row(super::sql_to_json::row_to_json(&r)),
        Ok(Either::Left(res)) => {
            log::debug!("Finished query with result: {:?}", res);
            DbItem::FinishedQuery
        }
        Err(err) => DbItem::Error(highlight_sql_error(
            "Failed to execute SQL statement",
            sql,
            err,
        )),
    }
}

fn clone_anyhow_err(err: &anyhow::Error) -> anyhow::Error {
    let mut e = anyhow!("SQLPage could not parse and prepare this SQL statement");
    for c in err.chain().rev() {
        e = e.context(c.to_string());
    }
    e
}

async fn bind_parameters<'a>(
    stmt: &'a StmtWithParams,
    request: &'a RequestInfo,
) -> anyhow::Result<StatementWithParams<'a>> {
    let sql = stmt.query.as_str();
    log::debug!("Preparing statement: {}", sql);
    let mut arguments = AnyArguments::default();
    for (param_idx, param) in stmt.params.iter().enumerate() {
        log::trace!("\tevaluating parameter {}: {:?}", param_idx + 1, param);
        let argument = extract_req_param(param, request).await?;
        log::debug!(
            "\tparameter {}: {}",
            param_idx + 1,
            argument.as_ref().unwrap_or(&Cow::Borrowed("NULL"))
        );
        match argument {
            None => arguments.add(None::<String>),
            Some(Cow::Owned(s)) => arguments.add(s),
            Some(Cow::Borrowed(v)) => arguments.add(v),
        }
    }
    Ok(StatementWithParams { sql, arguments })
}

pub struct StatementWithParams<'a> {
    sql: &'a str,
    arguments: AnyArguments<'a>,
}

impl<'q> sqlx::Execute<'q, Any> for StatementWithParams<'q> {
    fn sql(&self) -> &'q str {
        self.sql
    }

    fn statement(&self) -> Option<&<Any as sqlx::database::HasStatement<'q>>::Statement> {
        None
    }

    fn take_arguments(&mut self) -> Option<<Any as sqlx::database::HasArguments<'q>>::Arguments> {
        Some(std::mem::take(&mut self.arguments))
    }

    fn persistent(&self) -> bool {
        // Let sqlx create a prepared statement the first time it is executed, and then reuse it.
        true
    }
}
