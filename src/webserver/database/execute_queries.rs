use anyhow::{anyhow, Context};
use futures_util::stream::Stream;
use futures_util::StreamExt;
use std::borrow::Cow;
use std::collections::HashMap;
use std::pin::Pin;

use super::csv_import::run_csv_import;
use super::sql::{ParsedSqlFile, ParsedStatement, SimpleSelectValue, StmtWithParams};
use crate::dynamic_component::parse_dynamic_rows;
use crate::utils::add_value_to_map;
use crate::webserver::database::sql_to_json::row_to_string;
use crate::webserver::http::SingleOrVec;
use crate::webserver::http_request_info::RequestInfo;

use super::syntax_tree::{extract_req_param, StmtParam};
use super::{highlight_sql_error, Database, DbItem};
use sqlx::any::{AnyArguments, AnyQueryResult, AnyRow, AnyStatement, AnyTypeInfo};
use sqlx::pool::PoolConnection;
use sqlx::{Any, Arguments, Column, Either, Executor, Row as _, Statement, ValueRef};

pub type DbConn = Option<PoolConnection<sqlx::Any>>;

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

pub fn stream_query_results_with_conn<'a>(
    sql_file: &'a ParsedSqlFile,
    request: &'a mut RequestInfo,
    db_connection: &'a mut DbConn,
) -> impl Stream<Item = DbItem> + 'a {
    async_stream::try_stream! {
        for res in &sql_file.statements {
            match res {
                ParsedStatement::CsvImport(csv_import) => {
                    let connection = take_connection(&request.app_state.db, db_connection).await?;
                    log::debug!("Executing CSV import: {:?}", csv_import);
                    run_csv_import(connection, csv_import, request).await?;
                },
                ParsedStatement::StmtWithParams(stmt) => {
                    let query = bind_parameters(stmt, request, db_connection).await?;
                    let connection = take_connection(&request.app_state.db, db_connection).await?;
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
                    execute_set_variable_query(db_connection, request, variable, value).await
                    .with_context(||
                        format!("Failed to set the {variable} variable to {value:?}")
                    )?;
                },
                ParsedStatement::StaticSimpleSelect(value) => {
                    for i in parse_dynamic_rows(DbItem::Row(exec_static_simple_select(value, request, db_connection).await?)) {
                        yield i;
                    }
                }
                ParsedStatement::Error(e) => yield DbItem::Error(clone_anyhow_err(e)),
            }
        }
    }
    .map(|res| res.unwrap_or_else(DbItem::Error))
}

/// Executes the sqlpage pseudo-functions contained in a static simple select
async fn exec_static_simple_select(
    columns: &[(String, SimpleSelectValue)],
    req: &RequestInfo,
    db_connection: &mut DbConn,
) -> anyhow::Result<serde_json::Value> {
    let mut map = serde_json::Map::with_capacity(columns.len());
    for (name, value) in columns {
        let value = match value {
            SimpleSelectValue::Static(s) => s.clone(),
            SimpleSelectValue::Dynamic(p) => {
                extract_req_param_as_json(p, req, db_connection).await?
            }
        };
        map = add_value_to_map(map, (name.clone(), value));
    }
    Ok(serde_json::Value::Object(map))
}

/// Extracts the value of a parameter from the request.
/// Returns `Ok(None)` when NULL should be used as the parameter value.
async fn extract_req_param_as_json(
    param: &StmtParam,
    request: &RequestInfo,
    db_connection: &mut DbConn,
) -> anyhow::Result<serde_json::Value> {
    if let Some(val) = extract_req_param(param, request, db_connection).await? {
        Ok(serde_json::Value::String(val.into_owned()))
    } else {
        Ok(serde_json::Value::Null)
    }
}

/// This function is used to create a pinned boxed stream of query results.
/// This allows recursive calls.
pub fn stream_query_results_boxed<'a>(
    sql_file: &'a ParsedSqlFile,
    request: &'a mut RequestInfo,
    db_connection: &'a mut DbConn,
) -> Pin<Box<dyn Stream<Item = DbItem> + 'a>> {
    Box::pin(stream_query_results_with_conn(
        sql_file,
        request,
        db_connection,
    ))
}

async fn execute_set_variable_query<'a>(
    db_connection: &'a mut DbConn,
    request: &'a mut RequestInfo,
    variable: &StmtParam,
    statement: &StmtWithParams,
) -> anyhow::Result<()> {
    let query = bind_parameters(statement, request, db_connection).await?;
    let connection = take_connection(&request.app_state.db, db_connection).await?;
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
        vars.insert(name.to_owned(), SingleOrVec::Single(value));
    } else {
        log::debug!("Removing variable {name}");
        vars.remove(name);
    }
    Ok(())
}

fn vars_and_name<'a, 'b>(
    request: &'a mut RequestInfo,
    variable: &'b StmtParam,
) -> anyhow::Result<(&'a mut HashMap<String, SingleOrVec>, &'b str)> {
    match variable {
        StmtParam::PostOrGet(name) => {
            if request.post_variables.contains_key(name) {
                log::warn!("Deprecation warning! Setting the value of ${name}, but there is already a form field named :{name}. This will stop working soon. Please rename the variable, or use :{name} directly if you intended to overwrite the posted form field value.");
                Ok((&mut request.post_variables, name))
            } else {
                Ok((&mut request.get_variables, name))
            }
        }
        StmtParam::Get(name) => Ok((&mut request.get_variables, name)),
        StmtParam::Post(name) => Ok((&mut request.post_variables, name)),
        _ => Err(anyhow!(
            "Only GET and POST variables can be set, not {variable:?}"
        )),
    }
}

async fn take_connection<'a, 'b>(
    db: &'a Database,
    conn: &'b mut DbConn,
) -> anyhow::Result<&'b mut PoolConnection<sqlx::Any>> {
    if let Some(c) = conn {
        return Ok(c);
    }
    match db.connection.acquire().await {
        Ok(c) => {
            log::debug!("Acquired a database connection");
            *conn = Some(c);
            Ok(conn.as_mut().unwrap())
        }
        Err(e) => {
            let err_msg = format!("Unable to acquire a database connection to execute the SQL file. All of the {} {:?} connections are busy.", db.connection.size(), db.connection.any_kind());
            Err(anyhow::Error::new(e).context(err_msg))
        }
    }
}

#[inline]
fn parse_single_sql_result(sql: &str, res: sqlx::Result<Either<AnyQueryResult, AnyRow>>) -> DbItem {
    match res {
        Ok(Either::Right(r)) => {
            if log::log_enabled!(log::Level::Trace) {
                debug_row(&r);
            }
            DbItem::Row(super::sql_to_json::row_to_json(&r))
        }
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

fn debug_row(r: &AnyRow) {
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
    log::trace!("Received db row: {}", row_str);
}

fn clone_anyhow_err(err: &anyhow::Error) -> anyhow::Error {
    let mut e = anyhow!("SQLPage could not parse and prepare this SQL statement");
    for c in err.chain().rev() {
        e = e.context(c.to_string());
    }
    e
}

async fn bind_parameters<'a, 'b>(
    stmt: &'a StmtWithParams,
    request: &'a RequestInfo,
    db_connection: &'b mut DbConn,
) -> anyhow::Result<StatementWithParams<'a>> {
    let sql = stmt.query.as_str();
    log::debug!("Preparing statement: {}", sql);
    let mut arguments = AnyArguments::default();
    for (param_idx, param) in stmt.params.iter().enumerate() {
        log::trace!("\tevaluating parameter {}: {}", param_idx + 1, param);
        let argument = extract_req_param(param, request, db_connection).await?;
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
