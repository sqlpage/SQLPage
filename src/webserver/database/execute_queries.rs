use anyhow::anyhow;
use futures_util::stream::Stream;
use futures_util::StreamExt;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;

use super::sql::{ParsedSQLStatement, ParsedSqlFile};
use crate::webserver::database::sql_pseudofunctions::extract_req_param;
use crate::webserver::http::{RequestInfo, SingleOrVec};

use sqlx::any::{AnyArguments, AnyQueryResult, AnyRow, AnyStatement, AnyTypeInfo};
use sqlx::pool::PoolConnection;
use sqlx::query::Query;
use sqlx::{AnyConnection, Arguments, Either, Executor, Row, Statement};

use super::sql_pseudofunctions::StmtParam;
use super::sql_to_json::sql_to_json;
use super::{highlight_sql_error, Database, DbItem, PreparedStatement};

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

pub fn stream_query_results<'a>(
    db: &'a Database,
    sql_file: &'a ParsedSqlFile,
    request: &'a mut RequestInfo,
) -> impl Stream<Item = DbItem> + 'a {
    async_stream::try_stream! {
        let mut connection_opt = None;
        for res in &sql_file.statements {
            match res {
                ParsedSQLStatement::Statement(stmt) => {
                    let query = bind_parameters(stmt, request).await?;
                    let connection = take_connection(db, &mut connection_opt).await?;
                    let mut stream = query.fetch_many(connection);
                    while let Some(elem) = stream.next().await {
                        let is_err = elem.is_err();
                        yield parse_single_sql_result(stmt, elem);
                        if is_err {
                            break;
                        }
                    }
                },
                ParsedSQLStatement::SetVariable { variable, value} => {
                    let query = bind_parameters(value, request).await?;
                    let connection = take_connection(db, &mut connection_opt).await?;
                    let row = query.fetch_optional(connection).await?;
                    let (vars, name) = vars_and_name(request, variable)?;
                    if let Some(row) = row {
                        vars.insert(name.clone(), row_to_varvalue(&row));
                    } else {
                        vars.remove(&name);
                    }
                },
                ParsedSQLStatement::StaticSimpleSelect(value) => {
                    yield DbItem::Row(value.clone().into())
                }
                ParsedSQLStatement::Error(e) => yield DbItem::Error(clone_anyhow_err(e)),
            }
        }
    }
    .map(|res| res.unwrap_or_else(DbItem::Error))
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

fn row_to_varvalue(row: &AnyRow) -> SingleOrVec {
    let Some(col) = row.columns().first() else {
        return SingleOrVec::Single(String::new());
    };
    match sql_to_json(row, col) {
        Value::String(s) => SingleOrVec::Single(s),
        Value::Array(vals) => SingleOrVec::Vec(
            vals.into_iter()
                .map(|v| match v {
                    Value::String(s) => s,
                    other => other.to_string(),
                })
                .collect(),
        ),
        other => SingleOrVec::Single(other.to_string()),
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
fn parse_single_sql_result(
    stmt: &PreparedStatement,
    res: sqlx::Result<Either<AnyQueryResult, AnyRow>>,
) -> DbItem {
    match res {
        Ok(Either::Right(r)) => DbItem::Row(super::sql_to_json::row_to_json(&r)),
        Ok(Either::Left(res)) => {
            log::debug!("Finished query with result: {:?}", res);
            DbItem::FinishedQuery
        }
        Err(err) => DbItem::Error(highlight_sql_error(
            "Failed to execute SQL statement",
            stmt.statement.sql(),
            err,
        )),
    }
}

fn clone_anyhow_err(err: &anyhow::Error) -> anyhow::Error {
    let mut e = anyhow!("An error occurred while trying to prepare this SQL statement");
    for c in err.chain().rev() {
        e = e.context(c.to_string());
    }
    e
}

async fn bind_parameters<'a>(
    stmt: &'a PreparedStatement,
    request: &'a RequestInfo,
) -> anyhow::Result<Query<'a, sqlx::Any, AnyArguments<'a>>> {
    let mut arguments = AnyArguments::default();
    for param in &stmt.parameters {
        let argument = extract_req_param(param, request).await?;
        log::debug!("Binding value {:?} in statement {}", &argument, stmt);
        match argument {
            None => arguments.add(None::<String>),
            Some(Cow::Owned(s)) => arguments.add(s),
            Some(Cow::Borrowed(v)) => arguments.add(v),
        }
    }
    Ok(stmt.statement.query_with(arguments))
}
