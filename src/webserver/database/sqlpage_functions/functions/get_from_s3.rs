use std::borrow::Cow;

use crate::webserver::http_request_info::RequestInfo;
use super::super::s3;

pub(super) async fn get_from_s3<'a>(
    request: &'a RequestInfo,
    bucket: Option<Cow<'a, str>>, key: Cow<'a, str>,
) -> anyhow::Result<String> {
    s3::get_from_s3(request, bucket, key).await
}
