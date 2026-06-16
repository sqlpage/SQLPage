use std::borrow::Cow;

use anyhow::Context;

use crate::{
    filesystem::FileAccess,
    webserver::{
        database::blob_to_data_url::vec_to_data_uri_with_mime,
        http_request_info::RequestInfo,
    },
};

use super::uploaded_file_mime_type::{mime_from_upload_path, mime_guess_from_filename};

pub(super) async fn read_file_bytes(request: &RequestInfo, path_str: &str) -> Result<Vec<u8>, anyhow::Error> {
    let path = std::path::Path::new(path_str);
    // If the path is relative, it's relative to the web root, not the current working directory,
    // and it can be fetched from the on-database filesystem table
    if path.is_relative() {
        request
            .app_state
            .file_system
            .read_file(&request.app_state, FileAccess::privileged(path))
            .await
    } else {
        tokio::fs::read(path)
            .await
            .with_context(|| format!("Unable to read file \"{}\"", path.display()))
    }
}

pub(super) async fn read_file_as_data_url<'a>(
    request: &'a RequestInfo,
    file_path: Option<Cow<'a, str>>,
) -> Result<Option<Cow<'a, str>>, anyhow::Error> {
    let Some(file_path) = file_path else {
        log::debug!("read_file: first argument is NULL, returning NULL");
        return Ok(None);
    };
    let bytes = read_file_bytes(request, &file_path).await?;
    let mime = mime_from_upload_path(request, &file_path).map_or_else(
        || Cow::Owned(mime_guess_from_filename(&file_path)),
        Cow::Borrowed,
    );
    let data_url = vec_to_data_uri_with_mime(&bytes, &mime.to_string());
    Ok(Some(Cow::Owned(data_url)))
}
