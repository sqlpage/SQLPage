use std::borrow::Cow;

use crate::webserver::http_request_info::RequestInfo;
use super::super::s3;

pub(super) async fn upload_to_s3<'a>(
    request: &'a RequestInfo,
    bucket: Option<Cow<'a, str>>, data: Cow<'a, str>, key: Cow<'a, str>,
) -> anyhow::Result<String> {
    s3::upload_to_s3(request, bucket, data, key).await
}
