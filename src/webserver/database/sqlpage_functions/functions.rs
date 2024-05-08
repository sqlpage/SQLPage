use super::RequestInfo;
use crate::webserver::{http::SingleOrVec, ErrorWithStatus};
use anyhow::{anyhow, Context as _};
use std::borrow::Cow;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_random_string() {
        let s = random_string(10).await.unwrap();
        assert_eq!(s.len(), 10);
    }

    #[tokio::test]
    async fn test_hash_password() {
        let s = hash_password("password".to_string()).await.unwrap();
        assert!(s.starts_with("$argon2"));
    }
}
