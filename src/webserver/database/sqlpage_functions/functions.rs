//! Built-in `SQLPage` SQL functions.
//!
//! Every function is a plain `async fn` in its own module under [`functions/`](self). To add one,
//! create `functions/<name>.rs` with an `async fn <name>`, declare the module below and add it to
//! the [`sqlpage_functions!`](super::function_traits::sqlpage_functions) call. Argument conversion
//! and dispatch are handled generically in [`super::function_traits`].

use std::fmt::Write;

use super::function_traits::sqlpage_functions;

mod basic_auth_password;
mod basic_auth_username;
mod client_ip;
mod configuration_directory;
mod cookie;
mod current_working_directory;
mod environment_variable;
mod exec;
mod fetch;
mod fetch_with_meta;
mod hash_password;
mod header;
mod headers;
mod hmac;
mod link;
mod oidc_logout_url;
mod path;
mod persist_uploaded_file;
mod protocol;
mod random_string;
mod read_file_as_data_url;
mod read_file_as_text;
mod regex_match;
mod request_body;
mod request_body_base64;
mod request_method;
mod run_sql;
mod send_mail;
mod set_variable;
mod uploaded_file_mime_type;
mod uploaded_file_name;
mod uploaded_file_path;
mod url_encode;
mod user_info;
mod user_info_token;
mod variables;
mod version;
mod web_root;

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
    send_mail,
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
            .find(|function| function.name().eq_ignore_ascii_case(name))
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

#[cfg(test)]
mod tests {
    use super::SqlPageFunctionName;
    use std::collections::BTreeSet;

    #[test]
    fn functions_directory_matches_registered_functions() {
        let directory = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/webserver/database/sqlpage_functions/functions"
        );
        let files: BTreeSet<String> = std::fs::read_dir(directory)
            .expect("functions directory")
            .map(|entry| entry.expect("directory entry").path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
            .map(|path| {
                path.file_stem()
                    .expect("file stem")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        let registered: BTreeSet<String> = SqlPageFunctionName::ALL
            .iter()
            .map(|function| function.name().to_owned())
            .collect();
        assert_eq!(files, registered);
    }
}
