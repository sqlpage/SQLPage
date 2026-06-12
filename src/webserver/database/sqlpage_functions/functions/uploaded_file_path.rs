use super::*;

pub(super) async fn uploaded_file_path<'a>(
    request: &'a RequestInfo,
    upload_name: Cow<'a, str>,
) -> Option<Cow<'a, str>> {
    let uploaded_file = request.uploaded_files.get(&*upload_name)?;
    Some(uploaded_file.file.path().to_string_lossy())
}
