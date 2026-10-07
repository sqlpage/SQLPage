use actix_web::dev::ServiceRequest;
use anyhow::{Context, anyhow};
use rustls_native_certs::CertificateResult;
use std::sync::OnceLock;
use std::time::Duration;

/// The stage that failed while buffering an outbound HTTP response.
#[derive(Debug)]
pub(crate) enum BufferedRequestError {
    Send(awc::error::SendRequestError),
    ResponseHead(anyhow::Error),
    Body(awc::error::PayloadError),
}

/// Sends a prepared request and validates the response head before reading its body.
/// Callers retain their header policy, timeout selection, and error context.
pub(crate) async fn send_buffered_request<B, H>(
    request: awc::ClientRequest,
    body: B,
    body_timeout: Option<Duration>,
    prepare_response_head: impl FnOnce(
        awc::http::StatusCode,
        &awc::http::header::HeaderMap,
    ) -> anyhow::Result<H>,
) -> Result<(H, actix_web::web::Bytes), BufferedRequestError>
where
    B: actix_web::body::MessageBody + 'static,
{
    let response = request
        .send_body(body)
        .await
        .map_err(BufferedRequestError::Send)?;
    let head = prepare_response_head(response.status(), response.headers())
        .map_err(BufferedRequestError::ResponseHead)?;
    let mut response = match body_timeout {
        Some(timeout) => response.timeout(timeout),
        None => response,
    };
    let body = response.body().await.map_err(BufferedRequestError::Body)?;
    Ok((head, body))
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
pub(crate) mod transport_tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;

    pub(crate) async fn response_server(
        response: &'static [u8],
        hold_open: bool,
    ) -> (String, JoinHandle<Vec<u8>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/transport", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let (header_end, content_length) = loop {
                let mut buffer = [0; 1024];
                let count = socket.read(&mut buffer).await.unwrap();
                assert!(count > 0, "request ended before its headers");
                request.extend_from_slice(&buffer[..count]);
                if let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                    let length = String::from_utf8_lossy(&request[..end])
                        .lines()
                        .filter_map(|line| line.split_once(':'))
                        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                        .map_or(0, |(_, value)| value.trim().parse::<usize>().unwrap());
                    break (end + 4, length);
                }
            };
            while request.len() < header_end + content_length {
                let mut buffer = [0; 1024];
                let count = socket.read(&mut buffer).await.unwrap();
                assert!(count > 0, "request ended before its body");
                request.extend_from_slice(&buffer[..count]);
            }
            socket.write_all(response).await.unwrap();
            if hold_open {
                std::future::pending::<()>().await;
            }
            request
        });
        (url, task)
    }

    pub(crate) async fn unavailable_url() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        format!("http://{}/transport", listener.local_addr().unwrap())
    }

    #[actix_web::test]
    async fn applies_body_timeout_after_validating_headers() {
        let (url, server) =
            response_server(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n", true).await;
        let error = send_buffered_request(
            awc::Client::default().get(url),
            Vec::<u8>::new(),
            Some(Duration::from_millis(20)),
            |status, _| {
                assert_eq!(status, awc::http::StatusCode::OK);
                Ok(())
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(error, BufferedRequestError::Body(_)));
        server.abort();
    }
}
