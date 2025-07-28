//! This module determines how incoming HTTP requests are mapped to
//! SQL files for execution, static assets for serving, or error pages.
//!
//! ## Routing Rules
//!
//! `SQLPage` follows a file-based routing system with the following precedence:
//!
//! ### 1. Site Prefix Handling
//! - If a `site_prefix` is configured and the request path doesn't start with it, redirect to the prefixed path
//! - All subsequent routing operates on the path after stripping the prefix
//!
//! ### 2. Path Resolution (in order of precedence)
//!
//! #### Paths ending with `/` (directories):
//! - Look for `index.sql` in that directory
//! - If found: **Execute** the SQL file
//! - If not found: Look for custom 404 handlers (see Error Handling below)
//!
//! #### Paths with `.sql` extension:
//! - If the file exists: **Execute** the SQL file  
//! - If not found: Look for custom 404 handlers (see Error Handling below)
//!
//! #### Paths with other extensions (assets):
//! - If the file exists: **Serve** the static file
//! - If not found: Look for custom 404 handlers (see Error Handling below)
//!
//! #### Paths without extension:
//! - First, try to find `{path}.sql` and **Execute** if found
//! - If no SQL file found but `{path}/index.sql` exists: **Redirect** to `{path}/`
//! - Otherwise: Look for custom 404 handlers (see Error Handling below)
//!
//! ### 3. Error Handling (404 cases)
//!
//! When a requested file is not found, `SQLPage` looks for custom 404 handlers:
//!
//! - Starting from the requested path's directory, walk up the directory tree
//! - Look for `404.sql` in each parent directory  
//! - If found: **Execute** the custom 404 SQL file
//! - If no custom 404 found anywhere: Return default **404 Not Found** response
//!
//! ## Examples
//!
//! ```text
//! Request: GET /
//! Result: Execute index.sql
//!
//! Request: GET /users
//! - If users.sql exists: Execute users.sql  
//! - Else if users/index.sql exists: Redirect to /users/
//! - Else if 404.sql exists: Execute 404.sql
//! - Else: Default 404
//!
//! Request: GET /users/
//! - If users/index.sql exists: Execute users/index.sql
//! - Else if users/404.sql exists: Execute users/404.sql  
//! - Else if 404.sql exists: Execute 404.sql
//! - Else: Default 404
//!
//! Request: GET /api/users.sql  
//! - If api/users.sql exists: Execute api/users.sql
//! - Else if api/404.sql exists: Execute api/404.sql
//! - Else if 404.sql exists: Execute 404.sql  
//! - Else: Default 404
//!
//! Request: GET /favicon.ico
//! - If favicon.ico exists: Serve favicon.ico
//! - Else if 404.sql exists: Execute 404.sql
//! - Else: Default 404
//! ```

use crate::filesystem::FileSystem;
use crate::webserver::database::ParsedSqlFile;
use crate::{file_cache::FileCache, AppState};
use awc::http::uri::PathAndQuery;
use log::debug;
use percent_encoding;
use std::path::{Path, PathBuf};
use RoutingAction::{CustomNotFound, Execute, NotFound, Redirect, Serve};

const INDEX: &str = "index.sql";
const NOT_FOUND: &str = "404.sql";
const SQL_EXTENSION: &str = "sql";
const FORWARD_SLASH: &str = "/";

#[derive(Debug, PartialEq)]
pub enum RoutingAction {
    CustomNotFound(PathBuf),
    Execute(PathBuf),
    NotFound,
    Redirect(String),
    Serve(PathBuf),
}

#[expect(async_fn_in_trait)]
pub trait FileStore {
    async fn contains(&self, path: &Path) -> anyhow::Result<bool>;
}

pub trait RoutingConfig {
    fn prefix(&self) -> &str;
}

pub(crate) struct AppFileStore<'a> {
    cache: &'a FileCache<ParsedSqlFile>,
    filesystem: &'a FileSystem,
    app_state: &'a AppState,
}

impl<'a> AppFileStore<'a> {
    pub fn new(
        cache: &'a FileCache<ParsedSqlFile>,
        filesystem: &'a FileSystem,
        app_state: &'a AppState,
    ) -> Self {
        Self {
            cache,
            filesystem,
            app_state,
        }
    }
}

impl FileStore for AppFileStore<'_> {
    async fn contains(&self, path: &Path) -> anyhow::Result<bool> {
        if self.cache.contains(path).await? {
            Ok(true)
        } else {
            self.filesystem.file_exists(self.app_state, path).await
        }
    }
}

pub async fn calculate_route<T, C>(
    path_and_query: &PathAndQuery,
    store: &T,
    config: &C,
) -> anyhow::Result<RoutingAction>
where
    T: FileStore,
    C: RoutingConfig,
{
    let result = match check_path(path_and_query, config) {
        Ok(path) => match path.extension() {
            None => calculate_route_without_extension(path_and_query, path, store).await?,
            Some(extension) => {
                let ext = extension.to_str().unwrap_or_default();
                find_file_or_not_found(&path, ext, store).await?
            }
        },
        Err(action) => action,
    };
    debug!("Route: [{path_and_query}] -> {result:?}");
    Ok(result)
}

fn check_path<C>(path_and_query: &PathAndQuery, config: &C) -> Result<PathBuf, RoutingAction>
where
    C: RoutingConfig,
{
    match path_and_query.path().strip_prefix(config.prefix()) {
        None => Err(Redirect(config.prefix().to_string())),
        Some(path) => {
            let decoded = percent_encoding::percent_decode_str(path);
            #[cfg(unix)]
            {
                use std::ffi::OsString;
                use std::os::unix::ffi::OsStringExt;

                let decoded = decoded.collect::<Vec<u8>>();
                Ok(PathBuf::from(OsString::from_vec(decoded)))
            }
            #[cfg(not(unix))]
            {
                Ok(PathBuf::from(decoded.decode_utf8_lossy().as_ref()))
            }
        }
    }
}

async fn calculate_route_without_extension<T>(
    path_and_query: &PathAndQuery,
    mut path: PathBuf,
    store: &T,
) -> anyhow::Result<RoutingAction>
where
    T: FileStore,
{
    if path_and_query.path().ends_with(FORWARD_SLASH) {
        path.push(INDEX);
        find_file_or_not_found(&path, SQL_EXTENSION, store).await
    } else {
        let path_with_ext = path.with_extension(SQL_EXTENSION);
        match find_file_or_not_found(&path_with_ext, SQL_EXTENSION, store).await? {
            Execute(x) => Ok(Execute(x)),
            other_action => {
                if store.contains(&path.join(INDEX)).await? {
                    Ok(Redirect(append_to_path(path_and_query, FORWARD_SLASH)))
                } else {
                    Ok(other_action)
                }
            }
        }
    }
}

async fn find_file_or_not_found<T>(
    path: &Path,
    extension: &str,
    store: &T,
) -> anyhow::Result<RoutingAction>
where
    T: FileStore,
{
    match find_file(path, extension, store).await? {
        None => find_not_found(path, store).await,
        Some(execute) => Ok(execute),
    }
}

async fn find_file<T>(
    path: &Path,
    extension: &str,
    store: &T,
) -> anyhow::Result<Option<RoutingAction>>
where
    T: FileStore,
{
    if store.contains(path).await? {
        Ok(Some(if extension == SQL_EXTENSION {
            Execute(path.to_path_buf())
        } else {
            Serve(path.to_path_buf())
        }))
    } else {
        Ok(None)
    }
}

async fn find_not_found<T>(path: &Path, store: &T) -> anyhow::Result<RoutingAction>
where
    T: FileStore,
{
    let mut parent = path.parent();
    while let Some(p) = parent {
        let target = p.join(NOT_FOUND);
        if store.contains(&target).await? {
            return Ok(CustomNotFound(target));
        }
        parent = p.parent();
    }

    Ok(NotFound)
}

fn append_to_path(path_and_query: &PathAndQuery, append: &str) -> String {
    let mut full_uri = path_and_query.to_string();
    full_uri.insert_str(path_and_query.path().len(), append);
    full_uri
}

#[cfg(test)]
mod tests {
    use super::RoutingAction::{CustomNotFound, Execute, NotFound, Redirect, Serve};
    use super::{calculate_route, FileStore, RoutingAction, RoutingConfig};
    use awc::http::uri::PathAndQuery;
    use std::default::Default as StdDefault;
    use std::path::{Path, PathBuf};
    use std::str::FromStr;
    use StoreConfig::{Custom, Default, Empty, File};

    mod execute {
        use super::StoreConfig::{Default, File};
        use super::{do_route, execute};

        #[tokio::test]
        async fn root_path_executes_index() {
            let actual = do_route("/", Default, None).await;
            let expected = execute("index.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn root_path_and_site_prefix_executes_index() {
            let actual = do_route("/prefix/", Default, Some("/prefix/")).await;
            let expected = execute("index.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn extension() {
            let actual = do_route("/index.sql", Default, None).await;
            let expected = execute("index.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn extension_and_site_prefix() {
            let actual = do_route("/prefix/index.sql", Default, Some("/prefix/")).await;
            let expected = execute("index.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn no_extension() {
            let actual = do_route("/path", File("path.sql"), None).await;
            let expected = execute("path.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn no_extension_and_site_prefix() {
            let actual = do_route("/prefix/path", File("path.sql"), Some("/prefix/")).await;
            let expected = execute("path.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn trailing_slash_executes_index_in_directory() {
            let actual = do_route("/folder/", File("folder/index.sql"), None).await;
            let expected = execute("folder/index.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn trailing_slash_and_site_prefix_executes_index_in_directory() {
            let actual = do_route(
                "/prefix/folder/",
                File("folder/index.sql"),
                Some("/prefix/"),
            )
            .await;
            let expected = execute("folder/index.sql");

            assert_eq!(expected, actual);
        }
    }

    mod custom_not_found {
        use super::StoreConfig::{Default, File};
        use super::{custom_not_found, do_route};

        #[tokio::test]
        async fn sql_extension() {
            let actual = do_route("/unknown.sql", Default, None).await;
            let expected = custom_not_found("404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn sql_extension_and_site_prefix() {
            let actual = do_route("/prefix/unknown.sql", Default, Some("/prefix/")).await;
            let expected = custom_not_found("404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn sql_extension_executes_deeper_not_found_file_if_exists() {
            let actual = do_route("/unknown/unknown.sql", File("unknown/404.sql"), None).await;
            let expected = custom_not_found("unknown/404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn sql_extension_and_site_prefix_executes_deeper_not_found_file_if_exists() {
            let actual = do_route(
                "/prefix/unknown/unknown.sql",
                File("unknown/404.sql"),
                Some("/prefix/"),
            )
            .await;
            let expected = custom_not_found("unknown/404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn sql_extension_executes_deepest_not_found_file_that_exists() {
            let actual = do_route(
                "/unknown/unknown/unknown.sql",
                File("unknown/404.sql"),
                None,
            )
            .await;
            let expected = custom_not_found("unknown/404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn sql_extension_and_site_prefix_executes_deepest_not_found_file_that_exists() {
            let actual = do_route(
                "/prefix/unknown/unknown/unknown.sql",
                File("unknown/404.sql"),
                Some("/prefix/"),
            )
            .await;
            let expected = custom_not_found("unknown/404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn no_extension_path_that_would_result_in_404_does_not_redirect() {
            let actual = do_route("/nonexistent", Default, None).await;
            let expected = custom_not_found("404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn no_extension_path_that_would_result_in_404_does_not_redirect_with_site_prefix() {
            let actual = do_route("/prefix/nonexistent", Default, Some("/prefix/")).await;
            let expected = custom_not_found("404.sql");

            assert_eq!(expected, actual);
        }
    }

    mod not_found {
        use super::StoreConfig::Empty;
        use super::{default_not_found, do_route};

        #[tokio::test]
        async fn default_404_when_no_not_found_file_available() {
            let actual = do_route("/unknown.sql", Empty, None).await;
            let expected = default_not_found();

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn default_404_when_no_not_found_file_available_and_site_prefix() {
            let actual = do_route("/prefix/unknown.sql", Empty, Some("/prefix/")).await;
            let expected = default_not_found();

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn asset_not_found() {
            let actual = do_route("/favicon.ico", Empty, None).await;
            let expected = default_not_found();

            assert_eq!(expected, actual);
        }
    }

    mod asset {
        use super::StoreConfig::File;
        use super::{do_route, serve};

        #[tokio::test]
        async fn serves_corresponding_asset() {
            let actual = do_route("/favicon.ico", File("favicon.ico"), None).await;
            let expected = serve("favicon.ico");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn asset_trims_query() {
            let actual = do_route("/favicon.ico?version=10", File("favicon.ico"), None).await;
            let expected = serve("favicon.ico");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn asset_trims_fragment() {
            let actual = do_route("/favicon.ico#asset1", File("favicon.ico"), None).await;
            let expected = serve("favicon.ico");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn serves_corresponding_asset_given_site_prefix() {
            let actual =
                do_route("/prefix/favicon.ico", File("favicon.ico"), Some("/prefix/")).await;
            let expected = serve("favicon.ico");

            assert_eq!(expected, actual);
        }
    }

    mod redirect {
        use super::StoreConfig::{Default, Empty};
        use super::{custom_not_found, default_not_found, do_route, redirect};

        #[tokio::test]
        async fn path_without_site_prefix_redirects_to_site_prefix() {
            let actual = do_route("/path", Default, Some("/prefix/")).await;
            let expected = redirect("/prefix/");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn no_extension_and_no_corresponding_file_with_custom_404_does_not_redirect() {
            let actual = do_route("/folder", Default, None).await;
            let expected = custom_not_found("404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn no_extension_no_corresponding_file_with_custom_404_does_not_redirect_with_query() {
            let actual = do_route("/folder?misc=1&foo=bar", Default, None).await;
            let expected = custom_not_found("404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn no_extension_site_prefix_and_no_corresponding_file_with_custom_404_does_not_redirect(
        ) {
            let actual = do_route("/prefix/folder", Default, Some("/prefix/")).await;
            let expected = custom_not_found("404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn no_extension_returns_404_when_no_404sql_available() {
            assert_eq!(do_route("/folder", Empty, None).await, default_not_found());
        }
    }

    async fn do_route(path: &str, config: StoreConfig, prefix: Option<&str>) -> RoutingAction {
        let store = match config {
            Default => Store::with_default_contents(),
            Empty => Store::empty(),
            File(file) => Store::new(file),
            Custom(files) => Store::with_files(&files),
        };
        let config = match prefix {
            None => Config::default(),
            Some(value) => Config::new(value),
        };
        calculate_route(&PathAndQuery::from_str(path).unwrap(), &store, &config)
            .await
            .unwrap()
    }

    fn default_not_found() -> RoutingAction {
        NotFound
    }

    fn execute(path: &str) -> RoutingAction {
        Execute(PathBuf::from(path))
    }

    fn custom_not_found(path: &str) -> RoutingAction {
        CustomNotFound(PathBuf::from(path))
    }

    fn redirect(uri: &str) -> RoutingAction {
        Redirect(uri.to_string())
    }

    fn serve(path: &str) -> RoutingAction {
        Serve(PathBuf::from(path))
    }

    enum StoreConfig {
        Default,
        Empty,
        File(&'static str),
        Custom(Vec<&'static str>),
    }

    struct Store {
        contents: Vec<String>,
    }

    impl Store {
        const INDEX: &'static str = "index.sql";
        const NOT_FOUND: &'static str = "404.sql";
        fn new(path: &str) -> Self {
            let mut contents = Self::default_contents();
            contents.push(path.to_string());
            Self { contents }
        }

        fn default_contents() -> Vec<String> {
            vec![Self::INDEX.to_string(), Self::NOT_FOUND.to_string()]
        }

        fn with_default_contents() -> Self {
            Self {
                contents: Self::default_contents(),
            }
        }

        fn empty() -> Self {
            Self { contents: vec![] }
        }

        fn contains(&self, path: &str) -> bool {
            let normalized_path = path.replace('\\', "/");
            dbg!(&normalized_path, &self.contents);
            self.contents.contains(&normalized_path)
        }

        fn with_files(files: &[&str]) -> Self {
            Self {
                contents: files.iter().map(|s| (*s).to_string()).collect(),
            }
        }
    }

    impl FileStore for Store {
        async fn contains(&self, path: &Path) -> anyhow::Result<bool> {
            Ok(self.contains(path.to_string_lossy().to_string().as_str()))
        }
    }

    struct Config {
        prefix: String,
    }

    impl Config {
        fn new(prefix: &str) -> Self {
            Self {
                prefix: prefix.to_string(),
            }
        }
    }
    impl RoutingConfig for Config {
        fn prefix(&self) -> &str {
            &self.prefix
        }
    }

    impl StdDefault for Config {
        fn default() -> Self {
            Self::new("/")
        }
    }

    mod specific_configuration {
        use crate::webserver::routing::tests::default_not_found;

        use super::StoreConfig::Custom;
        use super::{custom_not_found, do_route, execute, redirect, RoutingAction};

        async fn route_with_index_and_folder_404(path: &str) -> RoutingAction {
            do_route(
                path,
                Custom(vec![
                    "index.sql",
                    "folder/404.sql",
                    "folder_with_index/index.sql",
                ]),
                None,
            )
            .await
        }

        #[tokio::test]
        async fn root_path_executes_index() {
            let actual = route_with_index_and_folder_404("/").await;
            let expected = execute("index.sql");
            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn index_sql_path_executes_index() {
            let actual = route_with_index_and_folder_404("/index.sql").await;
            let expected = execute("index.sql");
            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn folder_without_trailing_slash_redirects() {
            let actual = route_with_index_and_folder_404("/folder_with_index").await;
            let expected = redirect("/folder_with_index/");
            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn folder_without_trailing_slash_without_index_does_not_redirect() {
            let actual = route_with_index_and_folder_404("/folder").await;
            let expected = default_not_found();
            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn folder_with_trailing_slash_executes_custom_404() {
            let actual = route_with_index_and_folder_404("/folder/").await;
            let expected = custom_not_found("folder/404.sql");
            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn folder_xxx_executes_custom_404() {
            let actual = route_with_index_and_folder_404("/folder/xxx").await;
            let expected = custom_not_found("folder/404.sql");
            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn folder_xxx_with_query_executes_custom_404() {
            let actual = route_with_index_and_folder_404("/folder/xxx?x=1").await;
            let expected = custom_not_found("folder/404.sql");
            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn folder_nested_path_executes_custom_404() {
            let actual = route_with_index_and_folder_404("/folder/xxx/yyy").await;
            let expected = custom_not_found("folder/404.sql");
            assert_eq!(expected, actual);
        }
    }
}
