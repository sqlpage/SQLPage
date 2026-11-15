use std::borrow::Cow;

use crate::webserver::http_request_info::RequestInfo;

use super::persist_uploaded_file::{allowed_extension, persist_file};

/// Persists all the files uploaded in a field (a file input with the `multiple` attribute),
/// and returns their paths as a JSON array. Returns NULL if no file was uploaded.
pub(super) async fn persist_uploaded_files<'a>(
    request: &'a RequestInfo,
    field_name: Cow<'a, str>,
    folder: Option<Cow<'a, str>>,
    allowed_extensions: Option<Cow<'a, str>>,
    mode: Option<Cow<'a, str>>,
) -> anyhow::Result<Option<String>> {
    let files: Vec<_> = request.uploaded_files.all(&field_name).collect();
    if files.is_empty() {
        return Ok(None);
    }
    // Check every extension before writing anything: a rejected file leaves no partial upload behind.
    let extensions = files
        .iter()
        .map(|file| allowed_extension(file, allowed_extensions.as_deref()))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let mut paths = Vec::with_capacity(files.len());
    for (file, extension) in files.into_iter().zip(extensions) {
        paths.push(
            persist_file(
                request,
                file,
                extension,
                &field_name,
                folder.as_deref(),
                mode.as_deref(),
            )
            .await?,
        );
    }
    Ok(Some(serde_json::to_string(&paths)?))
}
