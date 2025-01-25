use awc::http::Uri;
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
    fn contains(&self, path: &PathBuf) -> bool;
}

pub fn calculate_route<T>(uri: Uri, store: &T) -> RoutingAction
where
    T: ExecutionStore,
{
    let mut path = PathBuf::from(uri.path());
    match path.extension() {
        None => {
            if uri.path().ends_with(FORWARD_SLASH) {
                path.push(INDEX);
                find_execution_or_not_found(path, store)
            } else {
                let path_with_ext = path.with_extension(EXECUTION_EXTENSION);
                match find_execution(path_with_ext, store) {
                    Some(action) => action,
                    None => Redirect(append_to_uri_path(&uri, FORWARD_SLASH)),
                }
            }
        }
        Some(extension) => {
            if extension == EXECUTION_EXTENSION {
                find_execution_or_not_found(path, store)
            } else {
                Serve(PathBuf::from(uri.path()))
            }
        }
    }
}

fn find_execution_or_not_found<T>(path: PathBuf, store: &T) -> RoutingAction
where
    T: ExecutionStore,
{
    match find_execution(path.clone(), store) {
        None => find_not_found(path, store),
        Some(execute) => execute,
    }
}

fn find_execution<T>(path: PathBuf, store: &T) -> Option<RoutingAction>
where
    T: ExecutionStore,
{
    if store.contains(&path) {
        Some(Execute(path))
    } else {
        None
    }
}

fn find_not_found<T>(path: PathBuf, store: &T) -> RoutingAction
where
    T: ExecutionStore,
{
    let mut parent = path.parent();
    while let Some(p) = parent {
        let target = p.join(NOT_FOUND);
        if store.contains(&target) {
            return NotFound(target);
        } else {
            parent = p.parent()
        }
    }

    Error(path_to_string(&path))
}

fn append_to_uri_path(uri: &Uri, append: &str) -> Uri {
    let mut full_uri = uri.to_string();
    full_uri.insert_str(uri.path().len(), append);
    full_uri.parse().unwrap()
}

fn path_to_string(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::RoutingAction::{Error, Execute, NotFound, Redirect, Serve};
    use super::{calculate_route, path_to_string, ExecutionStore, RoutingAction};
    use awc::http::Uri;
    use std::default::Default as StdDefault;
    use std::path::PathBuf;
    use std::str::FromStr;
    use StoreConfig::{Default, Empty, File};

    #[test]
    fn root_path_executes_index_sql() {
        let actual = do_route("/", Default);
        let expected = execute("/index.sql");

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_sql_extension_executes_corresponding_sql_file() {
        let actual = do_route("/index.sql", Default);
        let expected = execute("/index.sql");

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_sql_extension_executes_corresponding_not_found_file() {
        let actual = do_route("/unknown.sql", Default);
        let expected = not_found("/404.sql");

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_sql_extension_executes_deeper_not_found_file_if_exists() {
        let actual = do_route("/unknown/unknown.sql", File("/unknown/404.sql"));
        let expected = not_found("/unknown/404.sql");

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_sql_extension_executes_deepest_not_found_file_that_exists() {
        let actual = do_route("/unknown/unknown/unknown.sql", File("/unknown/404.sql"));
        let expected = not_found("/unknown/404.sql");

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_sql_extension_errors_when_no_not_found_file_available() {
        let actual = do_route("/unknown.sql", Empty);
        let expected = error("/unknown.sql");

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_no_extension_and_no_corresponding_sql_file_redirects_with_trailing_slash() {
        let actual = do_route("/folder", Default);
        let expected = redirect("/folder/");

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_no_extension_executes_corresponding_sql_file_if_exists() {
        let actual = do_route("/path", File("/path.sql"));
        let expected = execute("/path.sql");

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_trailing_slash_executes_index_sql_from_directory() {
        let actual = do_route("/folder/", File("/folder/index.sql"));
        let expected = execute("/folder/index.sql");

        assert_eq!(expected, actual);
    }

    #[test]
    fn non_sql_file_extension_serves_corresponding_asset() {
        let actual = do_route("/favicon.ico", Default);
        let expected = serve("/favicon.ico");

        assert_eq!(expected, actual);
    }

    #[test]
    #[ignore]
    fn path_without_site_prefix_redirects_to_site_prefix() {
        let _prefix = "/sqlpage/";
        let actual = do_route("/path", File("/path.sql"));
        let expected = redirect("/sqlpage/");

        assert_eq!(expected, actual);
    }

    fn do_route(uri: &str, config: StoreConfig) -> RoutingAction {
        let store = match config {
            Default => Store::default(),
            Empty => Store::empty(),
            File(file) => Store::new(file),
        };
        calculate_route(Uri::from_str(uri).unwrap(), &store)
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
        fn contains(&self, path: &PathBuf) -> bool {
            self.contents.contains(&path_to_string(path))
        }
    }
}
