use opentelemetry_semantic_conventions::attribute as otel;
use tracing::Instrument;

use crate::webserver::{
    database::sqlpage_functions::http_fetch_request::HttpFetchRequest,
    http_request_info::RequestInfo,
};

use super::fetch::{decode_response, fetch_span, send_request};

pub(super) async fn fetch_with_meta(
    request: &RequestInfo,
    http_request: Option<HttpFetchRequest<'_>>,
) -> anyhow::Result<Option<String>> {
    use serde::{Serializer, ser::SerializeMap};

    let Some(http_request) = http_request else {
        return Ok(None);
    };

    let fetch_span = fetch_span(&http_request);

    async {
        let response_result = send_request(request, &http_request)?.await;

        let mut resp_str = Vec::new();
        let mut encoder = serde_json::Serializer::new(&mut resp_str);
        let mut obj = encoder.serialize_map(Some(3))?;
        match response_result {
            Ok(mut response) => {
                let status = response.status();
                tracing::Span::current()
                    .record(otel::HTTP_RESPONSE_STATUS_CODE, i64::from(status.as_u16()));
                obj.serialize_entry("status", &status.as_u16())?;
                let mut has_error = false;
                if status.is_server_error() {
                    has_error = true;
                    obj.serialize_entry("error", &format!("Server error: {status}"))?;
                }

                let headers = response.headers();

                let is_json = headers
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or_default()
                    .starts_with("application/json");

                obj.serialize_entry(
                    "headers",
                    &headers
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default()))
                        .collect::<std::collections::HashMap<_, _>>(),
                )?;

                match response.body().await {
                    Ok(body) => {
                        let body_bytes = body.to_vec();
                        let body_str =
                            decode_response(body_bytes, http_request.response_encoding.as_deref())?;
                        if is_json {
                            obj.serialize_entry(
                                "json_body",
                                &serde_json::value::RawValue::from_string(body_str)?,
                            )?;
                        } else {
                            obj.serialize_entry("body", &body_str)?;
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to read response body: {e}");
                        if !has_error {
                            obj.serialize_entry(
                                "error",
                                &format!("Failed to read response body: {e}"),
                            )?;
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Request failed: {e}");
                obj.serialize_entry("error", &format!("Request failed: {e}"))?;
            }
        }

        obj.end()?;
        let return_value = String::from_utf8(resp_str)?;
        Ok(Some(return_value))
    }
    .instrument(fetch_span)
    .await
}
