//! Built-in `SQLPage` SQL functions.
//!
//! Every function is a plain `async fn` in its own module under [`functions/`](self). To add one,
//! create `functions/<name>.rs` with an `async fn <name>` and add it to the
//! [`sqlpage_functions!`](super::function_traits::sqlpage_functions) call below. The macro declares
//! the module and adds it to the dispatch enum. Argument conversion and
//! dispatch are handled generically in [`super::function_traits`].

use std::fmt::Write;

use super::function_traits::sqlpage_functions;

sqlpage_functions! {
    basic_auth_password,
    basic_auth_username,
    client_ip,
    configuration_directory,
    cookie,
    current_working_directory,
    environment_variable,
    exec,
    fetch,
    fetch_with_meta,
    hash_password,
    header,
    headers,
    hmac,
    link,
    oidc_logout_url,
    path,
    persist_uploaded_file,
    protocol,
    random_string,
    read_file_as_data_url,
    read_file_as_text,
    regex_match,
    request_body,
    request_body_base64,
    request_method,
    run_sql,
    set_variable,
    uploaded_file_mime_type,
    uploaded_file_name,
    uploaded_file_path,
    url_encode,
    user_info,
    user_info_token,
    variables,
    version,
    web_root,
}

impl ::std::str::FromStr for SqlPageFunctionName {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> anyhow::Result<Self> {
        SqlPageFunctionName::ALL
            .iter()
            .copied()
            .find(|function| function.name() == name)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown function {name:?}. Supported functions:\n{}",
                    supported_function_list()
                )
            })
    }
}

impl ::std::fmt::Display for SqlPageFunctionName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.write_str("sqlpage.")?;
        f.write_str(self.name())
    }
}

fn supported_function_list() -> String {
    let mut supported = String::new();
    for function in SqlPageFunctionName::ALL {
        writeln!(supported, "  - {function}").expect("writing to a String cannot fail");
    }
    supported
}
