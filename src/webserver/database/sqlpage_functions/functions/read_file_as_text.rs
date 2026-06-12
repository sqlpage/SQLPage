use super::*;

/// Returns the contents of a file as a string
pub(super) async fn read_file_as_text<'a>(
    request: &'a RequestInfo,
    file_path: Option<Cow<'a, str>>,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    let Some(file_path) = file_path else {
        log::debug!("read_file: first argument is NULL, returning NULL");
        return Ok(None);
    };
    let bytes = read_file_bytes(request, &file_path).await?;
    let as_str = String::from_utf8(bytes).with_context(|| {
        format!("read_file_as_text: {file_path} does not contain raw UTF8 text")
    })?;
    Ok(Some(Cow::Owned(as_str)))
}
