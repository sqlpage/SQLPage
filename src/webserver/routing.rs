use awc::http::Uri;
use std::ffi::OsStr;
use std::path::PathBuf;
use RoutingAction::{Error, Execute, NotFound, Redirect, Serve};

const INDEX: &'static str = "index.sql";
const NOT_FOUND: &'static str = "404.sql";
const EXECUTION_EXTENSION: &'static str = "sql";
const FORWARD_SLASH: &'static str = "/";

#[derive(Debug, PartialEq)]
pub enum RoutingAction {
    Error(String),
    Execute(PathBuf),
    NotFound(PathBuf),
    Redirect(Uri),
    Serve(PathBuf),
}

pub trait ExecutionStore {
    async fn contains(&self, path: &PathBuf) -> bool;
}

pub trait RoutingConfig {
    fn prefix(&self) -> &Uri;
}

pub async fn calculate_route<T, C>(uri: Uri, store: &T, config: &C) -> RoutingAction
where
    T: ExecutionStore,
    C: RoutingConfig,
{
    match check_uri(&uri, config) {
        Ok(path) => match path.clone().extension() {
            None => calculate_route_without_extension(&uri, path, store).await,
            Some(extension) => calculate_route_with_extension(path, extension, store).await,
        },
        Err(action) => action,
    }
}

fn check_uri<C>(uri: &Uri, config: &C) -> Result<PathBuf, RoutingAction>
where
    C: RoutingConfig,
{
    if uri.path().starts_with(config.prefix().path()) {
        let mut result = String::from("/");
        result.push_str(
            uri.path()
                .strip_prefix(config.prefix().path())
                .expect("Unable to remove expected prefix from path"),
        );

        Ok(PathBuf::from(result))
    } else {
        Err(Redirect(config.prefix().clone()))
    }
}

async fn calculate_route_without_extension<T>(
    uri: &Uri,
    mut path: PathBuf,
    store: &T,
) -> RoutingAction
where
    T: ExecutionStore,
{
    if uri.path().ends_with(FORWARD_SLASH) {
        path.push(INDEX);
        find_execution_or_not_found(&path, store).await
    } else {
        let path_with_ext = path.with_extension(EXECUTION_EXTENSION);
        match find_execution(&path_with_ext, store).await {
            Some(action) => action,
            None => Redirect(append_to_uri_path(&uri, FORWARD_SLASH)),
        }
    }
}

async fn calculate_route_with_extension<T>(
    path: PathBuf,
    extension: &OsStr,
    store: &T,
) -> RoutingAction
where
    T: ExecutionStore,
{
    if extension == EXECUTION_EXTENSION {
        find_execution_or_not_found(&path, store).await
    } else {
        Serve(path)
    }
}

async fn find_execution_or_not_found<T>(path: &PathBuf, store: &T) -> RoutingAction
where
    T: ExecutionStore,
{
    match find_execution(path, store).await {
        None => find_not_found(path, store).await,
        Some(execute) => execute,
    }
}

async fn find_execution<T>(path: &PathBuf, store: &T) -> Option<RoutingAction>
where
    T: ExecutionStore,
{
    if store.contains(path).await {
        Some(Execute(path.clone()))
    } else {
        None
    }
}

async fn find_not_found<T>(path: &PathBuf, store: &T) -> RoutingAction
where
    T: ExecutionStore,
{
    let mut parent = path.parent();
    while let Some(p) = parent {
        let target = p.join(NOT_FOUND);
        if store.contains(&target).await {
            return NotFound(target);
        } else {
            parent = p.parent()
        }
    }

    Error(path_to_string(path))
}

fn append_to_uri_path(uri: &Uri, append: &str) -> Uri {
    let mut full_uri = uri.to_string();
    full_uri.insert_str(uri.path().len(), append);
    full_uri.parse().expect("Could not append uri path")
}

fn path_to_string(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::RoutingAction::{Error, Execute, NotFound, Redirect, Serve};
    use super::{calculate_route, path_to_string, ExecutionStore, RoutingAction, RoutingConfig};
    use awc::http::Uri;
    use std::default::Default as StdDefault;
    use std::path::PathBuf;
    use std::str::FromStr;
    use StoreConfig::{Default, Empty, File};

    mod execute {
        use super::StoreConfig::{Default, File};
        use super::{do_route, execute};

        #[tokio::test]
        async fn root_path_executes_index() {
            let actual = do_route("/", Default, None).await;
            let expected = execute("/index.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn root_path_and_site_prefix_executes_index() {
            let actual = do_route("/prefix/", Default, Some("/prefix/")).await;
            let expected = execute("/index.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn extension() {
            let actual = do_route("/index.sql", Default, None).await;
            let expected = execute("/index.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn extension_and_site_prefix() {
            let actual = do_route("/prefix/index.sql", Default, Some("/prefix/")).await;
            let expected = execute("/index.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn no_extension() {
            let actual = do_route("/path", File("/path.sql"), None).await;
            let expected = execute("/path.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn no_extension_and_site_prefix() {
            let actual = do_route("/prefix/path", File("/path.sql"), Some("/prefix/")).await;
            let expected = execute("/path.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn trailing_slash_executes_index_in_directory() {
            let actual = do_route("/folder/", File("/folder/index.sql"), None).await;
            let expected = execute("/folder/index.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn trailing_slash_and_site_prefix_executes_index_in_directory() {
            let actual = do_route(
                "/prefix/folder/",
                File("/folder/index.sql"),
                Some("/prefix/"),
            )
            .await;
            let expected = execute("/folder/index.sql");

            assert_eq!(expected, actual);
        }
    }

    mod not_found {
        use super::StoreConfig::{Default, File};
        use super::{do_route, not_found};

        #[tokio::test]
        async fn sql_extension() {
            let actual = do_route("/unknown.sql", Default, None).await;
            let expected = not_found("/404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn sql_extension_and_site_prefix() {
            let actual = do_route("/prefix/unknown.sql", Default, Some("/prefix/")).await;
            let expected = not_found("/404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn sql_extension_executes_deeper_not_found_file_if_exists() {
            let actual = do_route("/unknown/unknown.sql", File("/unknown/404.sql"), None).await;
            let expected = not_found("/unknown/404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn sql_extension_and_site_prefix_executes_deeper_not_found_file_if_exists() {
            let actual = do_route(
                "/prefix/unknown/unknown.sql",
                File("/unknown/404.sql"),
                Some("/prefix/"),
            )
            .await;
            let expected = not_found("/unknown/404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn sql_extension_executes_deepest_not_found_file_that_exists() {
            let actual = do_route(
                "/unknown/unknown/unknown.sql",
                File("/unknown/404.sql"),
                None,
            )
            .await;
            let expected = not_found("/unknown/404.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn sql_extension_and_site_prefix_executes_deepest_not_found_file_that_exists() {
            let actual = do_route(
                "/prefix/unknown/unknown/unknown.sql",
                File("/unknown/404.sql"),
                Some("/prefix/"),
            )
            .await;
            let expected = not_found("/unknown/404.sql");

            assert_eq!(expected, actual);
        }
    }

    mod error {
        use super::StoreConfig::Empty;
        use super::{do_route, error};

        #[tokio::test]
        async fn sql_extension_errors_when_no_not_found_file_available() {
            let actual = do_route("/unknown.sql", Empty, None).await;
            let expected = error("/unknown.sql");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn sql_extension_and_site_prefix_errors_when_no_not_found_file_available() {
            let actual = do_route("/prefix/unknown.sql", Empty, Some("/prefix/")).await;
            let expected = error("/unknown.sql");

            assert_eq!(expected, actual);
        }
    }

    mod asset {
        use super::StoreConfig::Default;
        use super::{do_route, serve};

        #[tokio::test]
        async fn serves_corresponding_asset() {
            let actual = do_route("/favicon.ico", Default, None).await;
            let expected = serve("/favicon.ico");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn asset_trims_query() {
            let actual = do_route("/favicon.ico?version=10", Default, None).await;
            let expected = serve("/favicon.ico");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn asset_trims_fragment() {
            let actual = do_route("/favicon.ico#asset1", Default, None).await;
            let expected = serve("/favicon.ico");

            assert_eq!(expected, actual);
        }

        #[tokio::test]
        async fn serves_corresponding_asset_given_site_prefix() {
            let actual = do_route("/prefix/favicon.ico", Default, Some("/prefix/")).await;
            let expected = serve("/favicon.ico");

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
        async fn no_extension_no_corresponding_file_redirects_with_trailing_slash_and_fragment() {
            let actual = do_route("/folder#anchor1", Default, None).await;
            let expected = redirect("/folder/#anchor1");

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

    async fn do_route(uri: &str, config: StoreConfig, prefix: Option<&str>) -> RoutingAction {
        let store = match config {
            Default => Store::default(),
            Empty => Store::empty(),
            File(file) => Store::new(file),
        };
        let config = match prefix {
            None => Config::default(),
            Some(value) => Config::new(value),
        };
        calculate_route(Uri::from_str(uri).unwrap(), &store, &config).await
    }

    fn error(uri: &str) -> RoutingAction {
        Error(uri.to_string())
    }

    fn execute(path: &str) -> RoutingAction {
        Execute(PathBuf::from(path))
    }

    fn not_found(path: &str) -> RoutingAction {
        NotFound(PathBuf::from(path))
    }

    fn redirect(uri: &str) -> RoutingAction {
        Redirect(Uri::from_str(uri).unwrap())
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
        const INDEX: &'static str = "/index.sql";
        const NOT_FOUND: &'static str = "/404.sql";
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

    impl ExecutionStore for Store {
        async fn contains(&self, path: &PathBuf) -> bool {
            self.contents.contains(&path_to_string(path))
        }
    }

    struct Config {
        prefix: Uri,
    }

    impl Config {
        fn new(prefix: &str) -> Self {
            Self {
                prefix: prefix.parse().unwrap(),
            }
        }
    }
    impl RoutingConfig for Config {
        fn prefix(&self) -> &Uri {
            &self.prefix
        }
    }

    impl StdDefault for Config {
        fn default() -> Self {
            Self::new("/")
        }
    }
}
