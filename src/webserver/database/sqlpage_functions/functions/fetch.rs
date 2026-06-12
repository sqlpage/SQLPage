use super::*;

pub(super) fn build_request<'a>(
    client: &'a awc::Client,
    http_request: &'a HttpFetchRequest<'_>,
) -> anyhow::Result<awc::ClientRequest> {
    use awc::http::Method;
    let method = if let Some(method) = &http_request.method {
        Method::from_str(method).with_context(|| format!("Invalid HTTP method: {method}"))?
    } else {
        Method::GET
    };
    let mut req = client.request(method, http_request.url.as_ref());
    if let Some(timeout) = http_request.timeout_ms {
        req = req.timeout(core::time::Duration::from_millis(timeout));
    }
    for (k, v) in &http_request.headers {
        req = req.insert_header((k.as_ref(), v.as_ref()));
    }
    if let Some(username) = &http_request.username {
        let password = http_request.password.as_deref().unwrap_or_default();
        req = req.basic_auth(username, password);
    }
    Ok(req)
}

pub(super) fn prepare_request_body(
    body: &serde_json::value::RawValue,
    mut req: awc::ClientRequest,
) -> anyhow::Result<(String, awc::ClientRequest)> {
    let val = body.get();
    let body_str = if val.starts_with('"') {
        serde_json::from_str::<'_, String>(val).with_context(|| {
            format!("Invalid JSON string in the body of the HTTP request: {val}")
        })?
    } else {
        req = req.content_type("application/json");
        val.to_owned()
    };
    Ok((body_str, req))
}

pub(super) fn fetch_span(http_request: &HttpFetchRequest<'_>) -> tracing::Span {
    let method = http_request.method.as_deref().unwrap_or("GET");
    tracing::info_span!(
        "http.client",
        "otel.name" = format!("{method}"),
        { otel::HTTP_REQUEST_METHOD } = method,
        { otel::URL_FULL } = %http_request.url,
        { otel::HTTP_REQUEST_BODY_SIZE } = tracing::field::Empty,
        { otel::HTTP_RESPONSE_STATUS_CODE } = tracing::field::Empty,
    )
}

pub(super) fn send_request(
    request: &RequestInfo,
    http_request: &HttpFetchRequest<'_>,
) -> anyhow::Result<awc::SendClientRequest> {
    let client = make_http_client(&request.app_state.config)
        .with_context(|| "Unable to create an HTTP client")?;
    let req = build_request(&client, http_request)?;

    log::info!("Fetching {}", http_request.url);
    if let Some(body) = &http_request.body {
        let (body, req) = prepare_request_body(body, req)?;
        tracing::Span::current().record(
            otel::HTTP_REQUEST_BODY_SIZE,
            i64::try_from(body.len()).unwrap_or(i64::MAX),
        );
        Ok(req.send_body(body))
    } else {
        Ok(req.send())
    }
}

pub(super) async fn fetch(
    request: &RequestInfo,
    http_request: Option<HttpFetchRequest<'_>>,
) -> anyhow::Result<Option<String>> {
    let Some(http_request) = http_request else {
        return Ok(None);
    };
    let fetch_span = fetch_span(&http_request);

    async {
        let response_result = send_request(request, &http_request)?.await;
        let mut response = response_result
            .map_err(|e| anyhow!("Unable to fetch {}: {e}", http_request.url))?;

        tracing::Span::current().record(
            otel::HTTP_RESPONSE_STATUS_CODE,
            i64::from(response.status().as_u16()),
        );

        log::debug!(
            "Finished fetching {}. Status: {}",
            http_request.url,
            response.status()
        );
        log::debug!(
            "Fetch response headers for {}: content_type={:?}",
            http_request.url,
            response
                .headers()
                .get("content-type")
                .and_then(|value| value.to_str().ok())
        );

        let body = response
            .body()
            .await
            .with_context(|| {
                format!(
                    "Unable to read the body of the response from {}",
                    http_request.url
                )
            })?
            .to_vec();
        log::debug!(
            "Fetched {} response body: body_len={} bytes, encoding={:?}",
            http_request.url,
            body.len(),
            http_request.response_encoding
        );
        let response_str = decode_response(body, http_request.response_encoding.as_deref())?;
        Ok(Some(response_str))
    }
    .instrument(fetch_span)
    .await
}

pub(super) fn decode_response(response: Vec<u8>, encoding: Option<&str>) -> anyhow::Result<String> {
    match encoding {
        Some("base64") => Ok(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            response,
        )),
        Some("base64url") => Ok(base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE,
            response,
        )),
        Some("hex") => Ok(response.into_iter().fold(String::new(), |mut acc, byte| {
            write!(&mut acc, "{byte:02x}").unwrap();
            acc
        })),
        Some(encoding_label) => Ok(encoding_rs::Encoding::for_label(encoding_label.as_bytes())
            .with_context(|| format!("Invalid encoding name: {encoding_label}"))?
            .decode(&response)
            .0
            .into_owned()),
        None => {
            let body_str = String::from_utf8(response);
            match body_str {
                Ok(body_str) => Ok(body_str),
                Err(decoding_error) => {
                    log::warn!(
                        "fetch(...) response is not UTF-8 and no encoding was specified. Decoding the response as base64. Please explicitly set the encoding to \"base64\" if this is the expected behavior."
                    );
                    Ok(base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        decoding_error.into_bytes(),
                    ))
                }
            }
        }
    }
}
