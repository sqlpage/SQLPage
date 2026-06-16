
/// Returns the version of the sqlpage that is running.
pub(super) async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
