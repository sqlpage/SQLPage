use super::*;

pub(super) async fn current_working_directory() -> anyhow::Result<String> {
    std::env::current_dir()
        .with_context(|| "unable to access the current working directory")
        .map(|x| x.to_string_lossy().into_owned())
}
