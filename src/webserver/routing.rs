use std::path::PathBuf;
use RoutingAction::{Error, Execute, NotFound, Redirect, Serve};

const INDEX: &'static str = "index.sql";
const NOT_FOUND: &'static str = "404.sql";
const EXECUTION_EXTENSION: &'static str = "sql";

#[derive(Debug, PartialEq)]
pub enum RoutingAction {
    Error(String),
    Execute(PathBuf),
    NotFound(PathBuf),
    Redirect(String),
    Serve(PathBuf),
}

pub trait ExecutionStore {
    fn contains(&self, path: &PathBuf) -> bool;
}

pub fn calculate_route<T>(uri: &str, store: T) -> RoutingAction
where
    T: ExecutionStore,
{
    let mut path = PathBuf::from(uri);
    match path.extension() {
        None => {
            if uri.ends_with("/") {
                path.push(INDEX);
                find_execution(path, store)
            } else {
                Redirect(format!("{}/", uri))
            }
        }
        Some(extension) => {
            if extension == EXECUTION_EXTENSION {
                find_execution(path, store)
            } else {
                Serve(PathBuf::from(uri))
            }
        }
    }
}

fn find_execution<T>(path: PathBuf, store: T) -> RoutingAction
where
    T: ExecutionStore,
{
    if store.contains(&path) {
        Execute(path)
    } else {
        find_not_found(path, store)
    }
}

fn find_not_found<T>(path: PathBuf, store: T) -> RoutingAction
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

fn path_to_string(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::RoutingAction::{Error, Execute, NotFound, Redirect, Serve};
    use super::{calculate_route, path_to_string, ExecutionStore};
    use std::path::PathBuf;

    #[test]
    fn root_path_executes_index_sql() {
        let actual = calculate_route("/", Store::default());
        let expected = Execute(PathBuf::from("/index.sql"));

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_sql_extension_executes_corresponding_sql_file() {
        let actual = calculate_route("/index.sql", Store::default());
        let expected = Execute(PathBuf::from("/index.sql"));

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_sql_extension_executes_corresponding_not_found_file() {
        let actual = calculate_route("/unknown.sql", Store::default());
        let expected = NotFound(PathBuf::from("/404.sql"));

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_sql_extension_executes_deeper_not_found_file_if_exists() {
        let actual = calculate_route("/unknown/unknown.sql", Store::new("/unknown/404.sql"));
        let expected = NotFound(PathBuf::from("/unknown/404.sql"));

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_sql_extension_executes_deepest_not_found_file_that_exists() {
        let actual = calculate_route(
            "/unknown/unknown/unknown.sql",
            Store::new("/unknown/404.sql"),
        );
        let expected = NotFound(PathBuf::from("/unknown/404.sql"));

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_sql_extension_errors_when_no_not_found_file_available() {
        let actual = calculate_route("/unknown.sql", Store::empty());
        let expected = Error("/unknown.sql".to_string());

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_no_extension_and_no_corresponding_sql_file_redirects_with_trailing_slash() {
        let actual = calculate_route("/folder", Store::default());
        let expected = Redirect("/folder/".to_string());

        assert_eq!(expected, actual);
    }

    #[test]
    fn path_with_trailing_slash_executes_index_sql_from_directory() {
        let actual = calculate_route("/folder/", Store::new("/folder/index.sql"));
        let expected = Execute(PathBuf::from("/folder/index.sql"));

        assert_eq!(expected, actual);
    }

    #[test]
    fn non_sql_file_extension_serves_corresponding_asset() {
        let actual = calculate_route("/favicon.ico", Store::default());
        let expected = Serve(PathBuf::from("/favicon.ico"));

        assert_eq!(expected, actual);
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

    impl Default for Store {
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
