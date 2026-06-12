use super::*;

/// Returns the directory where the .sql files are located (the web root).
pub(super) async fn web_root(request: &RequestInfo) -> String {
    request
        .app_state
        .config
        .web_root
        .to_string_lossy()
        .into_owned()
}
