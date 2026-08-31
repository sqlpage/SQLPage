use actix_web::dev::ServiceRequest;
use anyhow::{Context, anyhow};
use rustls_native_certs::CertificateResult;
use std::sync::OnceLock;

struct NativeCertificates {
    certificates: Vec<rustls::pki_types::CertificateDer<'static>>,
    root_store: rustls::RootCertStore,
}

static NATIVE_CERTIFICATES: OnceLock<anyhow::Result<NativeCertificates>> = OnceLock::new();

pub fn make_http_client(config: &crate::app_config::AppConfig) -> anyhow::Result<awc::Client> {
    make_http_client_with_system_roots(config.system_root_ca_certificates)
}

pub(crate) fn default_system_root_ca_certificates_from_env() -> bool {
    system_roots_selected_by(|name| std::env::var(name).ok())
}

fn system_roots_selected_by(lookup: impl Fn(&str) -> Option<String>) -> bool {
    ["SSL_CERT_FILE", "SSL_CERT_DIR"]
        .into_iter()
        .any(|name| lookup(name).is_some_and(|value| !value.is_empty()))
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
mod tests {
    use super::system_roots_selected_by;

    #[test]
    fn either_ssl_cert_variable_selects_the_system_trust_store() {
        for variable in ["SSL_CERT_FILE", "SSL_CERT_DIR"] {
            assert!(
                system_roots_selected_by(
                    |name| (name == variable).then(|| "/etc/ssl/certs".to_owned())
                ),
                "{variable}"
            );
        }
    }

    #[test]
    fn unset_or_empty_ssl_cert_variables_leave_the_bundled_roots() {
        assert!(!system_roots_selected_by(|_| None));
        assert!(!system_roots_selected_by(|_| Some(String::new())));
        assert!(!system_roots_selected_by(
            |name| (name == "SSL_CERT_PATH").then(|| "/etc/ssl/certs".to_owned())
        ));
    }
}
