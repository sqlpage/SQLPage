use rustls_acme::{caches::DirCache, futures_rustls::rustls::ServerConfig, AcmeConfig};
use tokio_stream::StreamExt;

use crate::app_config::AppConfig;

pub fn make_auto_rustls_config(domain: &str, config: &AppConfig) -> ServerConfig {
    log::info!("Starting HTTPS configuration for {domain}");
    let mut state = AcmeConfig::new([domain])
        .contact([if let Some(email) = &config.https_certificate_email {
            format!("mailto:{}", email.as_str())
        } else {
            format!("mailto:contact@{domain}")
        }])
        .cache_option(Some(DirCache::new(
            config.https_certificate_cache_dir.clone(),
        )))
        .directory(&config.https_acme_directory_url)
        .state();
    let rustls_config = state.challenge_rustls_config();

    tokio::spawn(async move {
        loop {
            match state.next().await.unwrap() {
                Ok(ok) => log::info!("ACME configuration event: {ok:?}"),
                Err(err) => log::error!("ACME configuration error: {err:?}"),
            }
        }
    });

    ServerConfig::clone(&rustls_config)
}
