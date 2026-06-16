use std::borrow::Cow;

use crate::webserver::http_request_info::RequestInfo;

pub(super) async fn uploaded_file_name<'a>(
    request: &'a RequestInfo,
    upload_name: Cow<'a, str>,
) -> Option<Cow<'a, str>> {
    let fname = request
        .uploaded_files
        .get(&*upload_name)?
        .file_name
        .as_ref()?;
    Some(Cow::Borrowed(fname.as_str()))
}
