use super::*;

pub(super) fn mime_from_upload_path<'a>(request: &'a RequestInfo, path: &str) -> Option<&'a mime_guess::Mime> {
    request.uploaded_files.values().find_map(|uploaded_file| {
        if uploaded_file.file.path() == OsStr::new(path) {
            uploaded_file.content_type.as_ref()
        } else {
            None
        }
    })
}

pub(super) fn mime_guess_from_filename(filename: &str) -> mime_guess::Mime {
    let maybe_mime = mime_guess::from_path(filename).first();
    maybe_mime.unwrap_or(mime::APPLICATION_OCTET_STREAM)
}

pub(super) async fn uploaded_file_mime_type<'a>(
    request: &'a RequestInfo,
    upload_name: Cow<'a, str>,
) -> Option<Cow<'a, str>> {
    let mime = request
        .uploaded_files
        .get(&*upload_name)?
        .content_type
        .as_ref()?;
    Some(Cow::Borrowed(mime.as_ref()))
}
