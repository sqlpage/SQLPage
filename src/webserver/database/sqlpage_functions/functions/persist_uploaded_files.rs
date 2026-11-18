use std::borrow::Cow;
use std::future::Future;
use std::path::Path;

use serde_json::{Value, json};

use crate::webserver::http_request_info::RequestInfo;

use super::persist_uploaded_file::{allowed_extension, persist_file};

/// Persists all the files uploaded in a field (a file input with the `multiple` attribute), and
/// returns a JSON array with one object per file: its persisted `path`, its original `name`, its
/// `mime_type` and its `size` in bytes. Returns NULL if no file was uploaded.
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
    let (field, folder, mode) = (&*field_name, folder.as_deref(), mode.as_deref());
    let web_root = &request.app_state.config.web_root;
    let persisted = persist_all(
        web_root,
        files.into_iter().zip(extensions),
        |(file, extension)| async move {
            let path = persist_file(request, file, extension, field, folder, mode).await?;
            Ok(json!({
                "path": path,
                "name": file.file_name,
                "mime_type": file.content_type.as_ref().map(ToString::to_string),
                "size": file.size,
            }))
        },
    )
    .await?;
    Ok(Some(serde_json::to_string(&persisted)?))
}

/// Runs `persist` on every item. If one of them fails, the files already persisted by this call
/// (their `path`, relative to `web_root`) are removed, so that a failed call leaves nothing behind.
async fn persist_all<I, F, Fut>(
    web_root: &Path,
    items: I,
    mut persist: F,
) -> anyhow::Result<Vec<Value>>
where
    I: IntoIterator,
    F: FnMut(I::Item) -> Fut,
    Fut: Future<Output = anyhow::Result<Value>>,
{
    let mut persisted = Vec::new();
    for item in items {
        match persist(item).await {
            Ok(file) => persisted.push(file),
            Err(error) => {
                for path in persisted.iter().filter_map(|file| file["path"].as_str()) {
                    let full_path = web_root.join(path.trim_start_matches('/'));
                    if let Err(remove_error) = tokio::fs::remove_file(&full_path).await {
                        log::warn!(
                            "Unable to remove {} after a failed upload: {remove_error}",
                            full_path.display()
                        );
                    }
                }
                return Err(error);
            }
        }
    }
    Ok(persisted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn failed_persist_removes_the_files_already_persisted() {
        let root = tempfile::tempdir().unwrap();
        let result = persist_all(root.path(), 1..=3, |i| {
            let root = root.path().to_path_buf();
            async move {
                if i == 3 {
                    anyhow::bail!("disk full");
                }
                let name = format!("file{i}.txt");
                tokio::fs::write(root.join(&name), "x").await?;
                Ok(json!({ "path": format!("/{name}") }))
            }
        })
        .await;
        assert_eq!(result.unwrap_err().to_string(), "disk full");
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }
}
