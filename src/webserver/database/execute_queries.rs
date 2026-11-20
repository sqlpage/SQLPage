use anyhow::{Context, anyhow};
use futures_util::stream::Stream;
use futures_util::{StreamExt, TryStreamExt, stream};
use serde_json::Value;
use std::borrow::Cow;
use std::path::Path;
use std::pin::Pin;
use tracing::Instrument;

use super::csv_import::run_csv_import;
use super::error_highlighting::{display_stmt_db_error, display_stmt_error, is_positioned_error};
use super::sql::{
    DatabaseQuery, FileStatement, Query, QueryBody, SourceSpan, SqlFile, StaticSimpleSelect,
};
use super::sqlpage_expr::{NoInputs, RowInputs};
use crate::dynamic_component::parse_dynamic_rows;
use crate::utils::add_value_to_map;
use crate::webserver::ErrorWithStatus;
use crate::webserver::http_request_info::ExecutionContext;
use crate::webserver::single_or_vec::SingleOrVec;

use super::{
    Database, DatabaseConnection, DbInfo, DbItem, ScalarSubqueryBehavior, SupportedDatabase,
};

pub type DbConn = Option<Box<dyn DatabaseConnection>>;

/// Database events before computed columns and dynamic components are evaluated.
pub enum QueryEvent {
    Row(QueryRow),
    FinishedQuery,
}

/// A decoded row together with private values reserved for computed columns.
pub struct QueryRow {
    pub(super) value: Value,
    pub(super) inputs: Vec<Value>,
    pub(super) public_column_count: usize,
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
                    Box::pin(run_csv_import(connection, request.app_state.db.info, csv_import, request)).await.with_context(|| format!("Failed to import the CSV file {:?} into the table {:?}", csv_import.uploaded_file, csv_import.table_name))?;
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
                    let mut query = RunningQuery::bind(stmt, source_file, statement.source_span, request, db_connection)
                        .await.map_err(|e| with_stmt_position(source_file, statement.source_span, e))?;
                    request.server_timing.record("bind_params");
                    // Fail to acquire before starting the statement or trying to roll it back.
                    take_connection(&request.app_state.db, db_connection, request).await?;
                    let error = {
                        let items = query.project_rows(request, db_connection);
                        futures_util::pin_mut!(items);
                        loop {
                            match items.try_next().await {
                                Ok(Some(item)) => for row in parse_dynamic_rows(item) { yield row; },
                                Ok(None) => break None,
                                Err(error) => break Some(error),
                            }
                        }
                    };
                    if let Some(error) = error {
                        yield DbItem::Error(query.rollback(db_connection, error).await);
                    } else {
                        query.metrics.record_success(query.returned_rows);
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
    async_stream::stream! {
        futures_util::pin_mut!(results_stream);
        while let Some(item) = results_stream.next().await {
            let failed = matches!(item, DbItem::Error(_));
            yield item;
            if failed {
                break;
            }
        }
    }
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

async fn try_rollback_transaction(db_connection: &mut dyn DatabaseConnection) {
    log::debug!("Attempting to rollback transaction");
    match db_connection.execute("ROLLBACK").await {
        Ok(()) => log::debug!("Rolled back transaction"),
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
    let stmt = match &statement.body {
        QueryBody::Database(stmt) => stmt,
        QueryBody::StaticSimpleSelect(stmt) => {
            ensure_scalar_column_count(stmt.columns.len())?;
            let row = execute_static_simple_select(stmt, request, db_connection).await?;
            return scalar_value_from_row(row);
        }
    };
    let mut query = RunningQuery::bind(
        stmt,
        source_file,
        statement.source_span,
        request,
        db_connection,
    )
    .await?;
    let connection = take_connection(&request.app_state.db, db_connection, request).await?;
    let behavior = request
        .app_state
        .db
        .info
        .database_type()
        .scalar_subquery_behavior();
    let row = first_scalar_row(query.fetch(connection), behavior).await;

    // Release the fetch stream before evaluating computed columns on its connection.
    let value = match row {
        Ok(Some(row)) => {
            ensure_scalar_column_count(row.public_column_count + stmt.computed_columns.len())?;
            prepare_query_row(request, stmt, row, db_connection)
                .instrument(query.metrics.span.clone())
                .await
                .map(Some)
        }
        Ok(None) => Ok(None),
        Err(error) => Err(error),
    };
    match value {
        Ok(value) => {
            let scalar = value.map(scalar_value_from_row).transpose()?.flatten();
            query.metrics.record_success(query.returned_rows);
            Ok(scalar)
        }
        Err(error) => Err(query.rollback(db_connection, error).await),
    }
}

async fn first_scalar_row(
    events: impl Stream<Item = anyhow::Result<QueryEvent>> + Unpin,
    behavior: ScalarSubqueryBehavior,
) -> anyhow::Result<Option<QueryRow>> {
    let mut rows = events.try_filter_map(|event| {
        std::future::ready(Ok(match event {
            QueryEvent::Row(row) => Some(row),
            QueryEvent::FinishedQuery => None,
        }))
    });
    let first = rows.try_next().await?;
    if first.is_some()
        && behavior == ScalarSubqueryBehavior::ErrorOnMultipleRows
        && rows.try_next().await?.is_some()
    {
        anyhow::bail!(
            "SET scalar query returned more than one row. A SET subquery must return zero or one row."
        );
    }
    Ok(first)
}

fn scalar_value_from_row(value: Value) -> anyhow::Result<Option<String>> {
    let Value::Object(row) = value else {
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
) -> anyhow::Result<&'a mut dyn DatabaseConnection> {
    if let Some(c) = conn {
        return Ok(c.as_mut());
    }
    let pool_size = db.connection.size();
    let acquire_span = tracing::info_span!(
        "db.pool.acquire",
        { otel::DB_SYSTEM_NAME } = db.info.database_type().otel_name(),
        { otel::DB_CLIENT_CONNECTION_POOL_NAME } = "sqlpage",
        sqlpage.db.pool.size = pool_size,
    );
    match db.connection.acquire().instrument(acquire_span).await {
        Ok(c) => {
            log::debug!("Acquired a database connection");
            request.server_timing.record("db_conn");
            let connection = conn.insert(c).as_mut();
            set_trace_context(connection, db).await;
            Ok(connection)
        }
        Err(e) => {
            let active_count = db.connection.size();
            let err_msg = format!(
                "Unable to connect to {db}. The connection pool currently has {active_count} active connections."
            );
            Err(anyhow::Error::new(e).context(err_msg))
        }
    }
}

/// Sets the current `OTel` trace context on the database connection so it is visible
/// in `pg_stat_activity.application_name` (`PostgreSQL`) or as a session variable (`MySQL`).
/// This allows correlating `SQLPage` traces with database-side monitoring.
async fn set_trace_context(connection: &mut dyn DatabaseConnection, db: &Database) {
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
    let sql = match db.info {
        DbInfo::Native(SupportedDatabase::Postgres) => {
            // postgresqlreceiver expects application_name to be a raw W3C traceparent value.
            format!("SET application_name = '{traceparent}'")
        }
        DbInfo::Native(SupportedDatabase::MySql) => {
            format!("SET @traceparent = '{traceparent}'")
        }
        _ => return,
    };
    if let Err(e) = connection.execute(&sql).await {
        log::debug!("Failed to set trace context on connection: {e}");
    }
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

struct RunningQuery<'a> {
    statement: &'a DatabaseQuery,
    params: Vec<Option<String>>,
    source_file: &'a Path,
    source_span: SourceSpan,
    metrics: DbQueryMetricsContext<'a>,
    returned_rows: i64,
}

impl<'a> RunningQuery<'a> {
    async fn bind(
        statement: &'a DatabaseQuery,
        source_file: &'a Path,
        source_span: SourceSpan,
        request: &'a ExecutionContext,
        connection: &mut DbConn,
    ) -> anyhow::Result<Self> {
        let mut params = Vec::with_capacity(statement.bindings.len());
        for binding in &statement.bindings {
            let value = binding
                .evaluate(request, connection, &mut NoInputs)
                .await?
                .into_function_argument();
            params.push(value.map(Cow::into_owned));
        }
        let system = request.app_state.db.info.database_type().otel_name();
        let (span, operation) =
            create_db_query_span(&statement.sql, source_file, source_span.start.line, system);
        log::debug!("Executing SQL: {}", statement.sql);
        record_query_params(&span, &params);
        let metrics = DbQueryMetricsContext::new(
            span,
            operation,
            system,
            &request.app_state.telemetry_metrics,
        );
        Ok(Self {
            statement,
            source_file,
            source_span,
            params,
            metrics,
            returned_rows: 0,
        })
    }

    fn fetch<'c>(
        &'c mut self,
        connection: &'c mut dyn DatabaseConnection,
    ) -> impl Stream<Item = anyhow::Result<QueryEvent>> + Unpin + 'c {
        let mut rows = connection.fetch_many(
            &self.statement.sql,
            &self.params,
            &self.statement.row_input_json,
        );
        Box::pin(async_stream::try_stream! {
            loop {
                let start = std::time::Instant::now();
                let next = rows.next().instrument(self.metrics.span.clone()).await;
                self.metrics.add_duration(start.elapsed());
                let Some(event) = next else { break };
                let event = event.map_err(|e| display_stmt_db_error(self.source_file, &self.statement.sql, self.source_span, e))?;
                if matches!(event, QueryEvent::Row(_)) { self.returned_rows += 1; }
                yield event;
            }
        })
    }

    fn project_rows<'c>(
        &'c mut self,
        request: &'c ExecutionContext,
        connection: &'c mut DbConn,
    ) -> impl Stream<Item = anyhow::Result<DbItem>> + 'c {
        async_stream::try_stream! {
            let stmt = self.statement;
            let span = self.metrics.span.clone();
            let events = self.fetch(connection.as_deref_mut().expect("connection acquired for query"));
            let mut unused = None;
            let (mut events, nested_connection) = if stmt.must_buffer_rows() {
                // Nested SQL must use this session after its active fetch has finished.
                let rows: Vec<_> = events.try_collect().await?;
                (stream::iter(rows.into_iter().map(Ok)).left_stream(), connection)
            } else {
                (events.right_stream(), &mut unused)
            };
            while let Some(event) = events.try_next().await? {
                yield prepare_query_event(request, stmt, event, nested_connection).instrument(span.clone()).await?;
            }
        }
    }

    async fn rollback(&self, connection: &mut DbConn, error: anyhow::Error) -> anyhow::Error {
        self.metrics.record_error(self.returned_rows, &error);
        if let Some(connection) = connection.as_deref_mut() {
            try_rollback_transaction(connection).await;
        }
        error
    }
}

async fn prepare_query_event(
    request: &ExecutionContext,
    query: &DatabaseQuery,
    event: QueryEvent,
    db_connection: &mut DbConn,
) -> anyhow::Result<DbItem> {
    match event {
        QueryEvent::Row(row) => Ok(DbItem::Row(
            prepare_query_row(request, query, row, db_connection).await?,
        )),
        QueryEvent::FinishedQuery => Ok(DbItem::FinishedQuery),
    }
}

async fn prepare_query_row(
    request: &ExecutionContext,
    query: &DatabaseQuery,
    mut row: QueryRow,
    db_connection: &mut DbConn,
) -> anyhow::Result<Value> {
    apply_json_columns(&mut row.value, &query.json_columns);
    if let Value::Object(results) = &mut row.value {
        let mut inputs = RowInputs::new(row.inputs);
        for column in &query.computed_columns {
            let value = column
                .value
                .evaluate(request, db_connection, &mut inputs)
                .await?
                .into_json();
            let old_results = std::mem::take(results);
            *results = add_value_to_map(old_results, (column.name.clone(), value));
        }
    }
    Ok(row.value)
}

fn apply_json_columns(value: &mut Value, json_columns: &[String]) {
    if let Value::Object(row) = value {
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

    fn assert_json_value(item: &Value, key: &str, expected: Value) {
        let Value::Object(row) = item else {
            panic!("Expected a row object");
        };
        assert_eq!(row[key], expected);
        drop(expected);
    }

    #[test]
    fn test_basic_json_string_conversion() {
        let mut item = json!({
            "json_col": "{\"key\": \"value\"}",
            "normal_col": "text"
        });
        apply_json_columns(&mut item, &["json_col".to_string()]);
        assert_json_value(&item, "json_col", json!({"key": "value"}));
        assert_json_value(&item, "normal_col", json!("text"));
    }

    #[test]
    fn test_json_array_conversion() {
        let mut item = json!({
            "array_col": ["{\"a\": 1}", "{\"b\": 2}"],
            "normal_array": ["text"]
        });
        apply_json_columns(&mut item, &["array_col".to_string()]);
        assert_json_value(&item, "array_col", json!([{"a": 1}, {"b": 2}]));
        assert_json_value(&item, "normal_array", json!(["text"]));
    }

    #[test]
    fn test_invalid_json_handling() {
        let mut item = json!({
            "invalid_json": "{not valid json}",
            "normal_col": "text"
        });
        apply_json_columns(&mut item, &["invalid_json".to_string()]);
        assert_json_value(&item, "invalid_json", json!("{not valid json}"));
        assert_json_value(&item, "normal_col", json!("text"));
    }

    #[test]
    fn test_missing_column_handling() {
        let mut item = json!({
            "existing_col": "text"
        });
        apply_json_columns(&mut item, &["missing_col".to_string()]);
        assert_json_value(&item, "existing_col", json!("text"));
    }

    #[test]
    fn test_non_object_json_handling() {
        let mut item = Value::Null;
        apply_json_columns(&mut item, &["json_col".to_string()]);
        assert!(item.is_null());
    }

    #[test]
    fn test_duplicate_json_column_names() {
        let mut item = json!({
            "json_col": "{\"key\": \"value\"}",
            "normal_col": "text"
        });
        apply_json_columns(&mut item, &["json_col".to_string(), "json_col".to_string()]);
        assert_json_value(&item, "json_col", json!({"key": "value"}));
        assert_json_value(&item, "normal_col", json!("text"));
    }

    #[actix_web::test]
    async fn an_error_ends_the_stream_without_polling_the_next_statement() {
        let error = stream::iter([DbItem::Error(anyhow!("bad query"))]);
        let later = stream::poll_fn(|_| panic!("polled another statement after the error"));
        assert_eq!(stop_at_first_error(error.chain(later)).count().await, 1);
    }

    #[actix_web::test]
    async fn dropping_stream_stops_later_rows_and_releases_connection_borrow() {
        let mut config: crate::app_config::AppConfig = serde_json::from_str("{}").unwrap();
        config.database_url = crate::app_config::test_database_url();
        config.max_database_pool_connections = Some(1);
        config.database_connection_acquire_timeout_seconds = 1.0;
        let state = Arc::new(crate::AppState::init(&config).await.unwrap());
        let mut request = actix_web::test::TestRequest::get().to_srv_request();
        let request = crate::webserver::http_request_info::extract_request_info(
            &mut request,
            state,
            crate::webserver::server_timing::ServerTiming::default(),
        )
        .await
        .unwrap();
        let file = SqlFile::new(
            &request.app_state.db,
            // Text markers keep this ordering test independent of driver numeric representations.
            "SELECT n, sqlpage.fetch(url) AS value
             FROM (SELECT 'row1' AS n, NULL AS url UNION ALL SELECT 'row2' AS n, 'invalid URL' AS url) source_rows
             ORDER BY n;
             SET after_query = 'unexpected'",
            Path::new("execution-test.sql"),
        );
        let mut connection = None;
        {
            let stream = stream_query_results_with_conn(&file, &request, &mut connection);
            futures_util::pin_mut!(stream);
            let Some(DbItem::Row(row)) = stream.next().await else {
                panic!("Expected the first row before the later function error");
            };
            assert_eq!(row, json!({"n": "row1", "value": null}));
        }
        assert!(!request.set_variables.borrow().contains_key("after_query"));
        let file = SqlFile::new(
            &request.app_state.db,
            "SELECT n FROM (SELECT 'reused' AS n) source_rows",
            Path::new("execution-test.sql"),
        );
        let stream = stream_query_results_with_conn(&file, &request, &mut connection);
        futures_util::pin_mut!(stream);
        let Some(DbItem::Row(row)) = stream.next().await else {
            panic!("Expected a row from the reused connection");
        };
        assert_eq!(row, json!({"n": "reused"}));
        while let Some(item) = stream.next().await {
            assert!(matches!(item, DbItem::FinishedQuery));
        }
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
