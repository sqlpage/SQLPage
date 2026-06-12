use super::*;

pub(super) async fn run_sql<'a>(
    request: &'a ExecutionContext,
    db_connection: &mut DbConn,
    sql_file_path: Option<Cow<'a, str>>,
    variables: Option<Cow<'a, str>>,
) -> anyhow::Result<Option<Cow<'a, str>>> {
    use serde::ser::{SerializeSeq, Serializer};
    let Some(sql_file_path) = sql_file_path else {
        log::debug!("run_sql: first argument is NULL, returning NULL");
        return Ok(None);
    };
    let run_sql_span = tracing::info_span!(
        "sqlpage.file",
        otel.name = format!("SQL {sql_file_path}"),
        code.file.path = %sql_file_path,
    );
    let app_state = &request.app_state;
    let sql_file = app_state
        .sql_file_cache
        .get(
            app_state,
            FileAccess::privileged(std::path::Path::new(sql_file_path.as_ref())),
        )
        .instrument(run_sql_span.clone())
        .await
        .with_context(|| format!("run_sql: invalid path {sql_file_path:?}"))?;
    let tmp_req = if let Some(variables) = variables {
        let variables: SetVariablesMap = serde_json::from_str(&variables).with_context(|| {
            format!("run_sql(\'{sql_file_path}\', \'{variables}\'): the second argument should be a JSON object with string keys and values")
        })?;
        request.fork_with_variables(variables)
    } else {
        request.fork()
    };
    let max_recursion_depth = app_state.config.max_recursion_depth;
    if tmp_req.clone_depth > max_recursion_depth {
        anyhow::bail!(
            "Too many nested inclusions. run_sql can include a file that includes another file, but the depth is limited to {max_recursion_depth} levels. \n\
        Executing sqlpage.run_sql('{sql_file_path}') would exceed this limit. \n\
        This is to prevent infinite loops and stack overflows.\n\
        Make sure that your SQL file does not try to run itself, directly or through a chain of other files.\n\
        If you need to include more files, you can increase max_recursion_depth in the configuration file.\
        "
        );
    }
    let mut results_stream =
        crate::webserver::database::execute_queries::stream_query_results_boxed(
            &sql_file,
            &tmp_req,
            db_connection,
        );
    let mut json_results_bytes = Vec::new();
    let mut json_encoder = serde_json::Serializer::new(&mut json_results_bytes);
    let mut seq = json_encoder.serialize_seq(None)?;
    while let Some(db_item) = results_stream.next().instrument(run_sql_span.clone()).await {
        use crate::webserver::database::DbItem::{Error, FinishedQuery, Row};
        match db_item {
            Row(row) => {
                log::debug!("run_sql: row: {row:?}");
                seq.serialize_element(&row)?;
            }
            FinishedQuery => log::trace!("run_sql: Finished query"),
            Error(err) => {
                return Err(err.context(format!("run_sql: unable to run {sql_file_path:?}")));
            }
        }
    }
    seq.end()?;
    Ok(Some(Cow::Owned(String::from_utf8(json_results_bytes)?)))
}
