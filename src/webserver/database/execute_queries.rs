use anyhow::{Context, anyhow};
use futures_util::StreamExt;
use futures_util::stream::Stream;
use serde_json::Value;
use std::borrow::Cow;
use std::path::Path;
use std::pin::Pin;
use tracing::Instrument;

use super::csv_import::run_csv_import;
use super::error_highlighting::{display_stmt_db_error, display_stmt_error};
use super::sql::{
    DelayedFunctionCall, ParsedSqlFile, ParsedStatement, SimpleSelectValue, StmtWithParams,
};
use crate::dynamic_component::parse_dynamic_rows;
use crate::utils::add_value_to_map;
use crate::webserver::ErrorWithStatus;
use crate::webserver::database::sql_to_json::row_to_string;
use crate::webserver::http_request_info::ExecutionContext;
use crate::webserver::request_variables::SetVariablesMap;
use crate::webserver::single_or_vec::SingleOrVec;

use super::syntax_tree::{StmtParam, extract_req_param};
use super::{Database, DbItem, error_highlighting::display_db_error};
use sqlx::any::{AnyArguments, AnyQueryResult, AnyRow, AnyStatement, AnyTypeInfo};
use sqlx::pool::PoolConnection;
use sqlx::{
    Any, AnyConnection, Arguments, Column, Either, Executor, Row as _, Statement, ValueRef,
};

pub type DbConn = Option<PoolConnection<sqlx::Any>>;

fn source_line_number(line: usize) -> i64 {
    i64::try_from(line).unwrap_or(i64::MAX)
}

use crate::telemetry_metrics::TelemetryMetrics;
use opentelemetry_semantic_conventions::attribute as otel;

fn record_query_params(span: &tracing::Span, params: &[Option<String>]) {
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    for (idx, value) in params.iter().enumerate() {
        let key = opentelemetry::Key::new(format!("{}.{idx}", otel::DB_QUERY_PARAMETER));
        let otel_value = match value {
            Some(v) => opentelemetry::Value::String(v.clone().into()),
            None => opentelemetry::Value::String("NULL".into()),
        };
        span.set_attribute(key, otel_value);
    }
}

struct DbQueryMetricsContext<'a> {
    span: tracing::Span,
    duration: std::time::Duration,
    db_system_name: &'static str,
    operation_name: String,
    metrics: &'a TelemetryMetrics,
}

impl<'a> DbQueryMetricsContext<'a> {
    fn new(
        span: tracing::Span,
        operation_name: String,
        db_system_name: &'static str,
        metrics: &'a TelemetryMetrics,
    ) -> Self {
        Self {
            span,
            duration: std::time::Duration::ZERO,
            db_system_name,
            operation_name,
            metrics,
        }
    }

    fn add_duration(&mut self, duration: std::time::Duration) {
        self.duration += duration;
    }

    fn record_success(&self, returned_rows: i64) {
        self.span
            .record(otel::DB_RESPONSE_RETURNED_ROWS, returned_rows);
        self.span.record(otel::OTEL_STATUS_CODE, "OK");
        let attributes = [
            opentelemetry::KeyValue::new(otel::DB_SYSTEM_NAME, self.db_system_name),
            opentelemetry::KeyValue::new(otel::DB_OPERATION_NAME, self.operation_name.clone()),
            opentelemetry::KeyValue::new(otel::OTEL_STATUS_CODE, "OK"),
        ];
        self.metrics
            .db_query_duration
            .record(self.duration.as_secs_f64(), &attributes);
    }

    fn record_error(&self, returned_rows: i64, error: &anyhow::Error) {
        self.span
            .record(otel::DB_RESPONSE_RETURNED_ROWS, returned_rows);
        self.span.record(otel::OTEL_STATUS_CODE, "ERROR");
        self.span
            .record(otel::EXCEPTION_MESSAGE, tracing::field::display(error));
        self.span
            .record("exception.details", tracing::field::debug(error));
        let attributes = [
            opentelemetry::KeyValue::new(otel::DB_SYSTEM_NAME, self.db_system_name),
            opentelemetry::KeyValue::new(otel::DB_OPERATION_NAME, self.operation_name.clone()),
            opentelemetry::KeyValue::new(otel::OTEL_STATUS_CODE, "ERROR"),
            opentelemetry::KeyValue::new(otel::ERROR_TYPE, error.to_string()),
        ];
        self.metrics
            .db_query_duration
            .record(self.duration.as_secs_f64(), &attributes);
    }
}

fn create_db_query_span(
    sql: &str,
    source_file: &Path,
    line: usize,
    db_system_name: &'static str,
) -> (tracing::Span, String) {
    let operation_name = sql.split_whitespace().next().unwrap_or("").to_uppercase();
    let span = tracing::info_span!(
        "db.query",
        { otel::DB_QUERY_TEXT } = sql,
        { otel::DB_SYSTEM_NAME } = db_system_name,
        { otel::DB_OPERATION_NAME } = operation_name,
        { otel::CODE_FILE_PATH } = %source_file.display(),
        { otel::CODE_LINE_NUMBER } = source_line_number(line),
        { otel::OTEL_STATUS_CODE } = tracing::field::Empty,
        { otel::EXCEPTION_MESSAGE } = tracing::field::Empty,
        "exception.details" = tracing::field::Empty,
        { otel::DB_RESPONSE_RETURNED_ROWS } = tracing::field::Empty,
    );
    (span, operation_name)
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
            .map_err(|e| display_db_error(Path::new("autogenerated sqlpage query"), query, e))
    }
}

pub fn stream_query_results_with_conn<'a>(
    sql_file: &'a ParsedSqlFile,
    request: &'a ExecutionContext,
    db_connection: &'a mut DbConn,
) -> impl Stream<Item = DbItem> + 'a {
    let source_file = &sql_file.source_path;
    async_stream::try_stream! {
        for res in &sql_file.statements {
            match res {
                ParsedStatement::CsvImport(csv_import) => {
                    let connection = take_connection(&request.app_state.db, db_connection, request).await?;
                    log::debug!("Executing CSV import: {csv_import:?}");
                    run_csv_import(connection, csv_import, request).await.with_context(|| format!("Failed to import the CSV file {:?} into the table {:?}", csv_import.uploaded_file, csv_import.table_name))?;
                },
                ParsedStatement::StmtWithParams(stmt) => {
                    let query = bind_parameters(stmt, request, db_connection)
                        .await
                        .map_err(|e| with_stmt_position(source_file, stmt.query_position, e))?;
                    request.server_timing.record("bind_params");
                    let connection = take_connection(&request.app_state.db, db_connection, request).await?;
                    log::trace!("Executing query {:?}", query.sql);
                    let db_system_name = request.app_state.db.info.database_type.otel_name();
                    let (query_span, operation_name) = create_db_query_span(
                        query.sql,
                        source_file,
                        stmt.query_position.start.line,
                        db_system_name,
                    );
                    let mut query_metrics = DbQueryMetricsContext::new(
                        query_span.clone(),
                        operation_name,
                        db_system_name,
                        &request.app_state.telemetry_metrics,
                    );
                    record_query_params(&query_metrics.span, &query.param_values);
                    let mut stream = connection.fetch_many(query);
                    let mut error = None;
                    let mut returned_rows: i64 = 0;
                    loop {
                        let start_next = std::time::Instant::now();
                        let next_elem = stream.next().instrument(query_span.clone()).await;
                        query_metrics.add_duration(start_next.elapsed());
                        let Some(elem) = next_elem else { break; };

                        let mut query_result = parse_single_sql_result(source_file, stmt, elem);
                        if let DbItem::Error(e) = query_result {
                            error = Some(e);
                            break;
                        }
                        if matches!(query_result, DbItem::Row(_)) {
                            returned_rows += 1;
                        }
                        apply_json_columns(&mut query_result, &stmt.json_columns);
                        if let Err(err) = apply_delayed_functions(request, &stmt.delayed_functions, &mut query_result)
                            .instrument(query_span.clone())
                            .await
                        {
                            error = Some(err);
                            break;
                        }
                        for db_item in parse_dynamic_rows(query_result) {
                            yield db_item;
                        }
                    }
                    drop(stream);
                    if let Some(error) = error {
                        query_metrics.record_error(returned_rows, &error);
                        try_rollback_transaction(connection).await;
                        yield DbItem::Error(error);
                    } else {
                        query_metrics.record_success(returned_rows);
                    }
                },
                ParsedStatement::SetVariable { variable, value} => {
                    execute_set_variable_query(db_connection, request, variable, value, source_file).await
                    .with_context(||
                        format!("Failed to set the {variable} variable to {value:?}")
                    )?;
                },
                ParsedStatement::StaticSimpleSet { variable, value} => {
                    execute_set_simple_static(db_connection, request, variable, value, source_file).await
                    .with_context(||
                        format!("Failed to set the {variable} variable to {value:?}")
                    )?;
                },
                ParsedStatement::StaticSimpleSelect { values, query_position } => {
                    let row = exec_static_simple_select(values, request, db_connection)
                        .await
                        .map_err(|e| with_stmt_position(source_file, *query_position, e))?;
                    for i in parse_dynamic_rows(DbItem::Row(row)) {
                        yield i;
                    }
                }
                ParsedStatement::Error(e) => yield DbItem::Error(clone_anyhow_err(source_file, e)),
            }
        }
    }
    .map(|res| res.unwrap_or_else(DbItem::Error))
}

fn with_stmt_position(
    source_file: &Path,
    query_position: super::sql::SourceSpan,
    error: anyhow::Error,
) -> anyhow::Error {
    if error.downcast_ref::<ErrorWithStatus>().is_some() {
        error
    } else {
        display_stmt_error(source_file, query_position, error)
    }
}

/// Transforms a stream of database items to stop processing after encountering the first error.
/// The error item itself is still emitted before stopping.
pub fn stop_at_first_error(
    results_stream: impl Stream<Item = DbItem>,
) -> impl Stream<Item = DbItem> {
    // We need a oneshot channel rather than a simple boolean flag because
    // take_while would poll the stream one extra time after the error,
    // while take_until stops immediately when the future completes
    let (error_tx, error_rx) = tokio::sync::oneshot::channel();
    let mut error_tx = Some(error_tx);

    results_stream
        .inspect(move |item| {
            if let DbItem::Error(err) = item {
                log::error!("{err:?}");
                if let Some(tx) = error_tx.take() {
                    let _ = tx.send(());
                }
            }
        })
        .take_until(error_rx)
}

/// Executes the sqlpage pseudo-functions contained in a static simple select
async fn exec_static_simple_select(
    columns: &[(String, SimpleSelectValue)],
    req: &ExecutionContext,
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

async fn try_rollback_transaction(db_connection: &mut AnyConnection) {
    log::debug!("Attempting to rollback transaction");
    match db_connection.execute("ROLLBACK").await {
        Ok(_) => log::debug!("Rolled back transaction"),
        Err(e) => {
            log::debug!("There was probably no transaction in progress when this happened: {e:?}");
        }
    }
}

/// Extracts the value of a parameter from the request.
/// Returns `Ok(None)` when NULL should be used as the parameter value.
async fn extract_req_param_as_json(
    param: &StmtParam,
    request: &ExecutionContext,
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
    request: &'a ExecutionContext,
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
    request: &'a ExecutionContext,
    variable: &StmtParam,
    statement: &StmtWithParams,
    source_file: &Path,
) -> anyhow::Result<()> {
    let query = bind_parameters(statement, request, db_connection).await?;
    let connection = take_connection(&request.app_state.db, db_connection, request).await?;
    log::debug!(
        "Executing query to set the {variable:?} variable: {:?}",
        query.sql
    );

    let db_system_name = request.app_state.db.info.database_type.otel_name();
    let (query_span, operation_name) = create_db_query_span(
        query.sql,
        source_file,
        statement.query_position.start.line,
        db_system_name,
    );
    let mut query_metrics = DbQueryMetricsContext::new(
        query_span.clone(),
        operation_name,
        db_system_name,
        &request.app_state.telemetry_metrics,
    );
    record_query_params(&query_metrics.span, &query.param_values);
    let start_time = std::time::Instant::now();
    let value = match connection
        .fetch_optional(query)
        .instrument(query_span.clone())
        .await
    {
        Ok(Some(row)) => {
            query_metrics.add_duration(start_time.elapsed());
            query_metrics.record_success(1_i64);
            row_to_string(&row)
        }
        Ok(None) => {
            query_metrics.add_duration(start_time.elapsed());
            query_metrics.record_success(0_i64);
            None
        }
        Err(e) => {
            query_metrics.add_duration(start_time.elapsed());
            try_rollback_transaction(connection).await;
            let err = display_stmt_db_error(source_file, statement, e);
            query_metrics.record_error(0_i64, &err);
            return Err(err);
        }
    };

    let (mut vars, name) = vars_and_name(request, variable)?;

    log::debug!("Setting variable {name} to {value:?}");
    vars.insert(name.to_owned(), value.map(SingleOrVec::Single));

    Ok(())
}

async fn execute_set_simple_static<'a>(
    db_connection: &'a mut DbConn,
    request: &'a ExecutionContext,
    variable: &StmtParam,
    value: &SimpleSelectValue,
    _source_file: &Path,
) -> anyhow::Result<()> {
    let value_str = match value {
        SimpleSelectValue::Static(json_value) => match json_value {
            serde_json::Value::Null => None,
            serde_json::Value::String(s) => Some(s.clone()),
            other => Some(other.to_string()),
        },
        SimpleSelectValue::Dynamic(stmt_param) => {
            extract_req_param(stmt_param, request, db_connection)
                .await?
                .map(std::borrow::Cow::into_owned)
        }
    };

    let (mut vars, name) = vars_and_name(request, variable)?;

    log::debug!("Setting variable {name} to static value {value_str:?}");
    vars.insert(name.to_owned(), value_str.map(SingleOrVec::Single));
    Ok(())
}

fn vars_and_name<'a, 'b>(
    request: &'a ExecutionContext,
    variable: &'b StmtParam,
) -> anyhow::Result<(std::cell::RefMut<'a, SetVariablesMap>, &'b str)> {
    match variable {
        StmtParam::PostOrGet(name) | StmtParam::Get(name) => {
            if request.post_variables.contains_key(name) {
                log::warn!(
                    "Deprecation warning! Setting the value of ${name}, but there is already a form field named :{name}. This will stop working soon. Please rename the variable, or use :{name} directly if you intended to overwrite the posted form field value."
                );
            }
            Ok((request.set_variables.borrow_mut(), name))
        }
        StmtParam::Post(name) => Ok((request.set_variables.borrow_mut(), name)),
        _ => Err(anyhow!(
            "Only GET and POST variables can be set, not {variable:?}"
        )),
    }
}

async fn take_connection<'a>(
    db: &'a Database,
    conn: &'a mut DbConn,
    request: &ExecutionContext,
) -> anyhow::Result<&'a mut PoolConnection<sqlx::Any>> {
    if let Some(c) = conn {
        return Ok(c);
    }
    let pool_size = db.connection.size();
    let acquire_span = tracing::info_span!("db.pool.acquire", db.pool.size = pool_size,);
    match db.connection.acquire().instrument(acquire_span).await {
        Ok(c) => {
            log::debug!("Acquired a database connection");
            request.server_timing.record("db_conn");
            *conn = Some(c);
            let connection = conn.as_mut().unwrap();
            set_trace_context(connection, db).await;
            Ok(connection)
        }
        Err(e) => {
            let db_name = db.connection.any_kind();
            let active_count = db.connection.size();
            let err_msg = format!(
                "Unable to connect to {db_name:?}. The connection pool currently has {active_count} active connections."
            );
            Err(anyhow::Error::new(e).context(err_msg))
        }
    }
}

/// Sets the current `OTel` trace context on the database connection so it is visible
/// in `pg_stat_activity.application_name` (`PostgreSQL`) or as a session variable (`MySQL`).
/// This allows correlating `SQLPage` traces with database-side monitoring.
async fn set_trace_context(connection: &mut AnyConnection, db: &Database) {
    use opentelemetry::trace::TraceContextExt;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let span = tracing::Span::current();
    let context = span.context();
    let otel_span = context.span();
    let span_context = otel_span.span_context();
    if !span_context.is_valid() {
        return;
    }
    let traceparent = format!(
        "00-{}-{}-{:02x}",
        span_context.trace_id(),
        span_context.span_id(),
        span_context.trace_flags()
    );
    let sql = match db.info.kind {
        sqlx::any::AnyKind::Postgres => {
            // postgresqlreceiver expects application_name to be a raw W3C traceparent value.
            format!("SET application_name = '{traceparent}'")
        }
        sqlx::any::AnyKind::MySql => {
            format!("SET @traceparent = '{traceparent}'")
        }
        _ => return,
    };
    if let Err(e) = connection.execute(sql.as_str()).await {
        log::debug!("Failed to set trace context on connection: {e}");
    }
}

#[inline]
fn parse_single_sql_result(
    source_file: &Path,
    stmt: &StmtWithParams,
    res: sqlx::Result<Either<AnyQueryResult, AnyRow>>,
) -> DbItem {
    match res {
        Ok(Either::Right(r)) => {
            if log::log_enabled!(log::Level::Trace) {
                debug_row(&r);
            }
            DbItem::Row(super::sql_to_json::row_to_json(&r))
        }
        Ok(Either::Left(res)) => {
            log::debug!("Finished query with result: {res:?}");
            DbItem::FinishedQuery
        }
        Err(err) => {
            let nice_err = display_stmt_db_error(source_file, stmt, err);
            DbItem::Error(nice_err)
        }
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
    log::trace!("Received db row: {row_str}");
}

fn clone_anyhow_err(source_file: &Path, err: &anyhow::Error) -> anyhow::Error {
    if let Some(func_err) = err.downcast_ref::<super::sql::SqlPageFunctionError>() {
        let line = func_err.line;
        let loc = if line > 0 {
            format!(":{line}")
        } else {
            String::new()
        };
        return anyhow::anyhow!("{}{loc} {}", source_file.display(), func_err);
    }

    let mut e = anyhow!(
        "{} contains a syntax error preventing SQLPage from parsing and preparing its SQL statements.",
        source_file.display()
    );
    for c in err.chain().rev() {
        e = e.context(c.to_string());
    }
    e
}

async fn bind_parameters<'a>(
    stmt: &'a StmtWithParams,
    request: &'a ExecutionContext,
    db_connection: &mut DbConn,
) -> anyhow::Result<StatementWithParams<'a>> {
    let sql = stmt.query.as_str();
    log::debug!("Preparing statement: {sql}");
    let mut arguments = AnyArguments::default();
    let mut param_values = Vec::with_capacity(stmt.params.len());
    for (param_idx, param) in stmt.params.iter().enumerate() {
        log::trace!("\tevaluating parameter {}: {}", param_idx + 1, param);
        let argument = extract_req_param(param, request, db_connection).await?;
        log::debug!(
            "\tparameter {}: {}",
            param_idx + 1,
            argument.as_ref().unwrap_or(&Cow::Borrowed("NULL"))
        );
        param_values.push(argument.as_deref().map(str::to_owned));
        match argument {
            None => arguments.add(None::<String>),
            Some(Cow::Owned(s)) => arguments.add(s),
            Some(Cow::Borrowed(v)) => arguments.add(v),
        }
    }
    let has_arguments = !stmt.params.is_empty();
    Ok(StatementWithParams {
        sql,
        arguments,
        has_arguments,
        param_values,
    })
}

async fn apply_delayed_functions(
    request: &ExecutionContext,
    delayed_functions: &[DelayedFunctionCall],
    item: &mut DbItem,
) -> anyhow::Result<()> {
    // We need to open new connections for each delayed function call, because we are still fetching the results of the current query in the main connection.
    let mut db_conn = None;
    if let DbItem::Row(serde_json::Value::Object(results)) = item {
        for f in delayed_functions {
            log::trace!("Applying delayed function {} to {:?}", f.function, results);
            apply_single_delayed_function(request, &mut db_conn, f, results).await?;
            log::trace!(
                "Delayed function applied {}. Result: {:?}",
                f.function,
                results
            );
        }
    }
    Ok(())
}

async fn apply_single_delayed_function(
    request: &ExecutionContext,
    db_connection: &mut DbConn,
    f: &DelayedFunctionCall,
    row: &mut serde_json::Map<String, serde_json::Value>,
) -> anyhow::Result<()> {
    let mut params = Vec::new();
    for arg in &f.argument_col_names {
        let Some(arg_value) = row.remove(arg) else {
            anyhow::bail!(
                "The column {arg} is missing in the result set, but it is required by the {} function.",
                f.function
            );
        };
        params.push(json_to_fn_param(arg_value));
    }
    let result_str = f.function.evaluate(request, db_connection, params).await?;
    let result_json = result_str
        .map(Cow::into_owned)
        .map_or(serde_json::Value::Null, serde_json::Value::String);
    row.insert(f.target_col_name.clone(), result_json);
    Ok(())
}

fn json_to_fn_param(json: serde_json::Value) -> Option<Cow<'static, str>> {
    match json {
        serde_json::Value::String(s) => Some(Cow::Owned(s)),
        serde_json::Value::Null => None,
        _ => Some(Cow::Owned(json.to_string())),
    }
}

fn apply_json_columns(item: &mut DbItem, json_columns: &[String]) {
    if let DbItem::Row(Value::Object(row)) = item {
        for column in json_columns {
            if let Some(value) = row.get_mut(column) {
                if let Value::String(json_str) = value {
                    if let Ok(parsed_json) = serde_json::from_str(json_str) {
                        log::trace!("Parsed JSON column {column}: {parsed_json}");
                        *value = parsed_json;
                    } else {
                        log::warn!("The column {column} contains invalid JSON: {json_str}");
                    }
                } else if let Value::Array(array) = value {
                    for item in array {
                        if let Value::String(json_str) = item {
                            if let Ok(parsed_json) = serde_json::from_str(json_str) {
                                log::trace!("Parsed JSON array item: {parsed_json}");
                                *item = parsed_json;
                            }
                        }
                    }
                }
            } else {
                log::warn!(
                    "The column {column} is missing from the result set, so it cannot be converted to JSON."
                );
            }
        }
    }
}

pub struct StatementWithParams<'a> {
    sql: &'a str,
    arguments: AnyArguments<'a>,
    has_arguments: bool,
    param_values: Vec<Option<String>>,
}

impl<'q> sqlx::Execute<'q, Any> for StatementWithParams<'q> {
    fn sql(&self) -> &'q str {
        self.sql
    }

    fn statement(&self) -> Option<&<Any as sqlx::database::HasStatement<'q>>::Statement> {
        None
    }

    fn take_arguments(&mut self) -> Option<<Any as sqlx::database::HasArguments<'q>>::Arguments> {
        if self.has_arguments {
            Some(std::mem::take(&mut self.arguments))
        } else {
            None
        }
    }

    fn persistent(&self) -> bool {
        // Let sqlx create a prepared statement the first time it is executed, and then reuse it.
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use serde_json::{Value, json};
    use tracing::field::{Field, Visit};
    use tracing_subscriber::Layer;
    use tracing_subscriber::layer::Context;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::registry::LookupSpan;

    fn create_row_item(value: Value) -> DbItem {
        DbItem::Row(value)
    }

    fn assert_json_value(item: &DbItem, key: &str, expected: Value) {
        let DbItem::Row(Value::Object(row)) = item else {
            panic!("Expected DbItem::Row");
        };
        assert_eq!(row[key], expected);
        drop(expected);
    }

    #[test]
    fn test_basic_json_string_conversion() {
        let mut item = create_row_item(json!({
            "json_col": "{\"key\": \"value\"}",
            "normal_col": "text"
        }));
        apply_json_columns(&mut item, &["json_col".to_string()]);
        assert_json_value(&item, "json_col", json!({"key": "value"}));
        assert_json_value(&item, "normal_col", json!("text"));
    }

    #[test]
    fn test_json_array_conversion() {
        let mut item = create_row_item(json!({
            "array_col": ["{\"a\": 1}", "{\"b\": 2}"],
            "normal_array": ["text"]
        }));
        apply_json_columns(&mut item, &["array_col".to_string()]);
        assert_json_value(&item, "array_col", json!([{"a": 1}, {"b": 2}]));
        assert_json_value(&item, "normal_array", json!(["text"]));
    }

    #[test]
    fn test_invalid_json_handling() {
        let mut item = create_row_item(json!({
            "invalid_json": "{not valid json}",
            "normal_col": "text"
        }));
        apply_json_columns(&mut item, &["invalid_json".to_string()]);
        assert_json_value(&item, "invalid_json", json!("{not valid json}"));
        assert_json_value(&item, "normal_col", json!("text"));
    }

    #[test]
    fn test_missing_column_handling() {
        let mut item = create_row_item(json!({
            "existing_col": "text"
        }));
        apply_json_columns(&mut item, &["missing_col".to_string()]);
        assert_json_value(&item, "existing_col", json!("text"));
    }

    #[test]
    fn test_non_row_dbitem_handling() {
        let mut item = DbItem::FinishedQuery;
        apply_json_columns(&mut item, &["json_col".to_string()]);
        assert!(matches!(item, DbItem::FinishedQuery));
    }

    #[test]
    fn test_duplicate_json_column_names() {
        let mut item = create_row_item(json!({
            "json_col": "{\"key\": \"value\"}",
            "normal_col": "text"
        }));
        apply_json_columns(&mut item, &["json_col".to_string(), "json_col".to_string()]);
        assert_json_value(&item, "json_col", json!({"key": "value"}));
        assert_json_value(&item, "normal_col", json!("text"));
    }

    #[derive(Default)]
    struct RecordedFields(HashMap<&'static str, String>);

    #[derive(Clone, Default)]
    struct TestSpanLayer {
        closed_spans: Arc<Mutex<Vec<HashMap<&'static str, String>>>>,
    }

    struct TestFieldVisitor<'a>(&'a mut HashMap<&'static str, String>);

    impl Visit for TestFieldVisitor<'_> {
        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            self.0.insert(field.name(), format!("{value:?}"));
        }

        fn record_str(&mut self, field: &Field, value: &str) {
            self.0.insert(field.name(), value.to_owned());
        }

        fn record_i64(&mut self, field: &Field, value: i64) {
            self.0.insert(field.name(), value.to_string());
        }
    }

    impl<S> Layer<S> for TestSpanLayer
    where
        S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_new_span(
            &self,
            attrs: &tracing::span::Attributes<'_>,
            id: &tracing::span::Id,
            ctx: Context<'_, S>,
        ) {
            let mut fields = RecordedFields::default();
            attrs.record(&mut TestFieldVisitor(&mut fields.0));
            if let Some(span) = ctx.span(id) {
                span.extensions_mut().insert(fields);
            }
        }

        fn on_record(
            &self,
            id: &tracing::span::Id,
            values: &tracing::span::Record<'_>,
            ctx: Context<'_, S>,
        ) {
            if let Some(span) = ctx.span(id) {
                let mut extensions = span.extensions_mut();
                let fields = extensions
                    .get_mut::<RecordedFields>()
                    .expect("recorded fields");
                values.record(&mut TestFieldVisitor(&mut fields.0));
            }
        }

        fn on_close(&self, id: tracing::span::Id, ctx: Context<'_, S>) {
            if let Some(span) = ctx.span(&id) {
                let extensions = span.extensions();
                let fields = extensions.get::<RecordedFields>().expect("recorded fields");
                self.closed_spans.lock().unwrap().push(fields.0.clone());
            }
        }
    }

    fn with_recorded_span_fields(
        f: impl FnOnce() + Send + 'static,
    ) -> HashMap<&'static str, String> {
        let layer = TestSpanLayer::default();
        let closed_spans = layer.closed_spans.clone();
        let subscriber = tracing_subscriber::registry().with(layer);
        tracing::subscriber::with_default(subscriber, f);
        let fields = closed_spans
            .lock()
            .unwrap()
            .pop()
            .expect("closed span fields");
        fields
    }

    #[test]
    fn db_query_success_records_ok_status_and_row_count() {
        let fields = with_recorded_span_fields(|| {
            let span = tracing::info_span!(
                "db.query",
                otel.status_code = tracing::field::Empty,
                exception.message = tracing::field::Empty,
                exception.details = tracing::field::Empty,
                db.response.returned_rows = tracing::field::Empty,
            );
            let metrics = crate::telemetry_metrics::TelemetryMetrics::default();
            let query_metrics =
                DbQueryMetricsContext::new(span.clone(), "SELECT".to_string(), "sqlite", &metrics);
            query_metrics.record_success(3);
            drop(span);
        });

        assert_eq!(fields[otel::OTEL_STATUS_CODE], "OK");
        assert_eq!(fields[otel::DB_RESPONSE_RETURNED_ROWS], "3");
        assert!(!fields.contains_key(otel::EXCEPTION_MESSAGE));
        assert!(!fields.contains_key("exception.details"));
    }

    #[test]
    fn db_query_error_records_error_status_and_exception_fields() {
        let fields = with_recorded_span_fields(|| {
            let span = tracing::info_span!(
                "db.query",
                otel.status_code = tracing::field::Empty,
                exception.message = tracing::field::Empty,
                exception.details = tracing::field::Empty,
                db.response.returned_rows = tracing::field::Empty,
            );
            let error = anyhow!("query failed").context("while executing SELECT 1");
            let metrics = crate::telemetry_metrics::TelemetryMetrics::default();
            let query_metrics =
                DbQueryMetricsContext::new(span.clone(), "SELECT".to_string(), "sqlite", &metrics);
            query_metrics.record_error(2, &error);
            drop(span);
        });

        assert_eq!(fields[otel::OTEL_STATUS_CODE], "ERROR");
        assert_eq!(fields[otel::DB_RESPONSE_RETURNED_ROWS], "2");
        assert!(fields[otel::EXCEPTION_MESSAGE].contains("while executing SELECT 1"));
        assert!(fields["exception.details"].contains("query failed"));
    }
}
