use std::{borrow::Cow, collections::HashMap, str::FromStr};

use actix_web::http::StatusCode;
use actix_web_httpauth::headers::authorization::Basic;
use awc::http::{header::USER_AGENT, Method};
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

#[derive(Debug, PartialEq, Eq, Clone)]
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
    UrlEncode(Box<StmtParam>),
    Exec(Vec<StmtParam>),
    RandomString(usize),
    CurrentWorkingDir,
    EnvironmentVariable(String),
    SqlPageVersion,
    Literal(String),
    Concat(Vec<StmtParam>),
    UploadedFilePath(String),
    UploadedFileMimeType(String),
    PersistUploadedFile {
        field_name: Box<StmtParam>,
        folder: Option<Box<StmtParam>>,
        allowed_extensions: Option<Box<StmtParam>>,
    },
    ReadFileAsText(Box<StmtParam>),
    ReadFileAsDataUrl(Box<StmtParam>),
    RunSql(Box<StmtParam>),
    Fetch(Box<StmtParam>),
    Path,
    Protocol,
    FunctionCall(SqlPageFunctionCall),
}

impl std::fmt::Display for StmtParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StmtParam::Get(name) => write!(f, "?{name}"),
            StmtParam::Post(name) => write!(f, ":{name}"),
            StmtParam::GetOrPost(name) => write!(f, "${name}"),
            StmtParam::Literal(x) => write!(f, "'{}'", x.replace('\'', "''")),
            StmtParam::Concat(items) => {
                write!(f, "CONCAT(")?;
                for item in items {
                    write!(f, "{}, ", item)?;
                }
                write!(f, ")")
            }
            StmtParam::FunctionCall(call) => write!(f, "{call}"),
            _ => todo!(),
        }
    }
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
        "persist_uploaded_file" => {
            let field_name = Box::new(extract_variable_argument(
                "persist_uploaded_file",
                arguments,
            ));
            let folder = arguments
                .get_mut(1)
                .and_then(function_arg_to_stmt_param)
                .map(Box::new);
            let allowed_extensions = arguments
                .get_mut(2)
                .and_then(function_arg_to_stmt_param)
                .map(Box::new);
            StmtParam::PersistUploadedFile {
                field_name,
                folder,
                allowed_extensions,
            }
        }
        "read_file_as_text" => StmtParam::ReadFileAsText(Box::new(extract_variable_argument(
            "read_file_as_text",
            arguments,
        ))),
        "read_file_as_data_url" => StmtParam::ReadFileAsDataUrl(Box::new(
            extract_variable_argument("read_file_as_data_url", arguments),
        )),
        "run_sql" => StmtParam::RunSql(Box::new(extract_variable_argument("run_sql", arguments))),
        "fetch" => StmtParam::Fetch(Box::new(extract_variable_argument("fetch", arguments))),
        _ => SqlPageFunctionCall::from_func_call(func_name, arguments)
            .with_context(|| {
                format!(
                    "Invalid function call: sqlpage.{func_name}({})",
                    FormatArguments(arguments)
                )
            })
            .map_or_else(
                |e| StmtParam::Error(format!("{e:#}")),
                StmtParam::FunctionCall,
            ),
    }
}

const DEFAULT_ALLOWED_EXTENSIONS: &str =
    "jpg,jpeg,png,gif,bmp,webp,pdf,txt,doc,docx,xls,xlsx,csv,mp3,mp4,wav,avi,mov";

async fn persist_uploaded_file<'a>(
    field_name: &StmtParam,
    folder: Option<&StmtParam>,
    allowed_extensions: Option<&StmtParam>,
    request: &'a RequestInfo,
) -> anyhow::Result<Option<Cow<'a, str>>> {
    let field_name = Box::pin(extract_req_param(field_name, request))
        .await?
        .ok_or_else(|| anyhow!("persist_uploaded_file: field_name is NULL"))?;
    let folder = if let Some(x) = folder {
        Box::pin(extract_req_param(x, request)).await?
    } else {
        None
    }
    .unwrap_or(Cow::Borrowed("uploads"));
    let allowed_extensions_str = if let Some(x) = allowed_extensions {
        Box::pin(extract_req_param(x, request)).await?
    } else {
        None
    }
    .unwrap_or(Cow::Borrowed(DEFAULT_ALLOWED_EXTENSIONS));
    let allowed_extensions = allowed_extensions_str.split(',');
    let uploaded_file = request
        .uploaded_files
        .get(&field_name.to_string())
        .ok_or_else(|| {
            anyhow!("persist_uploaded_file: no file uploaded with field name {field_name}. Uploaded files: {:?}", request.uploaded_files.keys())
        })?;
    let file_name = &uploaded_file.file_name.as_deref().unwrap_or_default();
    let extension = file_name.split('.').last().unwrap_or_default();
    if !allowed_extensions
        .clone()
        .any(|x| x.eq_ignore_ascii_case(extension))
    {
        let exts = allowed_extensions.collect::<Vec<_>>().join(", ");
        bail!(
            "persist_uploaded_file: file extension {extension} is not allowed. Allowed extensions: {exts}"
        );
    }
    // resolve the folder path relative to the web root
    let web_root = &request.app_state.config.web_root;
    let target_folder = web_root.join(&*folder);
    // create the folder if it doesn't exist
    tokio::fs::create_dir_all(&target_folder)
        .await
        .with_context(|| {
            format!("persist_uploaded_file: unable to create folder {target_folder:?}")
        })?;
    let date = chrono::Utc::now().format("%Y-%m-%d %Hh%Mm%Ss");
    let random_part = random_string(8);
    let random_target_name = format!("{date} {random_part}.{extension}");
    let target_path = target_folder.join(&random_target_name);
    tokio::fs::copy(&uploaded_file.file.path(), &target_path)
        .await
        .with_context(|| {
            format!(
                "persist_uploaded_file: unable to copy uploaded file {field_name:?} to {target_path:?}"
            )
        })?;
    // remove the WEB_ROOT prefix from the path, but keep the leading slash
    let path = "/".to_string()
        + target_path
            .strip_prefix(web_root)?
            .to_str()
            .with_context(|| {
                format!("persist_uploaded_file: unable to convert path {target_path:?} to a string")
            })?;
    Ok(Some(Cow::Owned(path)))
}

async fn url_encode<'a>(
    inner: &StmtParam,
    request: &'a RequestInfo,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    let param = Box::pin(extract_req_param(inner, request)).await;
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
    let Some(program_name) = Box::pin(extract_req_param(param0, request)).await? else {
        return Ok(None);
    };
    let mut args = Vec::with_capacity(iter_params.len());
    for arg in iter_params {
        args.push(
            Box::pin(extract_req_param(arg, request))
                .await?
                .unwrap_or_else(|| "".into()),
        );
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
    let Some(evaluated_param) = Box::pin(extract_req_param(param0, request)).await? else {
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
    let Some(evaluated_param) = Box::pin(extract_req_param(param0, request)).await? else {
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
    let Some(sql_file_path) = Box::pin(extract_req_param(param0, request)).await? else {
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
        Executing sqlpage.run_sql('{sql_file_path}') would exceed this limit. \n\
        This is to prevent infinite loops and stack overflows.\n\
        Make sure that your SQL file does not try to run itself, directly or through a chain of other files.");
    }
    let mut results_stream =
        stream_query_results_boxed(&request.app_state.db, &sql_file, &mut tmp_req);
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
            DbItem::Error(err) => {
                return Err(err.context(format!("run_sql: unable to run {sql_file_path:?}")))
            }
        }
    }
    seq.end()?;
    Ok(Some(Cow::Owned(String::from_utf8(json_results_bytes)?)))
}

type HeaderVec<'a> = Vec<(Cow<'a, str>, Cow<'a, str>)>;
#[derive(serde::Deserialize)]
struct Req<'b> {
    #[serde(borrow)]
    url: Cow<'b, str>,
    #[serde(borrow)]
    method: Option<Cow<'b, str>>,
    #[serde(borrow, deserialize_with = "deserialize_map_to_vec_pairs")]
    headers: HeaderVec<'b>,
    #[serde(borrow)]
    body: Option<&'b serde_json::value::RawValue>,
}

fn deserialize_map_to_vec_pairs<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<HeaderVec<'de>, D::Error> {
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Vec<(Cow<'de, str>, Cow<'de, str>)>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some((key, value)) = map.next_entry()? {
                vec.push((key, value));
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_map(Visitor)
}

async fn fetch<'a>(
    param0: &StmtParam,
    request: &'a RequestInfo,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    let Some(fetch_target) = Box::pin(extract_req_param(param0, request)).await? else {
        log::debug!("fetch: first argument is NULL, returning NULL");
        return Ok(None);
    };
    let client = awc::Client::builder()
        .add_default_header((USER_AGENT, env!("CARGO_PKG_NAME")))
        .finish();
    let res = if fetch_target.starts_with("http") {
        client.get(fetch_target.as_ref()).send()
    } else {
        let r = serde_json::from_str::<'_, Req<'_>>(&fetch_target)
            .with_context(|| format!("Invalid request: {fetch_target}"))?;
        let method = if let Some(method) = r.method {
            Method::from_str(&method)?
        } else {
            Method::GET
        };
        let mut req = client.request(method, r.url.as_ref());
        for (k, v) in r.headers {
            req = req.insert_header((k.as_ref(), v.as_ref()));
        }
        if let Some(body) = r.body {
            let val = body.get();
            // The body can be either json, or a string representing a raw body
            let body = if val.starts_with('"') {
                serde_json::from_str::<'_, String>(val)?
            } else {
                req = req.content_type("application/json");
                val.to_owned()
            };
            req.send_body(body)
        } else {
            req.send()
        }
    };
    log::info!("Fetching {fetch_target}");
    let mut res = res
        .await
        .map_err(|e| anyhow!("Unable to fetch {fetch_target}: {e}"))?;
    log::debug!("Finished fetching {fetch_target}. Status: {}", res.status());
    let body = res.body().await?.to_vec();
    let response_str = String::from_utf8(body)?.into();
    log::debug!("Fetch response: {response_str}");
    Ok(Some(response_str))
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

/// Extracts the value of a parameter from the request.
/// Returns `Ok(None)` when NULL should be used as the parameter value.
pub(super) async fn extract_req_param_as_json(
    param: &StmtParam,
    request: &RequestInfo,
) -> anyhow::Result<serde_json::Value> {
    if let Some(val) = extract_req_param(param, request).await? {
        Ok(serde_json::Value::String(val.into_owned()))
    } else {
        Ok(serde_json::Value::Null)
    }
}

/// Extracts the value of a parameter from the request.
/// Returns `Ok(None)` when NULL should be used as the parameter value.
pub(super) async fn extract_req_param<'a>(
    param: &StmtParam,
    request: &'a RequestInfo,
) -> anyhow::Result<Option<Cow<'a, str>>> {
    Ok(match param {
        // async functions
        StmtParam::Exec(args_params) => exec_external_command(args_params, request).await?,
        StmtParam::UrlEncode(inner) => url_encode(inner, request).await?,
        StmtParam::ReadFileAsText(inner) => read_file_as_text(inner, request).await?,
        StmtParam::ReadFileAsDataUrl(inner) => read_file_as_data_url(inner, request).await?,
        StmtParam::RunSql(inner) => run_sql(inner, request).await?,
        StmtParam::Fetch(inner) => fetch(inner, request).await?,
        StmtParam::PersistUploadedFile {
            field_name,
            folder,
            allowed_extensions,
        } => {
            persist_uploaded_file(
                field_name,
                folder.as_deref(),
                allowed_extensions.as_deref(),
                request,
            )
            .await?
        }
        // sync functions
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
        StmtParam::RandomString(len) => Some(Cow::Owned(random_string(*len))),
        StmtParam::CurrentWorkingDir => cwd()?,
        StmtParam::EnvironmentVariable(var) => std::env::var(var)
            .map(Cow::Owned)
            .map(Some)
            .with_context(|| format!("Unable to read environment variable {var}"))?,
        StmtParam::SqlPageVersion => Some(Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
        StmtParam::Literal(x) => Some(Cow::Owned(x.to_string())),
        StmtParam::Concat(args) => concat_params(&args[..], request).await?,
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
        StmtParam::FunctionCall(func) => func.evaluate(request).await.with_context(|| {
            format!(
                "Error in function call {func}.\nExpected {:#}",
                func.function
            )
        })?,
    })
}

async fn concat_params<'a>(
    args: &[StmtParam],
    request: &'a RequestInfo,
) -> anyhow::Result<Option<Cow<'a, str>>> {
    let mut result = String::new();
    for arg in args {
        let Some(arg) = Box::pin(extract_req_param(arg, request)).await? else {
            return Ok(None);
        };
        result.push_str(&arg);
    }
    Ok(Some(Cow::Owned(result)))
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

async fn hash_password<'a>(
    _request: &'a RequestInfo,
    password: String,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    let encoded =
        actix_web::rt::task::spawn_blocking(move || hash_password_blocking(&password)).await??;
    Ok(Some(Cow::Owned(encoded)))
}

/// Hashes a password using Argon2. This is a CPU-intensive blocking operation.
fn hash_password_blocking(password: &str) -> anyhow::Result<String> {
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SqlPageFunctionCall {
    function: SqlPageFunctionName,
    arguments: Vec<StmtParam>,
}

impl SqlPageFunctionCall {
    pub fn from_func_call(func_name: &str, arguments: &mut [FunctionArg]) -> anyhow::Result<Self> {
        let function = SqlPageFunctionName::from_str(func_name)?;
        let arguments = arguments
            .iter_mut()
            .map(|arg| {
                function_arg_to_stmt_param(arg)
                    .ok_or_else(|| anyhow!("Invalid argument format \"{arg}\" in {function:#}"))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(Self {
            function,
            arguments,
        })
    }

    pub async fn evaluate<'a>(
        &self,
        request: &'a RequestInfo,
    ) -> anyhow::Result<Option<Cow<'a, str>>> {
        let evaluated_args = self.arguments.iter().map(|x| extract_req_param(x, request));
        let evaluated_args = futures_util::future::try_join_all(evaluated_args).await?;
        self.function.evaluate(request, evaluated_args).await
    }
}

impl std::fmt::Display for SqlPageFunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}(", self.function)?;
        // interleave the arguments with commas
        let mut it = self.arguments.iter();
        if let Some(x) = it.next() {
            write!(f, "{}", x)?;
        }
        for x in it {
            write!(f, ", {}", x)?;
        }
        write!(f, ")")
    }
}

/// Defines all sqlpage functions using a simple syntax:
/// `sqlpage_functions! {
///    simple_function(param1, param2);
///    function_with_optional_param(param1, param2 optional);
///    function_with_varargs(param1, param2 repeated);
/// }`
macro_rules! sqlpage_functions {
    ($($func_name:ident($($param_name:ident : $param_type:ty),*);)*) => {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub enum SqlPageFunctionName {
            $( #[allow(non_camel_case_types)] $func_name ),*
        }

        impl FromStr for SqlPageFunctionName {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> anyhow::Result<Self> {
                match s {
                    $(stringify!($func_name) => Ok(SqlPageFunctionName::$func_name),)*
                    unknown_name => anyhow::bail!("Unknown function {unknown_name:?}"),
                }
            }
        }

        impl std::fmt::Display for SqlPageFunctionName {
            #[allow(unused_assignments)]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $(SqlPageFunctionName::$func_name => {
                        write!(f, "sqlpage.{}", stringify!($func_name))?;
                        if f.alternate() {
                            write!(f, "(")?;
                            let mut first = true;
                            $(
                                if !first {
                                    write!(f, ", ")?;
                                }
                                write!(f, "{}", stringify!($param_name))?;
                                first = false;
                            )*
                            write!(f, ")")?;
                        }
                        Ok(())
                    }),*
                }
            }
        }
        impl SqlPageFunctionName {
            pub(super) async fn evaluate<'a>(
                &self,
                request: &'a RequestInfo,
                params: Vec<Option<Cow<'a, str>>>
            ) -> anyhow::Result<Option<Cow<'a, str>>> {
                match self {
                    $(
                        SqlPageFunctionName::$func_name => {
                            let mut iter_params = params.into_iter();
                            $(
                                let $param_name = <$param_type>::from_param_iter(&mut iter_params)
                                    .map_err(|e| anyhow!("Parameter {}: {e}", stringify!($param_name)))?;
                            )*
                            if let Some(extraneous_param) = iter_params.next() {
                                anyhow::bail!("Too many arguments. Remove extra argument {}", as_sql(extraneous_param));
                            }
                            $func_name(request, $($param_name),*).await
                        }
                    )*
                }
            }
        }
    }
}

fn as_sql<'a>(param: Option<Cow<'a, str>>) -> String {
    param
        .map(|x| format!("'{}'", x.replace('\'', "''")))
        .unwrap_or_else(|| "NULL".into())
}

sqlpage_functions! {
    hash_password(password: String);
}

trait FunctionParamType<'a>: Sized {
    fn from_param_iter(arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>) -> anyhow::Result<Self>;
}

impl<'a, T: FromStr + Sized + 'a> FunctionParamType<'a> for T
where
    <T as FromStr>::Err: Sync + Send + std::error::Error + 'static,
{
    fn from_param_iter(arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>) -> anyhow::Result<Self> {
        let param = arg
            .next()
            .ok_or_else(|| anyhow!("Missing"))?
            .ok_or_else(|| anyhow!("Unexpected NULL value"))?;
        let param = param.as_ref();
        param
            .parse()
            .with_context(|| format!("Unable to parse {param:?}"))
    }
}
