use std::borrow::Cow;

use super::super::s3;
use crate::webserver::http_request_info::RequestInfo;

pub(super) async fn upload_to_s3<'a>(
    request: &'a RequestInfo,
    bucket: Option<Cow<'a, str>>,
    data: Cow<'a, str>,
    key: Cow<'a, str>,
) -> anyhow::Result<String> {
    // Keep the AWS SDK future out of the shared SQL function dispatch future.
    Box::pin(s3::upload_to_s3(request, bucket, data, key)).await
}
