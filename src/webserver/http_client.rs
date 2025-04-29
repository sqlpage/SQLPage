use actix_web::dev::ServiceRequest;
use anyhow::{anyhow, Context};
use std::sync::OnceLock;

static NATIVE_CERTS: OnceLock<anyhow::Result<rustls::RootCertStore>> = OnceLock::new();

pub fn make_http_client(config: &crate::app_config::AppConfig) -> anyhow::Result<awc::Client> {
    let connector = if config.system_root_ca_certificates {
        let roots = NATIVE_CERTS
            .get_or_init(|| {
                log::debug!("Loading native certificates because system_root_ca_certificates is enabled");
                let certs = rustls_native_certs::load_native_certs()
                    .with_context(|| "Initial native certificates load failed")?;
                log::debug!("Loaded {} native HTTPS client certificates", certs.len());
                let mut roots = rustls::RootCertStore::empty();
                for cert in certs {
                    log::trace!("Adding native certificate to root store: {cert:?}");
                    roots.add(cert.clone()).with_context(|| {
                        format!("Unable to add certificate to root store: {cert:?}")
                    })?;
                }
                Ok(roots)
            })
            .as_ref()
            .map_err(|e| anyhow!("Unable to load native certificates, make sure the system root CA certificates are available: {e}"))?;

        log::trace!("Creating HTTP client with custom TLS connector using native certificates. SSL_CERT_FILE={:?}, SSL_CERT_DIR={:?}",
            std::env::var("SSL_CERT_FILE").unwrap_or_default(),
            std::env::var("SSL_CERT_DIR").unwrap_or_default());

        let tls_conf = rustls::ClientConfig::builder()
            .with_root_certificates(roots.clone())
            .with_no_client_auth();

        awc::Connector::new().rustls_0_22(std::sync::Arc::new(tls_conf))
    } else {
        log::debug!("Using the default tls connector with builtin certs because system_root_ca_certificates is disabled");
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
