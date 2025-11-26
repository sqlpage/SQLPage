use super::{ExecutionContext, RequestInfo};
use crate::webserver::{
    database::{
        blob_to_data_url::vec_to_data_uri_with_mime, execute_queries::DbConn,
        sqlpage_functions::url_parameters::URLParameters,
    },
    http_client::make_http_client,
    request_variables::SetVariablesMap,
    single_or_vec::SingleOrVec,
    ErrorWithStatus,
};
use anyhow::{anyhow, Context};
use futures_util::StreamExt;
use mime_guess::mime;
use std::fmt::Write;
use std::{borrow::Cow, ffi::OsStr, str::FromStr};

super::function_definition_macro::sqlpage_functions! {
    basic_auth_password((&RequestInfo));
    basic_auth_username((&RequestInfo));

    client_ip((&RequestInfo));
    cookie((&RequestInfo), name: Cow<str>);
    current_working_directory();

    environment_variable(name: Cow<str>);
    exec((&RequestInfo), program_name: Cow<str>, args: Vec<Cow<str>>);

    fetch((&RequestInfo), http_request: SqlPageFunctionParam<super::http_fetch_request::HttpFetchRequest<'_>>);
    fetch_with_meta((&RequestInfo), http_request: SqlPageFunctionParam<super::http_fetch_request::HttpFetchRequest<'_>>);

    hash_password(password: Option<String>);
    header((&RequestInfo), name: Cow<str>);
    headers((&RequestInfo));
    hmac(data: Cow<str>, key: Cow<str>, algorithm: Option<Cow<str>>);

    user_info_token((&RequestInfo));
    link(file: Cow<str>, parameters: Option<Cow<str>>, hash: Option<Cow<str>>);

    path((&RequestInfo));
    persist_uploaded_file((&RequestInfo), field_name: Cow<str>, folder: Option<Cow<str>>, allowed_extensions: Option<Cow<str>>);
    protocol((&RequestInfo));

    random_string(string_length: SqlPageFunctionParam<usize>);
    read_file_as_data_url((&RequestInfo), file_path: Option<Cow<str>>);
    read_file_as_text((&RequestInfo), file_path: Option<Cow<str>>);
    request_method((&RequestInfo));
    run_sql((&ExecutionContext, &mut DbConn), sql_file_path: Option<Cow<str>>, variables: Option<Cow<str>>);
    set_variable((&ExecutionContext), name: Cow<str>, value: Option<Cow<str>>);

    uploaded_file_mime_type((&RequestInfo), upload_name: Cow<str>);
    uploaded_file_path((&RequestInfo), upload_name: Cow<str>);
    uploaded_file_name((&RequestInfo), upload_name: Cow<str>);
    url_encode(raw_text: Option<Cow<str>>);
    user_info((&RequestInfo), claim: Cow<str>);

    variables((&ExecutionContext), get_or_post: Option<Cow<str>>);
    version();
    request_body((&RequestInfo));
    request_body_base64((&RequestInfo));
}

/// Returns the password from the HTTP basic auth header, if present.
async fn basic_auth_password(request: &RequestInfo) -> anyhow::Result<&str> {
    let password = extract_basic_auth(request)?.password().ok_or_else(|| {
        anyhow::Error::new(ErrorWithStatus {
            status: actix_web::http::StatusCode::UNAUTHORIZED,
        })
    })?;
    Ok(password)
}

/// Returns the username from the HTTP basic auth header, if present.
/// Otherwise, returns an HTTP 401 Unauthorized error.
async fn basic_auth_username(request: &RequestInfo) -> anyhow::Result<&str> {
    Ok(extract_basic_auth(request)?.user_id())
}

fn extract_basic_auth(
    request: &RequestInfo,
) -> anyhow::Result<&actix_web_httpauth::headers::authorization::Basic> {
    request
        .basic_auth
        .as_ref()
        .ok_or_else(|| {
            anyhow::Error::new(ErrorWithStatus {
                status: actix_web::http::StatusCode::UNAUTHORIZED,
            })
        })
        .with_context(|| "Expected the user to be authenticated with HTTP basic auth")
}

async fn cookie<'a>(request: &'a RequestInfo, name: Cow<'a, str>) -> Option<Cow<'a, str>> {
    request.cookies.get(&*name).map(SingleOrVec::as_json_str)
}

async fn current_working_directory() -> anyhow::Result<String> {
    std::env::current_dir()
        .with_context(|| "unable to access the current working directory")
        .map(|x| x.to_string_lossy().into_owned())
}

/// Returns the value of an environment variable.
async fn environment_variable(name: Cow<'_, str>) -> anyhow::Result<Option<Cow<'_, str>>> {
    match std::env::var(&*name) {
        Ok(value) => Ok(Some(Cow::Owned(value))),
        Err(std::env::VarError::NotPresent) if name.contains(['=', '\0']) => anyhow::bail!("Invalid environment variable name: {name:?}. Environment variable names cannot contain an equals sign or a null character."),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(err).with_context(|| format!("unable to read the environment variable {name:?}"))
    }
}

/// Executes an external command and returns its output.
async fn exec<'a>(
    request: &'a RequestInfo,
    program_name: Cow<'a, str>,
    args: Vec<Cow<'a, str>>,
) -> anyhow::Result<String> {
    if !request.app_state.config.allow_exec {
        anyhow::bail!("The sqlpage.exec() function is disabled in the configuration, for security reasons.
        Make sure you understand the security implications before enabling it, and never allow user input to be passed as the first argument to this function.
        You can enable it by setting the allow_exec option to true in the sqlpage.json configuration file.")
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
        anyhow::bail!(
            "Command '{program_name}' failed with exit code {}: {}",
            res.status,
            String::from_utf8_lossy(&res.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&res.stdout).into_owned())
}

fn build_request<'a>(
    client: &'a awc::Client,
    http_request: &'a super::http_fetch_request::HttpFetchRequest<'_>,
) -> anyhow::Result<awc::ClientRequest> {
    use awc::http::Method;
    let method = if let Some(method) = &http_request.method {
        Method::from_str(method).with_context(|| format!("Invalid HTTP method: {method}"))?
    } else {
        Method::GET
    };
    let mut req = client.request(method, http_request.url.as_ref());
    if let Some(timeout) = http_request.timeout_ms {
        req = req.timeout(core::time::Duration::from_millis(timeout));
    }
    for (k, v) in &http_request.headers {
        req = req.insert_header((k.as_ref(), v.as_ref()));
    }
    if let Some(username) = &http_request.username {
        let password = http_request.password.as_deref().unwrap_or_default();
        req = req.basic_auth(username, password);
    }
    Ok(req)
}

fn prepare_request_body(
    body: &serde_json::value::RawValue,
    mut req: awc::ClientRequest,
) -> anyhow::Result<(String, awc::ClientRequest)> {
    let val = body.get();
    let body_str = if val.starts_with('"') {
        serde_json::from_str::<'_, String>(val).with_context(|| {
            format!("Invalid JSON string in the body of the HTTP request: {val}")
        })?
    } else {
        req = req.content_type("application/json");
        val.to_owned()
    };
    Ok((body_str, req))
}

async fn fetch(
    request: &RequestInfo,
    http_request: super::http_fetch_request::HttpFetchRequest<'_>,
) -> anyhow::Result<String> {
    let client = make_http_client(&request.app_state.config)
        .with_context(|| "Unable to create an HTTP client")?;
    let req = build_request(&client, &http_request)?;

    log::info!("Fetching {}", http_request.url);
    let mut response = if let Some(body) = &http_request.body {
        let (body, req) = prepare_request_body(body, req)?;
        req.send_body(body)
    } else {
        req.send()
    }
    .await
    .map_err(|e| anyhow!("Unable to fetch {}: {e}", http_request.url))?;

    log::debug!(
        "Finished fetching {}. Status: {}",
        http_request.url,
        response.status()
    );

    let body = response
        .body()
        .await
        .with_context(|| {
            format!(
                "Unable to read the body of the response from {}",
                http_request.url
            )
        })?
        .to_vec();
    let response_str = decode_response(body, http_request.response_encoding.as_deref())?;
    log::debug!("Fetch response: {response_str}");
    Ok(response_str)
}

fn decode_response(response: Vec<u8>, encoding: Option<&str>) -> anyhow::Result<String> {
    match encoding {
        Some("base64") => Ok(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            response,
        )),
        Some("base64url") => Ok(base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE,
            response,
        )),
        Some("hex") => Ok(response.into_iter().fold(String::new(), |mut acc, byte| {
            write!(&mut acc, "{byte:02x}").unwrap();
            acc
        })),
        Some(encoding_label) => Ok(encoding_rs::Encoding::for_label(encoding_label.as_bytes())
            .with_context(|| format!("Invalid encoding name: {encoding_label}"))?
            .decode(&response)
            .0
            .into_owned()),
        None => {
            let body_str = String::from_utf8(response);
            match body_str {
                Ok(body_str) => Ok(body_str),
                Err(decoding_error) => {
                    log::warn!("fetch(...) response is not UTF-8 and no encoding was specified. Decoding the response as base64. Please explicitly set the encoding to \"base64\" if this is the expected behavior.");
                    Ok(base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        decoding_error.into_bytes(),
                    ))
                }
            }
        }
    }
}

async fn fetch_with_meta(
    request: &RequestInfo,
    http_request: super::http_fetch_request::HttpFetchRequest<'_>,
) -> anyhow::Result<String> {
    use serde::{ser::SerializeMap, Serializer};

    let client = make_http_client(&request.app_state.config)
        .with_context(|| "Unable to create an HTTP client")?;
    let req = build_request(&client, &http_request)?;

    log::info!("Fetching {} with metadata", http_request.url);
    let response_result = if let Some(body) = &http_request.body {
        let (body, req) = prepare_request_body(body, req)?;
        req.send_body(body).await
    } else {
        req.send().await
    };

    let mut resp_str = Vec::new();
    let mut encoder = serde_json::Serializer::new(&mut resp_str);
    let mut obj = encoder.serialize_map(Some(3))?;
    match response_result {
        Ok(mut response) => {
            let status = response.status();
            obj.serialize_entry("status", &status.as_u16())?;
            let mut has_error = false;
            if status.is_server_error() {
                has_error = true;
                obj.serialize_entry("error", &format!("Server error: {status}"))?;
            }

            let headers = response.headers();

            let is_json = headers
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .starts_with("application/json");

            obj.serialize_entry(
                "headers",
                &headers
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default()))
                    .collect::<std::collections::HashMap<_, _>>(),
            )?;

            match response.body().await {
                Ok(body) => {
                    let body_bytes = body.to_vec();
                    let body_str =
                        decode_response(body_bytes, http_request.response_encoding.as_deref())?;
                    if is_json {
                        obj.serialize_entry(
                            "json_body",
                            &serde_json::value::RawValue::from_string(body_str)?,
                        )?;
                    } else {
                        obj.serialize_entry("body", &body_str)?;
                    }
                }
                Err(e) => {
                    log::warn!("Failed to read response body: {e}");
                    if !has_error {
                        obj.serialize_entry(
                            "error",
                            &format!("Failed to read response body: {e}"),
                        )?;
                    }
                }
            }
        }
        Err(e) => {
            log::warn!("Request failed: {e}");
            obj.serialize_entry("error", &format!("Request failed: {e}"))?;
        }
    }

    obj.end()?;
    let return_value = String::from_utf8(resp_str)?;
    Ok(return_value)
}

pub(crate) async fn hash_password(password: Option<String>) -> anyhow::Result<Option<String>> {
    let Some(password) = password else {
        return Ok(None);
    };
    actix_web::rt::task::spawn_blocking(move || {
        // Hashes a password using Argon2. This is a CPU-intensive blocking operation.
        let phf = argon2::Argon2::default();
        let salt = password_hash::SaltString::generate(&mut password_hash::rand_core::OsRng);
        let password_hash = &password_hash::PasswordHash::generate(phf, password, &salt)
            .map_err(|e| anyhow!("Unable to hash password: {e}"))?;
        Ok(password_hash.to_string())
    })
    .await?
    .map(Some)
}

async fn header<'a>(request: &'a RequestInfo, name: Cow<'a, str>) -> Option<Cow<'a, str>> {
    let lower_name = name.to_ascii_lowercase();
    request
        .headers
        .get(&lower_name)
        .map(SingleOrVec::as_json_str)
}

/// Builds a URL from a file name and a JSON object conatining URL parameters.
/// For instance, if the file is "index.sql" and the parameters are {"x": "hello world"},
/// the result will be "index.sql?x=hello%20world".
async fn link<'a>(
    file: Cow<'a, str>,
    parameters: Option<Cow<'a, str>>,
    hash: Option<Cow<'a, str>>,
) -> anyhow::Result<String> {
    let mut url = file.into_owned();
    if let Some(parameters) = parameters {
        let encoded = serde_json::from_str::<URLParameters>(&parameters).with_context(|| {
            format!("link: invalid URL parameters: not a valid json object:\n{parameters}")
        })?;
        encoded.append_to_path(&mut url);
    }
    if let Some(hash) = hash {
        url.push('#');
        url.push_str(&hash);
    }
    Ok(url)
}

/// Returns the path component of the URL of the current request.
async fn path(request: &RequestInfo) -> &str {
    &request.path
}

const DEFAULT_ALLOWED_EXTENSIONS: &str =
    "jpg,jpeg,png,gif,bmp,webp,pdf,txt,doc,docx,xls,xlsx,csv,mp3,mp4,wav,avi,mov";

async fn persist_uploaded_file<'a>(
    request: &'a RequestInfo,
    field_name: Cow<'a, str>,
    folder: Option<Cow<'a, str>>,
    allowed_extensions: Option<Cow<'a, str>>,
) -> anyhow::Result<Option<String>> {
    let folder = folder.unwrap_or(Cow::Borrowed("uploads"));
    let allowed_extensions_str =
        allowed_extensions.unwrap_or(Cow::Borrowed(DEFAULT_ALLOWED_EXTENSIONS));
    let allowed_extensions = allowed_extensions_str.split(',');
    let Some(uploaded_file) = request.uploaded_files.get(&field_name.to_string()) else {
        return Ok(None);
    };
    let file_name = uploaded_file.file_name.as_deref().unwrap_or_default();
    let extension = file_name.split('.').next_back().unwrap_or_default();
    if !allowed_extensions
        .clone()
        .any(|x| x.eq_ignore_ascii_case(extension))
    {
        let exts = allowed_extensions.collect::<Vec<_>>().join(", ");
        anyhow::bail!("file extension {extension} is not allowed. Allowed extensions: {exts}");
    }
    // resolve the folder path relative to the web root
    let web_root = &request.app_state.config.web_root;
    let target_folder = web_root.join(&*folder);
    // create the folder if it doesn't exist
    tokio::fs::create_dir_all(&target_folder)
        .await
        .with_context(|| format!("unable to create folder {}", target_folder.display()))?;
    let date = chrono::Utc::now().format("%Y-%m-%d_%Hh%Mm%Ss");
    let random_part = random_string_sync(8);
    let random_target_name = format!("{date}_{random_part}.{extension}");
    let target_path = target_folder.join(&random_target_name);
    tokio::fs::copy(&uploaded_file.file.path(), &target_path)
        .await
        .with_context(|| {
            format!(
                "unable to copy uploaded file {field_name:?} to \"{}\"",
                target_path.display()
            )
        })?;
    // remove the WEB_ROOT prefix from the path, but keep the leading slash
    let path = "/".to_string()
        + target_path
            .strip_prefix(web_root)?
            .to_str()
            .with_context(|| {
                format!(
                    "unable to convert path \"{}\" to a string",
                    target_path.display()
                )
            })?;
    Ok(Some(path))
}

/// Returns the protocol of the current request (http or https).
async fn protocol(request: &RequestInfo) -> &str {
    &request.protocol
}

/// Returns a random string of the specified length.
pub(crate) async fn random_string(len: usize) -> anyhow::Result<String> {
    // OsRng can block on Linux, so we run this on a blocking thread.
    Ok(tokio::task::spawn_blocking(move || random_string_sync(len)).await?)
}

/// Returns a random string of the specified length.
pub(crate) fn random_string_sync(len: usize) -> String {
    use rand::{distr::Alphanumeric, Rng};
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

#[tokio::test]
async fn test_random_string() {
    let s = random_string(10).await.unwrap();
    assert_eq!(s.len(), 10);
}

async fn read_file_bytes(request: &RequestInfo, path_str: &str) -> Result<Vec<u8>, anyhow::Error> {
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
            .with_context(|| format!("Unable to read file \"{}\"", path.display()))
    }
}

async fn read_file_as_data_url<'a>(
    request: &'a RequestInfo,
    file_path: Option<Cow<'a, str>>,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    let Some(file_path) = file_path else {
        log::debug!("read_file: first argument is NULL, returning NULL");
        return Ok(None);
    };
    let bytes = read_file_bytes(request, &file_path).await?;
    let mime = mime_from_upload_path(request, &file_path).map_or_else(
        || Cow::Owned(mime_guess_from_filename(&file_path)),
        Cow::Borrowed,
    );
    let data_url = vec_to_data_uri_with_mime(&bytes, &mime.to_string());
    Ok(Some(Cow::Owned(data_url)))
}

/// Returns the contents of a file as a string
async fn read_file_as_text<'a>(
    request: &'a RequestInfo,
    file_path: Option<Cow<'a, str>>,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    let Some(file_path) = file_path else {
        log::debug!("read_file: first argument is NULL, returning NULL");
        return Ok(None);
    };
    let bytes = read_file_bytes(request, &file_path).await?;
    let as_str = String::from_utf8(bytes).with_context(|| {
        format!("read_file_as_text: {file_path} does not contain raw UTF8 text")
    })?;
    Ok(Some(Cow::Owned(as_str)))
}

fn mime_from_upload_path<'a>(request: &'a RequestInfo, path: &str) -> Option<&'a mime_guess::Mime> {
    request.uploaded_files.values().find_map(|uploaded_file| {
        if uploaded_file.file.path() == OsStr::new(path) {
            uploaded_file.content_type.as_ref()
        } else {
            None
        }
    })
}

fn mime_guess_from_filename(filename: &str) -> mime_guess::Mime {
    let maybe_mime = mime_guess::from_path(filename).first();
    maybe_mime.unwrap_or(mime::APPLICATION_OCTET_STREAM)
}

async fn request_method(request: &RequestInfo) -> String {
    request.method.to_string()
}

async fn run_sql<'a>(
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
    let app_state = &request.app_state;
    let sql_file = app_state
        .sql_file_cache
        .get_with_privilege(
            app_state,
            std::path::Path::new(sql_file_path.as_ref()),
            true,
        )
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
        anyhow::bail!("Too many nested inclusions. run_sql can include a file that includes another file, but the depth is limited to {max_recursion_depth} levels. \n\
        Executing sqlpage.run_sql('{sql_file_path}') would exceed this limit. \n\
        This is to prevent infinite loops and stack overflows.\n\
        Make sure that your SQL file does not try to run itself, directly or through a chain of other files.\n\
        If you need to include more files, you can increase max_recursion_depth in the configuration file.\
        ");
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
    while let Some(db_item) = results_stream.next().await {
        use crate::webserver::database::DbItem::{Error, FinishedQuery, Row};
        match db_item {
            Row(row) => {
                log::debug!("run_sql: row: {row:?}");
                seq.serialize_element(&row)?;
            }
            FinishedQuery => log::trace!("run_sql: Finished query"),
            Error(err) => {
                return Err(err.context(format!("run_sql: unable to run {sql_file_path:?}")))
            }
        }
    }
    seq.end()?;
    Ok(Some(Cow::Owned(String::from_utf8(json_results_bytes)?)))
}

async fn set_variable<'a>(
    context: &'a ExecutionContext,
    name: Cow<'a, str>,
    value: Option<Cow<'a, str>>,
) -> anyhow::Result<String> {
    let mut params = URLParameters::new();

    for (k, v) in &context.url_params {
        if k == &name {
            continue;
        }
        params.push_single_or_vec(k, v.clone());
    }

    if let Some(value) = value {
        params.push_single_or_vec(&name, SingleOrVec::Single(value.into_owned()));
    }

    Ok(params.with_empty_path())
}

#[tokio::test]
async fn test_hash_password() {
    let s = hash_password(Some("password".to_string()))
        .await
        .unwrap()
        .unwrap();
    assert!(s.starts_with("$argon2"));
}

async fn uploaded_file_mime_type<'a>(
    request: &'a RequestInfo,
    upload_name: Cow<'a, str>,
) -> Option<Cow<'a, str>> {
    let mime = request
        .uploaded_files
        .get(&*upload_name)?
        .content_type
        .as_ref()?;
    Some(Cow::Borrowed(mime.as_ref()))
}

async fn uploaded_file_path<'a>(
    request: &'a RequestInfo,
    upload_name: Cow<'a, str>,
) -> Option<Cow<'a, str>> {
    let uploaded_file = request.uploaded_files.get(&*upload_name)?;
    Some(uploaded_file.file.path().to_string_lossy())
}

async fn uploaded_file_name<'a>(
    request: &'a RequestInfo,
    upload_name: Cow<'a, str>,
) -> Option<Cow<'a, str>> {
    let fname = request
        .uploaded_files
        .get(&*upload_name)?
        .file_name
        .as_ref()?;
    Some(Cow::Borrowed(fname.as_str()))
}

/// escapes a string for use in a URL using percent encoding
/// for example, spaces are replaced with %20, '/' with %2F, etc.
/// This is useful for constructing URLs in SQL queries.
/// If this function is passed a NULL value, it will return NULL (None in Rust),
/// rather than an empty string or an error.
async fn url_encode(raw_text: Option<Cow<'_, str>>) -> Option<Cow<'_, str>> {
    Some(match raw_text? {
        Cow::Borrowed(inner) => {
            let encoded = percent_encoding::percent_encode(
                inner.as_bytes(),
                percent_encoding::NON_ALPHANUMERIC,
            );
            encoded.into()
        }
        Cow::Owned(inner) => {
            let encoded = percent_encoding::percent_encode(
                inner.as_bytes(),
                percent_encoding::NON_ALPHANUMERIC,
            );
            Cow::Owned(encoded.collect())
        }
    })
}

/// Returns all variables in the request as a JSON object.
async fn variables<'a>(
    request: &'a ExecutionContext,
    get_or_post: Option<Cow<'a, str>>,
) -> anyhow::Result<String> {
    Ok(if let Some(get_or_post) = get_or_post {
        if get_or_post.eq_ignore_ascii_case("get") {
            serde_json::to_string(&request.url_params)?
        } else if get_or_post.eq_ignore_ascii_case("post") {
            serde_json::to_string(&request.post_variables)?
        } else if get_or_post.eq_ignore_ascii_case("set") {
            serde_json::to_string(&*request.set_variables.borrow())?
        } else {
            return Err(anyhow!(
                "Expected 'get', 'post', or 'set' as the argument to sqlpage.variables"
            ));
        }
    } else {
        use serde::{ser::SerializeMap, Serializer};
        let mut res = Vec::new();
        let mut serializer = serde_json::Serializer::new(&mut res);
        let set_vars = request.set_variables.borrow();
        let len = request.url_params.len() + request.post_variables.len() + set_vars.len();
        let mut ser = serializer.serialize_map(Some(len))?;
        for (k, v) in &request.url_params {
            ser.serialize_entry(k, v)?;
        }
        for (k, v) in &request.post_variables {
            ser.serialize_entry(k, v)?;
        }
        for (k, v) in &*set_vars {
            ser.serialize_entry(k, v)?;
        }
        ser.end()?;
        String::from_utf8(res)?
    })
}

/// Returns the version of the sqlpage that is running.
async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns the raw request body as a string.
/// If the request body is not valid UTF-8, invalid characters are replaced with the Unicode replacement character.
/// Returns NULL if there is no request body or if the request content type is
/// application/x-www-form-urlencoded or multipart/form-data (in this case, the body is accessible via the `post_variables` field).
async fn request_body(request: &RequestInfo) -> Option<String> {
    let raw_body = request.raw_body.as_ref()?;
    Some(String::from_utf8_lossy(raw_body).to_string())
}

/// Returns the raw request body encoded in base64.
/// Returns NULL if there is no request body or if the request content type is
/// application/x-www-form-urlencoded or multipart/form-data (in this case, the body is accessible via the `post_variables` field).
async fn request_body_base64(request: &RequestInfo) -> Option<String> {
    let raw_body = request.raw_body.as_ref()?;
    let mut base64_string = String::with_capacity((raw_body.len() * 4).div_ceil(3));
    base64::Engine::encode_string(
        &base64::engine::general_purpose::STANDARD,
        raw_body,
        &mut base64_string,
    );
    Some(base64_string)
}

async fn headers(request: &RequestInfo) -> String {
    serde_json::to_string(&request.headers).unwrap_or_default()
}

/// Computes the HMAC (Hash-based Message Authentication Code) of the input data
/// using the specified key and hashing algorithm.
async fn hmac<'a>(
    data: Cow<'a, str>,
    key: Cow<'a, str>,
    algorithm: Option<Cow<'a, str>>,
) -> anyhow::Result<Option<String>> {
    use hmac::{Hmac, Mac};
    use sha2::{Sha256, Sha512};

    let algorithm = algorithm.as_deref().unwrap_or("sha256");

    // Parse algorithm and output format (e.g., "sha256" or "sha256-base64")
    let (hash_algo, output_format) = if let Some((algo, format)) = algorithm.split_once('-') {
        (algo, format)
    } else {
        (algorithm, "hex")
    };

    let result = match hash_algo.to_lowercase().as_str() {
        "sha256" => {
            let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())
                .map_err(|e| anyhow!("Invalid HMAC key: {e}"))?;
            mac.update(data.as_bytes());
            mac.finalize().into_bytes().to_vec()
        }
        "sha512" => {
            let mut mac = Hmac::<Sha512>::new_from_slice(key.as_bytes())
                .map_err(|e| anyhow!("Invalid HMAC key: {e}"))?;
            mac.update(data.as_bytes());
            mac.finalize().into_bytes().to_vec()
        }
        _ => {
            anyhow::bail!(
                "Unsupported HMAC algorithm: {hash_algo}. Supported algorithms: sha256, sha512"
            )
        }
    };

    // Convert to requested output format
    let output = match output_format.to_lowercase().as_str() {
        "hex" => result.into_iter().fold(String::new(), |mut acc, byte| {
            write!(&mut acc, "{byte:02x}").unwrap();
            acc
        }),
        "base64" => base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result),
        _ => {
            anyhow::bail!(
                "Unsupported output format: {output_format}. Supported formats: hex, base64"
            )
        }
    };

    Ok(Some(output))
}

async fn client_ip(request: &RequestInfo) -> Option<String> {
    Some(request.client_ip?.to_string())
}

#[tokio::test]
async fn test_hmac() {
    // Test vector from RFC 4231 - HMAC-SHA256
    let result = hmac(
        Cow::Borrowed("The quick brown fox jumps over the lazy dog"),
        Cow::Borrowed("key"),
        Some(Cow::Borrowed("sha256")),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        result,
        "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
    );
}

/// Returns the ID token claims as a JSON object.
async fn user_info_token(request: &RequestInfo) -> anyhow::Result<Option<String>> {
    let Some(claims) = &request.oidc_claims else {
        return Ok(None);
    };
    Ok(Some(serde_json::to_string(claims)?))
}

/// Returns a specific claim from the ID token.
async fn user_info<'a>(
    request: &'a RequestInfo,
    claim: Cow<'a, str>,
) -> anyhow::Result<Option<String>> {
    let Some(claims) = &request.oidc_claims else {
        return Ok(None);
    };

    // Match against known OIDC claims accessible via direct methods.
    let claim_value_str = match claim.as_ref() {
        // Core Claims
        "iss" => Some(claims.issuer().to_string()),
        // aud requires serialization: handled separately if needed
        "exp" => Some(claims.expiration().timestamp().to_string()),
        "iat" => Some(claims.issue_time().timestamp().to_string()),
        "sub" => Some(claims.subject().to_string()),
        "auth_time" => claims.auth_time().map(|t| t.timestamp().to_string()),
        "nonce" => claims.nonce().map(|n| n.secret().clone()), // Assuming Nonce has secret()
        "acr" => claims.auth_context_ref().map(|acr| acr.to_string()),
        // amr requires serialization: handled separately if needed
        "azp" => claims.authorized_party().map(|azp| azp.to_string()),
        "at_hash" => claims.access_token_hash().map(|h| h.to_string()),
        "c_hash" => claims.code_hash().map(|h| h.to_string()),

        // Standard Claims (Profile Scope - subset)
        "name" => claims
            .name()
            .and_then(|n| n.get(None))
            .map(|s| s.to_string()),
        "given_name" => claims
            .given_name()
            .and_then(|n| n.get(None))
            .map(|s| s.to_string()),
        "family_name" => claims
            .family_name()
            .and_then(|n| n.get(None))
            .map(|s| s.to_string()),
        "middle_name" => claims
            .middle_name()
            .and_then(|n| n.get(None))
            .map(|s| s.to_string()),
        "nickname" => claims
            .nickname()
            .and_then(|n| n.get(None))
            .map(|s| s.to_string()),
        "preferred_username" => claims.preferred_username().map(|u| u.to_string()),
        "profile" => claims
            .profile()
            .and_then(|n| n.get(None))
            .map(|url_claim| url_claim.as_str().to_string()),
        "picture" => claims
            .picture()
            .and_then(|n| n.get(None))
            .map(|url_claim| url_claim.as_str().to_string()),
        "website" => claims
            .website()
            .and_then(|n| n.get(None))
            .map(|url_claim| url_claim.as_str().to_string()),
        "gender" => claims.gender().map(|g| g.to_string()), // Assumes GenderClaim impls ToString
        "birthdate" => claims.birthdate().map(|b| b.to_string()), // Assumes Birthdate impls ToString
        "zoneinfo" => claims.zoneinfo().map(|z| z.to_string()),   // Assumes ZoneInfo impls ToString
        "locale" => claims.locale().map(std::string::ToString::to_string), // Assumes Locale impls ToString
        "updated_at" => claims.updated_at().map(|t| t.timestamp().to_string()),

        // Standard Claims (Email Scope)
        "email" => claims.email().map(|e| e.to_string()),
        "email_verified" => claims.email_verified().map(|b| b.to_string()),

        // Standard Claims (Phone Scope)
        "phone_number" => claims.phone_number().map(|p| p.to_string()),
        "phone_number_verified" => claims.phone_number_verified().map(|b| b.to_string()),
        additional_claim => claims
            .additional_claims()
            .0
            .get(additional_claim)
            .map(std::string::ToString::to_string),
    };

    Ok(claim_value_str)
}
