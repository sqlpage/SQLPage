use super::*;

/// Returns the directory where the sqlpage.json configuration file, templates, and migrations are located.
pub(super) async fn configuration_directory(request: &RequestInfo) -> String {
    request
        .app_state
        .config
        .configuration_directory
        .to_string_lossy()
        .into_owned()
}
