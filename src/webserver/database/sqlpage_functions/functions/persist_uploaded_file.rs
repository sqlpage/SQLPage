use std::borrow::Cow;

use anyhow::Context;

use crate::webserver::http_request_info::RequestInfo;

use super::random_string::random_string_sync;

const DEFAULT_ALLOWED_EXTENSIONS: &str =
    "jpg,jpeg,png,gif,bmp,webp,pdf,txt,doc,docx,xls,xlsx,csv,mp3,mp4,wav,avi,mov";

pub(super) async fn persist_uploaded_file<'a>(
    request: &'a RequestInfo,
    field_name: Cow<'a, str>,
    folder: Option<Cow<'a, str>>,
    allowed_extensions: Option<Cow<'a, str>>,
    mode: Option<Cow<'a, str>>,
) -> anyhow::Result<Option<String>> {
    let folder = folder.unwrap_or(Cow::Borrowed("uploads"));
    let allowed_extensions_str =
        allowed_extensions.unwrap_or(Cow::Borrowed(DEFAULT_ALLOWED_EXTENSIONS));
    let allowed_extensions = allowed_extensions_str.split(',');
    let Some(uploaded_file) = request.uploaded_files.get(&field_name.to_string()) else {
        return Ok(None);
    };
    let file_name = uploaded_file.file_name.as_deref().unwrap_or_default();
    let extension = file_name.split('.').next_back().unwrap_or_default();
    if !allowed_extensions
        .clone()
        .any(|x| x.eq_ignore_ascii_case(extension))
    {
        let exts = allowed_extensions.collect::<Vec<_>>().join(", ");
        anyhow::bail!("file extension {extension} is not allowed. Allowed extensions: {exts}");
    }
    // Resolve the folder path relative to the web root.
    // `folder` is trusted application input: it is expected to be a constant chosen by the
    // app author in their SQL code, never attacker-controlled request data. It is joined
    // directly to the web root, so a `folder` containing `..` or an absolute path would let
    // the caller write the uploaded file outside the web root. Callers must not pass
    // untrusted input (form fields, query parameters, headers, ...) as the folder.
    let web_root = &request.app_state.config.web_root;
    let target_folder = web_root.join(&*folder);
    // create the folder if it doesn't exist
    tokio::fs::create_dir_all(&target_folder)
        .await
        .with_context(|| format!("unable to create folder {}", target_folder.display()))?;
    let date = chrono::Utc::now().format("%Y-%m-%d_%Hh%Mm%Ss");
    let random_part = random_string_sync(8);
    let random_target_name = format!("{date}_{random_part}.{extension}");
    let target_path = target_folder.join(&random_target_name);
    tokio::fs::copy(&uploaded_file.file.path(), &target_path)
        .await
        .with_context(|| {
            format!(
                "unable to copy uploaded file {field_name:?} to \"{}\"",
                target_path.display()
            )
        })?;
    set_file_mode(&target_path, mode.as_deref()).await?;
    // remove the WEB_ROOT prefix from the path, but keep the leading slash
    let path = "/".to_string()
        + target_path
            .strip_prefix(web_root)?
            .to_str()
            .with_context(|| {
                format!(
                    "unable to convert path \"{}\" to a string",
                    target_path.display()
                )
            })?;
    Ok(Some(path))
}

#[cfg(unix)]
pub(super) async fn set_file_mode(path: &std::path::Path, mode: Option<&str>) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = if let Some(mode) = mode {
        u32::from_str_radix(mode, 8)
            .with_context(|| format!("unable to parse file mode {mode:?} as an octal number"))?
    } else {
        0o600
    };
    tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .await
        .with_context(|| format!("unable to set permissions on {}", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
pub(super) async fn set_file_mode(_path: &std::path::Path, _mode: Option<&str>) -> anyhow::Result<()> {
    Ok(())
}
