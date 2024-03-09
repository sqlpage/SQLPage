use std::{borrow::Cow, collections::HashMap};

use actix_web::http::StatusCode;
use actix_web_httpauth::headers::authorization::Basic;
use base64::Engine;
use mime_guess::{mime::APPLICATION_OCTET_STREAM, Mime};
use sqlparser::ast::FunctionArg;
use tokio_stream::StreamExt;

use crate::webserver::{
    database::{execute_queries::stream_query_results_boxed, DbItem},
    http::SingleOrVec,
    http_request_info::RequestInfo,
    ErrorWithStatus,
};

use super::sql::{
    extract_integer, extract_single_quoted_string, extract_single_quoted_string_optional,
    extract_variable_argument, function_arg_to_stmt_param, stmt_param_error_invalid_arguments,
    FormatArguments,
};
use anyhow::{anyhow, bail, Context};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum StmtParam {
    Get(String),
    AllVariables(Option<GetOrPost>),
    Post(String),
    GetOrPost(String),
    Cookie(String),
    Header(String),
    Error(String),
    BasicAuthPassword,
    BasicAuthUsername,
    HashPassword(Box<StmtParam>),
    UrlEncode(Box<StmtParam>),
    Exec(Vec<StmtParam>),
    RandomString(usize),
    CurrentWorkingDir,
    EnvironmentVariable(String),
    SqlPageVersion,
    Literal(String),
    UploadedFilePath(String),
    UploadedFileMimeType(String),
    ReadFileAsText(Box<StmtParam>),
    ReadFileAsDataUrl(Box<StmtParam>),
    RunSql(Box<StmtParam>),
    Path,
    Protocol,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum GetOrPost {
    Get,
    Post,
}

fn parse_get_or_post(arg: Option<String>) -> StmtParam {
    if let Some(s) = arg {
        if s.eq_ignore_ascii_case("get") {
            StmtParam::AllVariables(Some(GetOrPost::Get))
        } else if s.eq_ignore_ascii_case("post") {
            StmtParam::AllVariables(Some(GetOrPost::Post))
        } else {
            StmtParam::Error(format!(
                "The variables() function expected 'get' or 'post' as argument, not {s:?}"
            ))
        }
    } else {
        StmtParam::AllVariables(None)
    }
}

pub(super) fn func_call_to_param(func_name: &str, arguments: &mut [FunctionArg]) -> StmtParam {
    match func_name {
        "cookie" => extract_single_quoted_string("cookie", arguments)
            .map_or_else(StmtParam::Error, StmtParam::Cookie),
        "header" => extract_single_quoted_string("header", arguments)
            .map_or_else(StmtParam::Error, StmtParam::Header),
        "basic_auth_username" => StmtParam::BasicAuthUsername,
        "basic_auth_password" => StmtParam::BasicAuthPassword,
        "hash_password" => StmtParam::HashPassword(Box::new(extract_variable_argument(
            "hash_password",
            arguments,
        ))),
        "exec" => arguments
            .iter_mut()
            .map(function_arg_to_stmt_param)
            .collect::<Option<Vec<_>>>()
            .map(StmtParam::Exec)
            .unwrap_or_else(|| stmt_param_error_invalid_arguments("exec", arguments)),
        "random_string" => extract_integer("random_string", arguments)
            .map_or_else(StmtParam::Error, StmtParam::RandomString),
        "current_working_directory" => StmtParam::CurrentWorkingDir,
        "environment_variable" => extract_single_quoted_string("environment_variable", arguments)
            .map_or_else(StmtParam::Error, StmtParam::EnvironmentVariable),
        "url_encode" => {
            StmtParam::UrlEncode(Box::new(extract_variable_argument("url_encode", arguments)))
        }
        "version" => StmtParam::SqlPageVersion,
        "variables" => parse_get_or_post(extract_single_quoted_string_optional(arguments)),
        "path" => StmtParam::Path,
        "protocol" => StmtParam::Protocol,
        "uploaded_file_path" => extract_single_quoted_string("uploaded_file_path", arguments)
            .map_or_else(StmtParam::Error, StmtParam::UploadedFilePath),
        "uploaded_file_mime_type" => {
            extract_single_quoted_string("uploaded_file_mime_type", arguments)
                .map_or_else(StmtParam::Error, StmtParam::UploadedFileMimeType)
        }
        "read_file_as_text" => StmtParam::ReadFileAsText(Box::new(extract_variable_argument(
            "read_file_as_text",
            arguments,
        ))),
        "read_file_as_data_url" => StmtParam::ReadFileAsDataUrl(Box::new(
            extract_variable_argument("read_file_as_data_url", arguments),
        )),
        "run_sql" => StmtParam::RunSql(Box::new(extract_variable_argument("run_sql", arguments))),
        unknown_name => StmtParam::Error(format!(
            "Unknown function {unknown_name}({})",
            FormatArguments(arguments)
        )),
    }
}

/// Extracts the value of a parameter from the request.
/// Returns `Ok(None)` when NULL should be used as the parameter value.
pub(super) async fn extract_req_param<'a>(
    param: &StmtParam,
    request: &'a RequestInfo,
) -> anyhow::Result<Option<Cow<'a, str>>> {
    Ok(match param {
        StmtParam::HashPassword(inner) => has_password_param(inner, request).await?,
        StmtParam::Exec(args_params) => exec_external_command(args_params, request).await?,
        StmtParam::UrlEncode(inner) => url_encode(inner, request)?,
        StmtParam::ReadFileAsText(inner) => read_file_as_text(inner, request).await?,
        StmtParam::ReadFileAsDataUrl(inner) => read_file_as_data_url(inner, request).await?,
        StmtParam::RunSql(inner) => run_sql(inner, request).await?,
        _ => extract_req_param_non_nested(param, request)?,
    })
}

fn url_encode<'a>(
    inner: &StmtParam,
    request: &'a RequestInfo,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    let param = extract_req_param_non_nested(inner, request);
    match param {
        Ok(Some(Cow::Borrowed(inner))) => {
            let encoded = percent_encoding::percent_encode(
                inner.as_bytes(),
                percent_encoding::NON_ALPHANUMERIC,
            );
            Ok(Some(encoded.into()))
        }
        Ok(Some(Cow::Owned(inner))) => {
            let encoded = percent_encoding::percent_encode(
                inner.as_bytes(),
                percent_encoding::NON_ALPHANUMERIC,
            );
            Ok(Some(Cow::Owned(encoded.to_string())))
        }
        param => param,
    }
}

async fn exec_external_command<'a>(
    args_params: &[StmtParam],
    request: &'a RequestInfo,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    if !request.app_state.config.allow_exec {
        anyhow::bail!("The sqlpage.exec() function is disabled in the configuration. Enable it by setting the allow_exec option to true in the sqlpage.json configuration file.")
    }
    let mut iter_params = args_params.iter();
    let param0 = iter_params
        .next()
        .with_context(|| "sqlite.exec(program) requires at least one argument")?;
    let Some(program_name) = extract_req_param_non_nested(param0, request)? else {
        return Ok(None);
    };
    let mut args = Vec::with_capacity(iter_params.len());
    for arg in iter_params {
        args.push(extract_req_param_non_nested(arg, request)?.unwrap_or_else(|| "".into()));
    }
    let res = tokio::process::Command::new(&*program_name)
        .args(args.iter().map(|x| &**x))
        .output()
        .await
        .with_context(|| {
            let mut s = format!("Unable to execute command: {program_name}");
            for arg in args {
                s.push(' ');
                s.push_str(&arg);
            }
            s
        })?;
    if !res.status.success() {
        bail!(
            "Command '{program_name}' failed with exit code {}: {}",
            res.status,
            String::from_utf8_lossy(&res.stderr)
        );
    }
    Ok(Some(Cow::Owned(
        String::from_utf8_lossy(&res.stdout).to_string(),
    )))
}

async fn read_file_bytes<'a>(
    path_str: &str,
    request: &'a RequestInfo,
) -> Result<Vec<u8>, anyhow::Error> {
    let path = std::path::Path::new(path_str);
    // If the path is relative, it's relative to the web root, not the current working directory,
    // and it can be fetched from the on-database filesystem table
    if path.is_relative() {
        request
            .app_state
            .file_system
            .read_file(&request.app_state, path, true)
            .await
    } else {
        tokio::fs::read(path)
            .await
            .with_context(|| format!("Unable to read file {path:?}"))
    }
}

async fn read_file_as_text<'a>(
    param0: &StmtParam,
    request: &'a RequestInfo,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    let Some(evaluated_param) = extract_req_param_non_nested(param0, request)? else {
        log::debug!("read_file: first argument is NULL, returning NULL");
        return Ok(None);
    };
    let bytes = read_file_bytes(&evaluated_param, request).await?;
    let as_str = String::from_utf8(bytes)
        .with_context(|| format!("read_file_as_text: {param0:?} does not contain raw UTF8 text"))?;
    Ok(Some(Cow::Owned(as_str)))
}

async fn read_file_as_data_url<'a>(
    param0: &StmtParam,
    request: &'a RequestInfo,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    let Some(evaluated_param) = extract_req_param_non_nested(param0, request)? else {
        log::debug!("read_file: first argument is NULL, returning NULL");
        return Ok(None);
    };
    let bytes = read_file_bytes(&evaluated_param, request).await?;
    let mime = mime_from_upload(param0, request).map_or_else(
        || Cow::Owned(mime_guess_from_filename(&evaluated_param)),
        Cow::Borrowed,
    );
    let mut data_url = format!("data:{}/{};base64,", mime.type_(), mime.subtype());
    base64::engine::general_purpose::STANDARD.encode_string(bytes, &mut data_url);
    Ok(Some(Cow::Owned(data_url)))
}

async fn run_sql<'a>(
    param0: &StmtParam,
    request: &'a RequestInfo,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    use serde::ser::{SerializeSeq, Serializer};
    let Some(sql_file_path) = extract_req_param_non_nested(param0, request)? else {
        log::debug!("run_sql: first argument is NULL, returning NULL");
        return Ok(None);
    };
    let sql_file = request
        .app_state
        .sql_file_cache
        .get_with_privilege(
            &request.app_state,
            std::path::Path::new(sql_file_path.as_ref()),
            true,
        )
        .await
        .with_context(|| format!("run_sql: invalid path {sql_file_path:?}"))?;
    let mut tmp_req = request.clone();
    if tmp_req.clone_depth > 8 {
        bail!("Too many nested inclusions. run_sql can include a file that includes another file, but the depth is limited to 8 levels. \n\
        This is to prevent infinite loops and stack overflows.\n\
        Make sure that your SQL file does not try to run itself, directly or through a chain of other files.");
    }
    let mut results_stream = Box::pin(stream_query_results_boxed(
        &request.app_state.db,
        &sql_file,
        &mut tmp_req,
    ));
    let mut json_results_bytes = Vec::new();
    let mut json_encoder = serde_json::Serializer::new(&mut json_results_bytes);
    let mut seq = json_encoder.serialize_seq(None)?;
    while let Some(db_item) = results_stream.next().await {
        match db_item {
            DbItem::Row(row) => {
                log::debug!("run_sql: row: {:?}", row);
                seq.serialize_element(&row)?;
            }
            DbItem::FinishedQuery => log::trace!("run_sql: Finished query"),
            DbItem::Error(err) => return Err(err),
        }
    }
    Ok(Some(Cow::Owned(String::from_utf8(json_results_bytes)?)))
}

fn mime_from_upload<'a>(param0: &StmtParam, request: &'a RequestInfo) -> Option<&'a Mime> {
    if let StmtParam::UploadedFilePath(name) | StmtParam::UploadedFileMimeType(name) = param0 {
        request.uploaded_files.get(name)?.content_type.as_ref()
    } else {
        None
    }
}

fn mime_guess_from_filename(filename: &str) -> Mime {
    let maybe_mime = mime_guess::from_path(filename).first();
    maybe_mime.unwrap_or(APPLICATION_OCTET_STREAM)
}

pub(super) fn extract_req_param_non_nested<'a>(
    param: &StmtParam,
    request: &'a RequestInfo,
) -> anyhow::Result<Option<Cow<'a, str>>> {
    Ok(match param {
        StmtParam::Get(x) => request.get_variables.get(x).map(SingleOrVec::as_json_str),
        StmtParam::Post(x) => request.post_variables.get(x).map(SingleOrVec::as_json_str),
        StmtParam::GetOrPost(x) => request
            .post_variables
            .get(x)
            .or_else(|| request.get_variables.get(x))
            .map(SingleOrVec::as_json_str),
        StmtParam::Cookie(x) => request.cookies.get(x).map(SingleOrVec::as_json_str),
        StmtParam::Header(x) => request.headers.get(x).map(SingleOrVec::as_json_str),
        StmtParam::Error(x) => anyhow::bail!("{}", x),
        StmtParam::BasicAuthPassword => extract_basic_auth_password(request)
            .map(Cow::Borrowed)
            .map(Some)?,
        StmtParam::BasicAuthUsername => extract_basic_auth_username(request)
            .map(Cow::Borrowed)
            .map(Some)?,
        StmtParam::HashPassword(_) => bail!("Nested hash_password() function not allowed"),
        StmtParam::Exec(_) => bail!("Nested exec() function not allowed"),
        StmtParam::UrlEncode(_) => bail!("Nested url_encode() function not allowed"),
        StmtParam::RandomString(len) => Some(Cow::Owned(random_string(*len))),
        StmtParam::CurrentWorkingDir => cwd()?,
        StmtParam::EnvironmentVariable(var) => std::env::var(var)
            .map(Cow::Owned)
            .map(Some)
            .with_context(|| format!("Unable to read environment variable {var}"))?,
        StmtParam::SqlPageVersion => Some(Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
        StmtParam::Literal(x) => Some(Cow::Owned(x.to_string())),
        StmtParam::AllVariables(get_or_post) => extract_get_or_post(*get_or_post, request),
        StmtParam::Path => Some(Cow::Borrowed(&request.path)),
        StmtParam::Protocol => Some(Cow::Borrowed(&request.protocol)),
        StmtParam::UploadedFilePath(x) => request
            .uploaded_files
            .get(x)
            .and_then(|x| x.file.path().to_str())
            .map(Cow::Borrowed),
        StmtParam::UploadedFileMimeType(x) => request
            .uploaded_files
            .get(x)
            .and_then(|x| x.content_type.as_ref())
            .map(|x| Cow::Borrowed(x.as_ref())),
        StmtParam::ReadFileAsText(_) => bail!("Nested read_file_as_text() function not allowed",),
        StmtParam::ReadFileAsDataUrl(_) => {
            bail!("Nested read_file_as_data_url() function not allowed")
        }
        StmtParam::RunSql(_) => bail!("Nested run_sql() function not allowed"),
    })
}

fn extract_get_or_post(
    get_or_post: Option<GetOrPost>,
    request: &RequestInfo,
) -> Option<Cow<'_, str>> {
    match get_or_post {
        Some(GetOrPost::Get) => serde_json::to_string(&request.get_variables),
        Some(GetOrPost::Post) => serde_json::to_string(&request.post_variables),
        None => {
            let all: HashMap<_, _> = request
                .get_variables
                .iter()
                .chain(&request.post_variables)
                .collect();
            serde_json::to_string(&all)
        }
    }
    .map_err(|e| log::warn!("{}", e))
    .map(Cow::Owned)
    .ok()
}

fn random_string(len: usize) -> String {
    use rand::{distributions::Alphanumeric, Rng};
    password_hash::rand_core::OsRng
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

async fn has_password_param<'a>(
    inner: &StmtParam,
    request: &'a RequestInfo,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    let password = match extract_req_param_non_nested(inner, request) {
        Ok(Some(x)) => x,
        err => return err,
    }
    .into_owned();
    let encoded = actix_web::rt::task::spawn_blocking(move || hash_password(&password)).await??;
    Ok(Some(Cow::Owned(encoded)))
}

/// Hashes a password using Argon2. This is a CPU-intensive blocking operation.
fn hash_password(password: &str) -> anyhow::Result<String> {
    let phf = argon2::Argon2::default();
    let salt = password_hash::SaltString::generate(&mut password_hash::rand_core::OsRng);
    let password_hash = &password_hash::PasswordHash::generate(phf, password, &salt)
        .map_err(|e| anyhow!("Unable to hash password: {}", e))?;
    Ok(password_hash.to_string())
}

fn extract_basic_auth_username(request: &RequestInfo) -> anyhow::Result<&str> {
    Ok(extract_basic_auth(request)?.user_id())
}

fn extract_basic_auth_password(request: &RequestInfo) -> anyhow::Result<&str> {
    let password = extract_basic_auth(request)?.password().ok_or_else(|| {
        anyhow::Error::new(ErrorWithStatus {
            status: StatusCode::UNAUTHORIZED,
        })
    })?;
    Ok(password)
}

fn extract_basic_auth(request: &RequestInfo) -> anyhow::Result<&Basic> {
    request
        .basic_auth
        .as_ref()
        .ok_or_else(|| {
            anyhow::Error::new(ErrorWithStatus {
                status: StatusCode::UNAUTHORIZED,
            })
        })
        .with_context(|| "Expected the user to be authenticated with HTTP basic auth")
}

fn cwd() -> anyhow::Result<Option<Cow<'static, str>>> {
    let cwd = std::env::current_dir()
        .with_context(|| "unable to access the current working directory")?;
    Ok(Some(Cow::Owned(cwd.to_string_lossy().to_string())))
}
