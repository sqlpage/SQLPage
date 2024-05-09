use super::RequestInfo;
use crate::webserver::{http::SingleOrVec, ErrorWithStatus};
use anyhow::{anyhow, Context};
use std::{borrow::Cow, ffi::OsStr};

super::function_definition_macro::sqlpage_functions! {
    cookie((&RequestInfo), name: Cow<str>);
    header((&RequestInfo), name: Cow<str>);
    random_string(string_length: SqlPageFunctionParam<usize>);
    hash_password(password: String);
    basic_auth_username((&RequestInfo));
    basic_auth_password((&RequestInfo));
    variables((&RequestInfo), get_or_post: Option<Cow<str>>);
    url_encode(raw_text: Option<Cow<str>>);
    exec((&RequestInfo), program_name: Cow<str>, args: Vec<Cow<str>>);
    current_working_directory();
    environment_variable(name: Cow<str>);
    version();
    read_file_as_text((&RequestInfo), file_path: Option<Cow<str>>);
    read_file_as_data_url((&RequestInfo), file_path: Option<Cow<str>>);
    uploaded_file_mime_type((&RequestInfo), upload_name: Cow<str>);
    uploaded_file_path((&RequestInfo), upload_name: Cow<str>);
    path((&RequestInfo));
    protocol((&RequestInfo));
    persist_uploaded_file((&RequestInfo), field_name: Cow<str>, folder: Option<Cow<str>>, allowed_extensions: Option<Cow<str>>);
}

async fn cookie<'a>(request: &'a RequestInfo, name: Cow<'a, str>) -> Option<Cow<'a, str>> {
    request.cookies.get(&*name).map(SingleOrVec::as_json_str)
}

async fn header<'a>(request: &'a RequestInfo, name: Cow<'a, str>) -> Option<Cow<'a, str>> {
    request.headers.get(&*name).map(SingleOrVec::as_json_str)
}

/// Returns a random string of the specified length.
pub(crate) async fn random_string(len: usize) -> anyhow::Result<String> {
    // OsRng can block on Linux, so we run this on a blocking thread.
    Ok(tokio::task::spawn_blocking(move || random_string_sync(len)).await?)
}

#[tokio::test]
async fn test_random_string() {
    let s = random_string(10).await.unwrap();
    assert_eq!(s.len(), 10);
}

/// Returns a random string of the specified length.
pub(crate) fn random_string_sync(len: usize) -> String {
    use rand::{distributions::Alphanumeric, Rng};
    password_hash::rand_core::OsRng
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub(crate) async fn hash_password(password: String) -> anyhow::Result<String> {
    actix_web::rt::task::spawn_blocking(move || {
        // Hashes a password using Argon2. This is a CPU-intensive blocking operation.
        let phf = argon2::Argon2::default();
        let salt = password_hash::SaltString::generate(&mut password_hash::rand_core::OsRng);
        let password_hash = &password_hash::PasswordHash::generate(phf, password, &salt)
            .map_err(|e| anyhow!("Unable to hash password: {}", e))?;
        Ok(password_hash.to_string())
    })
    .await?
}

#[tokio::test]
async fn test_hash_password() {
    let s = hash_password("password".to_string()).await.unwrap();
    assert!(s.starts_with("$argon2"));
}

/// Returns the username from the HTTP basic auth header, if present.
/// Otherwise, returns an HTTP 401 Unauthorized error.
async fn basic_auth_username(request: &RequestInfo) -> anyhow::Result<&str> {
    Ok(extract_basic_auth(request)?.user_id())
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

/// Returns all variables in the request as a JSON object.
async fn variables<'a>(
    request: &'a RequestInfo,
    get_or_post: Option<Cow<'a, str>>,
) -> anyhow::Result<String> {
    Ok(if let Some(get_or_post) = get_or_post {
        if get_or_post.eq_ignore_ascii_case("get") {
            serde_json::to_string(&request.get_variables)?
        } else if get_or_post.eq_ignore_ascii_case("post") {
            serde_json::to_string(&request.post_variables)?
        } else {
            return Err(anyhow!(
                "Expected 'get' or 'post' as the argument to sqlpage.all_variables"
            ));
        }
    } else {
        use serde::{ser::SerializeMap, Serializer};
        let mut res = Vec::new();
        let mut serializer = serde_json::Serializer::new(&mut res);
        let len = request.get_variables.len() + request.post_variables.len();
        let mut ser = serializer.serialize_map(Some(len))?;
        let iter = request.get_variables.iter().chain(&request.post_variables);
        for (k, v) in iter {
            ser.serialize_entry(k, v)?;
        }
        ser.end()?;
        String::from_utf8(res)?
    })
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

async fn current_working_directory() -> anyhow::Result<String> {
    std::env::current_dir()
        .with_context(|| "unable to access the current working directory")
        .map(|x| x.to_string_lossy().into_owned())
}

/// Returns the value of an environment variable.
async fn environment_variable(name: Cow<'_, str>) -> anyhow::Result<Cow<'_, str>> {
    std::env::var(&*name)
        .with_context(|| format!("unable to access the environment variable {name}"))
        .map(Cow::Owned)
}

/// Returns the version of the sqlpage that is running.
async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

async fn read_file_bytes<'a>(
    request: &'a RequestInfo,
    path_str: &str,
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
    maybe_mime.unwrap_or(mime_guess::mime::APPLICATION_OCTET_STREAM)
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
    let mut data_url = format!("data:{}/{};base64,", mime.type_(), mime.subtype());
    base64::Engine::encode_string(
        &base64::engine::general_purpose::STANDARD,
        bytes,
        &mut data_url,
    );
    Ok(Some(Cow::Owned(data_url)))
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

/// Returns the path component of the URL of the current request.
async fn path(request: &RequestInfo) -> &str {
    &request.path
}

/// Returns the protocol of the current request (http or https).
async fn protocol(request: &RequestInfo) -> &str {
    &request.protocol
}

const DEFAULT_ALLOWED_EXTENSIONS: &str =
    "jpg,jpeg,png,gif,bmp,webp,pdf,txt,doc,docx,xls,xlsx,csv,mp3,mp4,wav,avi,mov";

async fn persist_uploaded_file<'a>(
    request: &'a RequestInfo,
    field_name: Cow<'a, str>,
    folder: Option<Cow<'a, str>>,
    allowed_extensions: Option<Cow<'a, str>>,
) -> anyhow::Result<String> {
    let folder = folder.unwrap_or(Cow::Borrowed("uploads"));
    let allowed_extensions_str =
        allowed_extensions.unwrap_or(Cow::Borrowed(DEFAULT_ALLOWED_EXTENSIONS));
    let allowed_extensions = allowed_extensions_str.split(',');
    let uploaded_file = request
        .uploaded_files
        .get(&field_name.to_string())
        .ok_or_else(|| {
            anyhow!(
                "no file uploaded with field name {field_name}. Uploaded files: {:?}",
                request.uploaded_files.keys()
            )
        })?;
    let file_name = uploaded_file.file_name.as_deref().unwrap_or_default();
    let extension = file_name.split('.').last().unwrap_or_default();
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
        .with_context(|| format!("unable to create folder {target_folder:?}"))?;
    let date = chrono::Utc::now().format("%Y-%m-%d %Hh%Mm%Ss");
    let random_part = random_string_sync(8);
    let random_target_name = format!("{date} {random_part}.{extension}");
    let target_path = target_folder.join(&random_target_name);
    tokio::fs::copy(&uploaded_file.file.path(), &target_path)
        .await
        .with_context(|| {
            format!("unable to copy uploaded file {field_name:?} to {target_path:?}")
        })?;
    // remove the WEB_ROOT prefix from the path, but keep the leading slash
    let path = "/".to_string()
        + target_path
            .strip_prefix(web_root)?
            .to_str()
            .with_context(|| format!("unable to convert path {target_path:?} to a string"))?;
    Ok(path)
}
