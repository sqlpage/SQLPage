
/// Returns a random string of the specified length.
pub(super) async fn random_string(len: usize) -> anyhow::Result<String> {
    // OsRng can block on Linux, so we run this on a blocking thread.
    Ok(tokio::task::spawn_blocking(move || random_string_sync(len)).await?)
}

/// Returns a random string of the specified length.
pub(crate) fn random_string_sync(len: usize) -> String {
    use rand::{RngExt, distr::Alphanumeric};
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

#[tokio::test]
pub(super) async fn test_random_string() {
    let s = random_string(10).await.unwrap();
    assert_eq!(s.len(), 10);
}
