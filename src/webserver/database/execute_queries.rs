use anyhow::{Context, anyhow};
use futures_util::StreamExt;
use futures_util::stream::Stream;
use serde_json::Value;
use std::borrow::Cow;
use std::path::Path;
use std::pin::Pin;
use tracing::Instrument;

use super::csv_import::run_csv_import;
use super::error_highlighting::{display_stmt_db_error, display_stmt_error, is_positioned_error};
use super::sql::{
    DatabaseQuery, FileStatement, OutputColumn, Query, QueryBody, SourceSpan, SqlFile,
    StaticSimpleSelect,
};
use super::sqlpage_expr::{NoInputs, RowExpr, RowInputs};
use crate::dynamic_component::parse_dynamic_rows;
use crate::utils::add_value_to_map;
use crate::webserver::ErrorWithStatus;
use crate::webserver::http_request_info::ExecutionContext;
use crate::webserver::single_or_vec::SingleOrVec;

use super::{Database, DbItem, ScalarSubqueryBehavior, error_highlighting::display_db_error};
use sqlx::Either;
use sqlx::any::{
    Any, AnyArguments, AnyConnection, AnyQueryResult, AnyRow, AnyStatement, AnyTypeInfo,
};
use sqlx::arguments::Arguments;
use sqlx::column::Column;
use sqlx::executor::{Execute, Executor};
use sqlx::pool::PoolConnection;
use sqlx::row::Row as _;
use sqlx::statement::Statement;
use sqlx::value::ValueRef;

pub type DbConn = Option<PoolConnection<Any>>;

/// One database result together with private values reserved for computed
/// columns and therefore omitted from the user-visible row.
struct QueryResult {
    item: DbItem,
    inputs: RowInputs,
    output_column_count: usize,
}

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
            .record("sqlpage.exception.details", tracing::field::debug(error));
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
        "otel.kind" = "client",
        "otel.name" = %operation_name,
        { otel::DB_QUERY_TEXT } = sql,
        { otel::DB_SYSTEM_NAME } = db_system_name,
        { otel::DB_OPERATION_NAME } = operation_name,
        { otel::CODE_FILE_PATH } = %source_file.display(),
        { otel::CODE_LINE_NUMBER } = source_line_number(line),
        { otel::OTEL_STATUS_CODE } = tracing::field::Empty,
        { otel::EXCEPTION_MESSAGE } = tracing::field::Empty,
        "sqlpage.exception.details" = tracing::field::Empty,
        { otel::DB_RESPONSE_RETURNED_ROWS } = tracing::field::Empty,
    );
    (span, operation_name)
}

fn create_query_metrics<'a>(
    request: &'a ExecutionContext,
    source_file: &Path,
    source_span: SourceSpan,
    query: &BoundQuery<'_>,
) -> (tracing::Span, DbQueryMetricsContext<'a>) {
    let db_system_name = request.app_state.db.info.database_type.otel_name();
    let (query_span, operation_name) = create_db_query_span(
        query.sql,
        source_file,
        source_span.start.line,
        db_system_name,
    );
    let query_metrics = DbQueryMetricsContext::new(
        query_span.clone(),
        operation_name,
        db_system_name,
        &request.app_state.telemetry_metrics,
    );
    record_query_params(&query_metrics.span, &query.param_values);
    (query_span, query_metrics)
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

#[allow(clippy::too_many_lines)] // Keeps the single-connection statement dispatcher together.
pub fn stream_query_results_with_conn<'a>(
    sql_file: &'a SqlFile,
    request: &'a ExecutionContext,
    db_connection: &'a mut DbConn,
) -> impl Stream<Item = DbItem> + 'a {
    let source_file = &sql_file.source_path;
    async_stream::try_stream! {
        for res in &sql_file.statements {
            match res {
                FileStatement::CsvImport(csv_import) => {
                    let connection = take_connection(&request.app_state.db, db_connection, request).await?;
                    log::debug!("Executing CSV import: {csv_import:?}");
                    run_csv_import(connection, csv_import, request).await.with_context(|| format!("Failed to import the CSV file {:?} into the table {:?}", csv_import.uploaded_file, csv_import.table_name))?;
                },
                FileStatement::Query(statement) => match &statement.body {
                  QueryBody::StaticSimpleSelect(query) => {
                    let row = execute_static_simple_select(query, request, db_connection)
                        .await
                        .map_err(|error| with_stmt_position(source_file, statement.source_span, error))?;
                    for item in parse_dynamic_rows(DbItem::Row(row)) {
                        yield item;
                    }
                  }
                  QueryBody::Database(stmt) => {
                    let query = bind_query(stmt, request, db_connection)
                        .await
                        .map_err(|error| with_stmt_position(source_file, statement.source_span, error))?;
                    request.server_timing.record("bind_params");
                    log::trace!("Executing query {:?}", query.sql);
                    let (query_span, mut query_metrics) = create_query_metrics(request, source_file, statement.source_span, &query);
                    let mut error = None;
                    let mut returned_rows: i64 = 0;
                    let buffer_rows = stmt.must_buffer_rows();
                    let mut deferred_query_results = Vec::new();
                    {
                        let connection = take_connection(&request.app_state.db, db_connection, request).await?;
                        let mut stream = connection.fetch_many(query);
                        loop {
                        let start_next = std::time::Instant::now();
                        let next_elem = stream.next().instrument(query_span.clone()).await;
                        query_metrics.add_duration(start_next.elapsed());
                        let Some(elem) = next_elem else { break; };

                        let mut query_result = parse_single_sql_result(source_file, stmt, statement.source_span, elem);
                        if let DbItem::Error(e) = query_result.item {
                            error = Some(e);
                            break;
                        }
                        if matches!(query_result.item, DbItem::Row(_)) {
                            returned_rows += 1;
                        }
                        apply_json_columns(&mut query_result.item, &stmt.json_columns);
                        if buffer_rows {
                            deferred_query_results.push(query_result);
                        } else {
                            let mut computed_connection = None;
                            if let Err(err) = evaluate_computed_columns(request, &stmt.computed_columns, &mut query_result, &mut computed_connection)
                                .instrument(query_span.clone())
                                .await
                            {
                                error = Some(err);
                                break;
                            }
                            for db_item in parse_dynamic_rows(query_result.item) {
                                yield db_item;
                            }
                        }
                        }
                        drop(stream);
                    }
                    if error.is_none() && buffer_rows {
                        for mut query_result in deferred_query_results {
                            if let Err(err) = evaluate_computed_columns(
                                request,
                                &stmt.computed_columns,
                                &mut query_result,
                                db_connection,
                            )
                            .instrument(query_span.clone())
                            .await
                            {
                                error = Some(err);
                                break;
                            }
                            for db_item in parse_dynamic_rows(query_result.item) {
                                yield db_item;
                            }
                        }
                    }
                    if let Some(error) = error {
                        let connection = take_connection(&request.app_state.db, db_connection, request).await?;
                        let error = record_error_and_rollback(connection, &query_metrics, returned_rows, error).await;
                        yield DbItem::Error(error);
                    } else {
                        query_metrics.record_success(returned_rows);
                    }
                  }
                },
                FileStatement::SetVariable { target, value} => {
                    execute_set_variable_query(db_connection, request, target, value, source_file).await
                    .with_context(|| format!("Failed to set variable {}", target.0))
                    .map_err(|error| with_stmt_position(source_file, value.source_span, error))?;
                },
                FileStatement::Error(e) => yield DbItem::Error(clone_anyhow_err(source_file, e)),
            }
        }
    }
    .map(|res| res.unwrap_or_else(DbItem::Error))
}

fn with_stmt_position(
    source_file: &Path,
    query_position: SourceSpan,
    error: anyhow::Error,
) -> anyhow::Error {
    if error.downcast_ref::<ErrorWithStatus>().is_some() || is_positioned_error(&error) {
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
            if matches!(item, DbItem::Error(_))
                && let Some(tx) = error_tx.take()
            {
                let _ = tx.send(());
            }
        })
        .take_until(error_rx)
}

async fn execute_static_simple_select(
    query: &StaticSimpleSelect,
    req: &ExecutionContext,
    db_connection: &mut DbConn,
) -> anyhow::Result<Value> {
    let mut map = serde_json::Map::with_capacity(query.columns.len());
    let mut inputs = NoInputs;
    for column in &query.columns {
        let value = column
            .value
            .evaluate(req, db_connection, &mut inputs)
            .await?
            .into_json();
        map = add_value_to_map(map, (column.name.clone(), value));
    }
    Ok(Value::Object(map))
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

/// This function is used to create a pinned boxed stream of query results.
/// This allows recursive calls.
pub fn stream_query_results_boxed<'a>(
    sql_file: &'a SqlFile,
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
    variable: &super::sql::VariableName,
    statement: &Query,
    source_file: &Path,
) -> anyhow::Result<()> {
    let value = execute_scalar_query(db_connection, request, statement, source_file).await?;

    log::debug!("Setting variable {} to {value:?}", variable.0);
    request
        .set_variables
        .borrow_mut()
        .insert(variable.0.clone(), value.map(SingleOrVec::Single));

    Ok(())
}

async fn execute_scalar_query<'a>(
    db_connection: &'a mut DbConn,
    request: &'a ExecutionContext,
    statement: &Query,
    source_file: &Path,
) -> anyhow::Result<Option<String>> {
    let QueryBody::Database(database_query) = &statement.body else {
        let QueryBody::StaticSimpleSelect(static_select) = &statement.body else {
            unreachable!()
        };
        ensure_scalar_column_count(static_select.columns.len())?;
        let row = execute_static_simple_select(static_select, request, db_connection).await?;
        return scalar_value_from_row(DbItem::Row(row));
    };
    let query = bind_query(database_query, request, db_connection).await?;
    log::debug!("Executing scalar query: {:?}", query.sql);
    let (query_span, mut query_metrics) =
        create_query_metrics(request, source_file, statement.source_span, &query);

    let mut scalar_row = None;
    let mut returned_rows: i64 = 0;
    let mut error = None;
    let scalar_subquery_behavior = request
        .app_state
        .db
        .info
        .database_type
        .scalar_subquery_behavior();
    {
        let connection = take_connection(&request.app_state.db, db_connection, request).await?;
        let mut stream = connection.fetch_many(query);
        loop {
            let start_next = std::time::Instant::now();
            let next_elem = stream.next().instrument(query_span.clone()).await;
            query_metrics.add_duration(start_next.elapsed());
            let Some(elem) = next_elem else { break };

            let result =
                parse_single_sql_result(source_file, database_query, statement.source_span, elem);
            let output_column_count = result.output_column_count;
            match result.item {
                row @ DbItem::Row(_) => {
                    returned_rows += 1;
                    if scalar_row.is_some() {
                        debug_assert_eq!(
                            scalar_subquery_behavior,
                            ScalarSubqueryBehavior::ErrorOnMultipleRows
                        );
                        error = Some(anyhow!(
                            "SET scalar query returned more than one row. A SET subquery must return zero or one row."
                        ));
                        break;
                    }
                    scalar_row = Some(QueryResult {
                        item: row,
                        inputs: result.inputs,
                        output_column_count,
                    });
                    if scalar_subquery_behavior == ScalarSubqueryBehavior::FirstRow {
                        break;
                    }
                }
                DbItem::FinishedQuery => {}
                DbItem::Error(err) => {
                    error = Some(err);
                    break;
                }
            }
        }
        drop(stream);
    }

    let value = if let Some(error) = error {
        let connection = take_connection(&request.app_state.db, db_connection, request).await?;
        return Err(
            record_error_and_rollback(connection, &query_metrics, returned_rows, error).await,
        );
    } else if let Some(mut row) = scalar_row {
        ensure_scalar_column_count(row.output_column_count)?;
        apply_json_columns(&mut row.item, &database_query.json_columns);
        if let Err(error) = evaluate_computed_columns(
            request,
            &database_query.computed_columns,
            &mut row,
            db_connection,
        )
        .instrument(query_span.clone())
        .await
        {
            let connection = take_connection(&request.app_state.db, db_connection, request).await?;
            return Err(record_error_and_rollback(
                connection,
                &query_metrics,
                returned_rows,
                error,
            )
            .await);
        }
        scalar_value_from_row(row.item)?
    } else {
        None
    };

    query_metrics.record_success(returned_rows);
    Ok(value)
}

async fn record_error_and_rollback(
    connection: &mut AnyConnection,
    query_metrics: &DbQueryMetricsContext<'_>,
    returned_rows: i64,
    error: anyhow::Error,
) -> anyhow::Error {
    query_metrics.record_error(returned_rows, &error);
    try_rollback_transaction(connection).await;
    error
}

fn scalar_value_from_row(item: DbItem) -> anyhow::Result<Option<String>> {
    let DbItem::Row(Value::Object(row)) = item else {
        anyhow::bail!("SET scalar query did not return a row object");
    };
    ensure_scalar_column_count(row.len())?;
    Ok(row
        .into_iter()
        .next()
        .and_then(|(_, value)| json_to_scalar_string(value)))
}

fn ensure_scalar_column_count(column_count: usize) -> anyhow::Result<()> {
    match column_count {
        0 => anyhow::bail!(
            "SET scalar query returned no columns. A SET subquery must select exactly one column."
        ),
        1 => Ok(()),
        _ => anyhow::bail!(
            "SET scalar query returned more than one column. A SET subquery must select exactly one column."
        ),
    }
}

fn json_to_scalar_string(value: Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(s) => Some(s),
        other => Some(other.to_string()),
    }
}

async fn take_connection<'a>(
    db: &'a Database,
    conn: &'a mut DbConn,
    request: &ExecutionContext,
) -> anyhow::Result<&'a mut PoolConnection<Any>> {
    if let Some(c) = conn {
        return Ok(c);
    }
    let pool_size = db.connection.size();
    let acquire_span = tracing::info_span!(
        "db.pool.acquire",
        { otel::DB_SYSTEM_NAME } = db.info.database_type.otel_name(),
        { otel::DB_CLIENT_CONNECTION_POOL_NAME } = "sqlpage",
        sqlpage.db.pool.size = pool_size,
    );
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
    query: &DatabaseQuery,
    source_span: SourceSpan,
    res: sqlx::error::Result<Either<AnyQueryResult, AnyRow>>,
) -> QueryResult {
    match res {
        Ok(Either::Right(r)) => {
            if log::log_enabled!(log::Level::Trace) {
                debug_row(&r);
            }
            match super::sql_to_json::row_to_json_with_inputs(&r, query.row_input_json.len()) {
                Ok((row, mut inputs)) => {
                    decode_json_values(&mut inputs, &query.row_input_json);
                    QueryResult {
                        item: DbItem::Row(row),
                        inputs: RowInputs::new(inputs),
                        output_column_count: r.columns().len() - query.row_input_json.len()
                            + query.computed_columns.len(),
                    }
                }
                Err(error) => QueryResult {
                    item: DbItem::Error(error),
                    inputs: RowInputs::new(Vec::new()),
                    output_column_count: 0,
                },
            }
        }
        Ok(Either::Left(res)) => {
            log::debug!("Finished query with result: {res:?}");
            QueryResult {
                item: DbItem::FinishedQuery,
                inputs: RowInputs::new(Vec::new()),
                output_column_count: 0,
            }
        }
        Err(err) => {
            let nice_err = display_stmt_db_error(source_file, &query.sql, source_span, err);
            QueryResult {
                item: DbItem::Error(nice_err),
                inputs: RowInputs::new(Vec::new()),
                output_column_count: 0,
            }
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
    let mut e = anyhow!(
        "{} contains a syntax error preventing SQLPage from parsing and preparing its SQL statements.",
        source_file.display()
    );
    for c in err.chain().rev() {
        e = e.context(c.to_string());
    }
    e
}

async fn bind_query<'a>(
    query: &'a DatabaseQuery,
    request: &'a ExecutionContext,
    db_connection: &mut DbConn,
) -> anyhow::Result<BoundQuery<'a>> {
    let sql = query.sql.as_str();
    log::debug!("Preparing statement: {sql}");
    let mut arguments = AnyArguments::default();
    let mut param_values = Vec::with_capacity(query.bindings.len());
    let mut inputs = NoInputs;
    for (param_idx, binding) in query.bindings.iter().enumerate() {
        log::trace!("\tevaluating binding {}: {:?}", param_idx + 1, binding);
        let argument = binding
            .evaluate(request, db_connection, &mut inputs)
            .await?
            .into_function_argument();
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
    let has_arguments = !query.bindings.is_empty();
    Ok(BoundQuery {
        sql,
        arguments,
        has_arguments,
        param_values,
    })
}

async fn evaluate_computed_columns(
    request: &ExecutionContext,
    columns: &[OutputColumn<RowExpr>],
    result: &mut QueryResult,
    db_connection: &mut DbConn,
) -> anyhow::Result<()> {
    if let DbItem::Row(Value::Object(results)) = &mut result.item {
        for column in columns {
            let value = column
                .value
                .evaluate(request, db_connection, &mut result.inputs)
                .await?
                .into_json();
            let old_results = std::mem::take(results);
            *results = add_value_to_map(old_results, (column.name.clone(), value));
        }
    }
    Ok(())
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
                        if let Value::String(json_str) = item
                            && let Ok(parsed_json) = serde_json::from_str(json_str)
                        {
                            log::trace!("Parsed JSON array item: {parsed_json}");
                            *item = parsed_json;
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

fn decode_json_values(values: &mut [Value], json_flags: &[bool]) {
    debug_assert_eq!(values.len(), json_flags.len());
    for (value, decode_as_json) in values.iter_mut().zip(json_flags) {
        if *decode_as_json
            && let Value::String(json) = value
            && let Ok(parsed) = serde_json::from_str(json)
        {
            *value = parsed;
        }
    }
}

/// Rewritten SQL and evaluated arguments in the form consumed by `sqlx`.
pub struct BoundQuery<'a> {
    sql: &'a str,
    arguments: AnyArguments<'a>,
    has_arguments: bool,
    param_values: Vec<Option<String>>,
}

impl<'q> Execute<'q, Any> for BoundQuery<'q> {
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
        closed_spans
            .lock()
            .unwrap()
            .pop()
            .expect("closed span fields")
    }

    #[test]
    fn db_query_span_uses_otel_database_client_semantics() {
        let fields = with_recorded_span_fields(|| {
            let (span, operation_name) =
                create_db_query_span("select * from users", Path::new("index.sql"), 7, "sqlite");
            assert_eq!(operation_name, "SELECT");
            drop(span);
        });

        assert_eq!(fields["otel.kind"], "client");
        assert_eq!(fields["otel.name"], "SELECT");
        assert_eq!(fields[otel::DB_QUERY_TEXT], "select * from users");
        assert_eq!(fields[otel::DB_SYSTEM_NAME], "sqlite");
        assert_eq!(fields[otel::DB_OPERATION_NAME], "SELECT");
        assert_eq!(fields[otel::CODE_FILE_PATH], "index.sql");
        assert_eq!(fields[otel::CODE_LINE_NUMBER], "7");
    }

    #[test]
    fn db_query_success_records_ok_status_and_row_count() {
        let fields = with_recorded_span_fields(|| {
            let span = tracing::info_span!(
                "db.query",
                otel.status_code = tracing::field::Empty,
                exception.message = tracing::field::Empty,
                sqlpage.exception.details = tracing::field::Empty,
                db.response.returned_rows = tracing::field::Empty,
            );
            let metrics = TelemetryMetrics::default();
            let query_metrics =
                DbQueryMetricsContext::new(span.clone(), "SELECT".to_string(), "sqlite", &metrics);
            query_metrics.record_success(3);
            drop(span);
        });

        assert_eq!(fields[otel::OTEL_STATUS_CODE], "OK");
        assert_eq!(fields[otel::DB_RESPONSE_RETURNED_ROWS], "3");
        assert!(!fields.contains_key(otel::EXCEPTION_MESSAGE));
        assert!(!fields.contains_key("sqlpage.exception.details"));
    }

    #[test]
    fn db_query_error_records_error_status_and_exception_fields() {
        let fields = with_recorded_span_fields(|| {
            let span = tracing::info_span!(
                "db.query",
                otel.status_code = tracing::field::Empty,
                exception.message = tracing::field::Empty,
                sqlpage.exception.details = tracing::field::Empty,
                db.response.returned_rows = tracing::field::Empty,
            );
            let error = anyhow!("query failed").context("while executing SELECT 1");
            let metrics = TelemetryMetrics::default();
            let query_metrics =
                DbQueryMetricsContext::new(span.clone(), "SELECT".to_string(), "sqlite", &metrics);
            query_metrics.record_error(2, &error);
            drop(span);
        });

        assert_eq!(fields[otel::OTEL_STATUS_CODE], "ERROR");
        assert_eq!(fields[otel::DB_RESPONSE_RETURNED_ROWS], "2");
        assert!(fields[otel::EXCEPTION_MESSAGE].contains("while executing SELECT 1"));
        assert!(fields["sqlpage.exception.details"].contains("query failed"));
    }
}
