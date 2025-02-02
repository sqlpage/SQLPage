use crate::file_cache::FileCache;
use crate::filesystem::FileSystem;
use crate::webserver::database::ParsedSqlFile;
use awc::http::uri::PathAndQuery;
use log::debug;
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
    async fn contains(&self, path: &Path) -> bool;
}

pub trait RoutingConfig {
    fn prefix(&self) -> &str;
}

pub(crate) struct AppFileStore<'a> {
    cache: &'a FileCache<ParsedSqlFile>,
    filesystem: &'a FileSystem,
}

impl<'a> AppFileStore<'a> {
    pub fn new(
        cache: &'a FileCache<ParsedSqlFile>,
        filesystem: &'a FileSystem,
    ) -> AppFileStore<'a> {
        Self { cache, filesystem }
    }
}

impl FileStore for AppFileStore<'_> {
    async fn contains(&self, path: &Path) -> bool {
        self.cache.contains(path).await || self.filesystem.contains(path).await
    }
}

pub async fn calculate_route<T, C>(
    path_and_query: &PathAndQuery,
    store: &T,
    config: &C,
) -> RoutingAction
where
    T: FileStore,
    C: RoutingConfig,
{
    let result = match check_path(path_and_query, config) {
        Ok(path) => match path.extension() {
            None => calculate_route_without_extension(path_and_query, path, store).await,
            Some(extension) => {
                let ext = extension.to_str().unwrap_or_default();
                find_file_or_not_found(&path, ext, store).await
            }
        },
        Err(action) => action,
    };
    debug!("Route: [{}] -> {:?}", path_and_query, result);
    result
}

fn check_path<C>(path_and_query: &PathAndQuery, config: &C) -> Result<PathBuf, RoutingAction>
where
    C: RoutingConfig,
{
    match path_and_query.path().strip_prefix(config.prefix()) {
        None => Err(Redirect(config.prefix().to_string())),
        Some(path) => Ok(PathBuf::from(path)),
    }
}

async fn calculate_route_without_extension<T>(
    path_and_query: &PathAndQuery,
    mut path: PathBuf,
    store: &T,
) -> RoutingAction
where
    T: FileStore,
{
    if path_and_query.path().ends_with(FORWARD_SLASH) {
        path.push(INDEX);
        find_file_or_not_found(&path, SQL_EXTENSION, store).await
    } else {
        let path_with_ext = path.with_extension(SQL_EXTENSION);
        match find_file(&path_with_ext, SQL_EXTENSION, store).await {
            Some(action) => action,
            None => Redirect(append_to_path(path_and_query, FORWARD_SLASH)),
        }
    }
}

async fn find_file_or_not_found<T>(path: &Path, extension: &str, store: &T) -> RoutingAction
where
    T: FileStore,
{
    match find_file(path, extension, store).await {
        None => find_not_found(path, store).await,
        Some(execute) => execute,
    }
}

async fn find_file<T>(path: &Path, extension: &str, store: &T) -> Option<RoutingAction>
where
    T: FileStore,
{
    if store.contains(path).await {
        if extension == SQL_EXTENSION {
            Some(Execute(path.to_path_buf()))
        } else {
            Some(Serve(path.to_path_buf()))
        }
    } else {
        None
    }
}

async fn find_not_found<T>(path: &Path, store: &T) -> RoutingAction
where
    T: FileStore,
{
    let mut parent = path.parent();
    while let Some(p) = parent {
        let target = p.join(NOT_FOUND);
        if store.contains(&target).await {
            return CustomNotFound(target);
        }
        parent = p.parent();
    }

    NotFound
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
    use StoreConfig::{Default, Empty, File};

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
        use super::StoreConfig::Default;
        use super::{do_route, redirect};

        #[tokio::test]
        async fn path_without_site_prefix_redirects_to_site_prefix() {
            let actual = do_route("/path", Default, Some("/prefix/")).await;
            let expected = redirect("/prefix/");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn no_extension_and_no_corresponding_file_redirects_with_trailing_slash() {
            let actual = do_route("/folder", Default, None).await;
            let expected = redirect("/folder/");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn no_extension_no_corresponding_file_redirects_with_trailing_slash_and_query() {
            let actual = do_route("/folder?misc=1&foo=bar", Default, None).await;
            let expected = redirect("/folder/?misc=1&foo=bar");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn no_extension_site_prefix_and_no_corresponding_file_redirects_with_trailing_slash()
        {
            let actual = do_route("/prefix/folder", Default, Some("/prefix/")).await;
            let expected = redirect("/prefix/folder/");

            assert_eq!(expected, actual);
        }
    }

    async fn do_route(path: &str, config: StoreConfig, prefix: Option<&str>) -> RoutingAction {
        let store = match config {
            Default => Store::default(),
            Empty => Store::empty(),
            File(file) => Store::new(file),
        };
        let config = match prefix {
            None => Config::default(),
            Some(value) => Config::new(value),
        };
        calculate_route(&PathAndQuery::from_str(path).unwrap(), &store, &config).await
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

        fn empty() -> Self {
            Self { contents: vec![] }
        }
    }

    impl StdDefault for Store {
        fn default() -> Self {
            Self {
                contents: Self::default_contents(),
            }
        }
    }

    impl FileStore for Store {
        async fn contains(&self, path: &Path) -> bool {
            self.contents.contains(&path.to_string_lossy().to_string())
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
}
