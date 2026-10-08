use actix_web::{dev::ServiceRequest, web::Bytes};
use anyhow::{Context, anyhow};
use opentelemetry_http::{Request, Response};
use rustls_native_certs::CertificateResult;
use std::sync::OnceLock;
use std::time::Duration;

/// Bridges the HTTP 1.x types used by OIDC and OpenTelemetry to AWC's HTTP 0.2 types.
/// Validates response headers before buffering the body.
pub(crate) async fn send_http_request<B>(
    client: &awc::Client,
    request: Request<B>,
    body_timeout: Option<Duration>,
) -> anyhow::Result<Response<Bytes>>
where
    B: actix_web::body::MessageBody + 'static,
{
    let (head, body) = request.into_parts();
    let method = awc::http::Method::from_bytes(head.method.as_str().as_bytes())?;
    let mut request = client.request(method, head.uri.to_string());
    for (name, value) in &head.headers {
        request = request.insert_header((name.as_str(), value.as_bytes()));
    }
    log::debug!("Executing HTTP request: {} {}", head.method, head.uri);
    let mut response = request.send_body(body).await.map_err(|error| {
        anyhow!(
            "Failed to send HTTP request: {} {}: {error}",
            head.method,
            head.uri
        )
    })?;
    log::debug!("Received HTTP response: {}", response.status());
    let mut builder = Response::builder().status(response.status().as_u16());
    for (name, value) in response.headers() {
        builder = builder.header(
            name.as_str(),
            value
                .to_str()
                .with_context(|| format!("Invalid HTTP response header: {name}"))?,
        );
    }
    if let Some(timeout) = body_timeout {
        response = response.timeout(timeout);
    }
    let body = response
        .body()
        .await
        .with_context(|| format!("Failed to read HTTP response body from {}", head.uri))?;
    log::debug!("Received HTTP response body: {} bytes", body.len());
    Ok(builder.body(body)?)
}

struct NativeCertificates {
    certificates: Vec<rustls::pki_types::CertificateDer<'static>>,
    root_store: rustls::RootCertStore,
}

static NATIVE_CERTIFICATES: OnceLock<anyhow::Result<NativeCertificates>> = OnceLock::new();

pub fn make_http_client(config: &crate::app_config::AppConfig) -> anyhow::Result<awc::Client> {
    make_http_client_with_system_roots(config.system_root_ca_certificates)
}

pub(crate) fn default_system_root_ca_certificates_from_env() -> bool {
    std::env::var("SSL_CERT_FILE").is_ok_and(|value| !value.is_empty())
        || std::env::var("SSL_CERT_DIR").is_ok_and(|value| !value.is_empty())
}

fn native_certificates() -> anyhow::Result<&'static NativeCertificates> {
    NATIVE_CERTIFICATES
        .get_or_init(|| {
            log::debug!(
                "Loading native certificates because system_root_ca_certificates is enabled"
            );
            let CertificateResult {
                certs,
                errors,
                ..
            } = rustls_native_certs::load_native_certs();
            log::debug!("Loaded {} native TLS client certificates", certs.len());
            for error in errors {
                log::error!("Unable to load native certificate: {error}");
            }
            let mut root_store = rustls::RootCertStore::empty();
            for cert in &certs {
                log::trace!("Adding native certificate to root store: {cert:?}");
                root_store.add(cert.clone()).with_context(|| {
                    format!("Unable to add certificate to root store: {cert:?}")
                })?;
            }
            Ok(NativeCertificates {
                certificates: certs,
                root_store,
            })
        })
        .as_ref()
        .map_err(|error| {
            anyhow!(
                "Unable to load native certificates, make sure the system root CA certificates are available: {error}"
            )
        })
}

pub(crate) fn native_certificate_der()
-> anyhow::Result<&'static [rustls::pki_types::CertificateDer<'static>]> {
    Ok(&native_certificates()?.certificates)
}

pub(crate) fn make_http_client_with_system_roots(
    system_root_ca_certificates: bool,
) -> anyhow::Result<awc::Client> {
    let connector = if system_root_ca_certificates {
        let roots = &native_certificates()?.root_store;

        log::trace!(
            "Creating HTTP client with custom TLS connector using native certificates. SSL_CERT_FILE={:?}, SSL_CERT_DIR={:?}",
            std::env::var("SSL_CERT_FILE").unwrap_or_default(),
            std::env::var("SSL_CERT_DIR").unwrap_or_default()
        );

        let tls_conf = rustls::ClientConfig::builder()
            .with_root_certificates(roots.clone())
            .with_no_client_auth();

        awc::Connector::new().rustls_0_23(std::sync::Arc::new(tls_conf))
    } else {
        log::debug!(
            "Using the default tls connector with builtin certs because system_root_ca_certificates is disabled"
        );
        awc::Connector::new()
    };
    let client = awc::Client::builder()
        .connector(connector)
        .add_default_header((awc::http::header::USER_AGENT, env!("CARGO_PKG_NAME")))
        .finish();
    log::debug!("Created HTTP client");
    Ok(client)
}

pub(crate) fn get_http_client_from_appdata(
    request: &ServiceRequest,
) -> anyhow::Result<&awc::Client> {
    if let Some(result) = request.app_data::<anyhow::Result<awc::Client>>() {
        result
            .as_ref()
            .map_err(|e| anyhow!("HTTP client initialization failed: {e}"))
    } else {
        Err(anyhow!("HTTP client not found in app data"))
    }
}

#[cfg(test)]
mod transport_tests {
    use super::*;
    use openidconnect::{AsyncHttpClient, http::HeaderValue};
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;

    #[derive(Clone, Copy, Debug)]
    enum Adapter {
        Oidc,
        Otlp,
    }
    use Adapter::{Oidc, Otlp};

    impl Adapter {
        async fn request(
            self,
            url: &str,
            method: &str,
            header: &[u8],
        ) -> anyhow::Result<Response<Vec<u8>>> {
            let request = Request::builder()
                .uri(url)
                .method(method)
                .header("x-request", header)
                .body(b"request".to_vec())?;
            let client = awc::Client::default();
            match self {
                Oidc => super::super::oidc::AwcHttpClient::from_client(&client)
                    .call(request)
                    .await
                    .map_err(anyhow::Error::msg),
                Otlp => send_http_request(&client, request.map(Bytes::from), None)
                    .await
                    .map(|response| response.map(|body| body.to_vec())),
            }
        }
    }

    // Raw responses are needed for invalid header bytes, invalid chunks, and stalled bodies.
    async fn response_server(
        response: &'static [u8],
        hold_open: bool,
    ) -> (String, JoinHandle<Vec<u8>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/transport", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut socket = BufReader::new(socket);
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                assert!(socket.read_until(b'\n', &mut request).await.unwrap() > 0);
            }
            let length = String::from_utf8_lossy(&request)
                .lines()
                .filter_map(|line| line.split_once(':'))
                .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                .map_or(0, |(_, value)| value.trim().parse::<usize>().unwrap());
            let body_start = request.len();
            request.resize(body_start + length, 0);
            socket.read_exact(&mut request[body_start..]).await.unwrap();
            socket.get_mut().write_all(response).await.unwrap();
            if hold_open {
                std::future::pending::<()>().await;
            }
            request
        });
        (url, task)
    }

    #[actix_web::test]
    async fn preserves_methods_bodies_status_and_headers() -> anyhow::Result<()> {
        for adapter in [Oidc, Otlp] {
            for method in ["GET", "POST", "PATCH", "DELETE"] {
                let (url, server) = response_server(
                    b"HTTP/1.1 202 Accepted\r\nContent-Length: 8\r\nX-Result: first\r\nX-Result: second\r\n\r\nresponse", false,
                ).await;
                let header = match adapter {
                    Oidc => b"value".as_slice(),
                    Otlp => b"\xff".as_slice(),
                };
                let response = adapter.request(&url, method, header).await?;
                assert_eq!(response.status().as_u16(), 202);
                assert_eq!(response.body(), b"response");
                assert_eq!(response.headers().get_all("x-result").iter().count(), 2);
                let sent = server.await?;
                assert!(sent.starts_with(format!("{method} /transport HTTP/1.1\r\n").as_bytes()));
                let expected_header = [b"x-request: ".as_slice(), header, b"\r\n"].concat();
                assert!(
                    sent.windows(expected_header.len())
                        .any(|bytes| bytes == expected_header)
                );
                assert!(sent.ends_with(b"\r\n\r\nrequest"));
            }
        }
        Ok(())
    }

    #[actix_web::test]
    async fn rejects_non_text_headers_before_reading_body() -> anyhow::Result<()> {
        let error = Oidc
            .request("http://127.0.0.1:1/transport", "GET", b"\xff")
            .await
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            HeaderValue::from_bytes(b"\xff")?
                .to_str()
                .unwrap_err()
                .to_string()
        );
        for adapter in [Oidc, Otlp] {
            let (url, server) = response_server(
                b"HTTP/1.1 200 OK\r\nX-Invalid: \xff\r\nContent-Length: 100\r\n\r\n",
                true,
            )
            .await;
            let error = tokio::time::timeout(
                Duration::from_secs(1),
                adapter.request(&url, "GET", b"value"),
            )
            .await?
            .unwrap_err();
            assert!(
                error
                    .to_string()
                    .starts_with("Invalid HTTP response header: x-invalid")
            );
            server.abort();
        }
        Ok(())
    }

    #[actix_web::test]
    async fn retains_send_and_body_error_context() -> anyhow::Result<()> {
        for adapter in [Oidc, Otlp] {
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let unavailable = format!("http://{}/transport", listener.local_addr()?);
            drop(listener);
            let error = adapter
                .request(&unavailable, "GET", b"value")
                .await
                .unwrap_err();
            assert!(
                error
                    .to_string()
                    .starts_with("Failed to send HTTP request: GET ")
            );
            let (url, server) = response_server(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\ninvalid\r\n",
                false,
            )
            .await;
            let error = adapter.request(&url, "GET", b"value").await.unwrap_err();
            assert_eq!(
                error.to_string(),
                format!("Failed to read HTTP response body from {url}")
            );
            server.await?;
        }
        Ok(())
    }

    #[actix_web::test]
    async fn applies_body_timeout_after_validating_headers() {
        let (url, server) =
            response_server(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n", true).await;
        let error = send_http_request(
            &awc::Client::default(),
            Request::builder().uri(&url).body(Vec::<u8>::new()).unwrap(),
            Some(Duration::from_millis(20)),
        )
        .await
        .unwrap_err();
        assert!(error.downcast_ref::<awc::error::PayloadError>().is_some());
        server.abort();
    }
}
