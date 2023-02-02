use anyhow::Context;
use config::{Config, FileFormat};
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::net::{SocketAddr, ToSocketAddrs};

const DEFAULT_DATABASE_FILE: &str = "sqlpage.db";

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_database_url")]
    pub database_url: String,
    pub max_database_pool_connections: Option<u32>,
    pub database_connection_idle_timeout_seconds: Option<f64>,
    pub database_connection_max_lifetime_seconds: Option<f64>,

    #[serde(deserialize_with = "deserialize_socket_addr")]
    pub listen_on: SocketAddr,
    pub port: Option<u16>,
}

pub fn load() -> anyhow::Result<AppConfig> {
    let mut conf = Config::builder()
        .set_default("listen_on", "0.0.0.0:8080")?
        .add_source(config::Environment::default())
        .add_source(config::File::new("sqlpage/sqlpage.json", FileFormat::Json).required(false))
        .build()?
        .try_deserialize::<AppConfig>()
        .with_context(|| "Unable to load configuration")?;
    if let Some(port) = conf.port {
        conf.listen_on.set_port(port);
    }
    Ok(conf)
}

fn deserialize_socket_addr<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<SocketAddr, D::Error> {
    let host_str: String = Deserialize::deserialize(deserializer)?;
    parse_socket_addr(&host_str).map_err(D::Error::custom)
}

fn parse_socket_addr(host_str: &str) -> anyhow::Result<SocketAddr> {
    host_str
        .to_socket_addrs()?
        .next()
        .with_context(|| format!("host '{host_str}' does not resolve to an IP"))
}

fn default_database_url() -> String {
    let prefix = "sqlite://".to_owned();

    if cfg!(test) {
        return prefix + ":memory:";
    }

    #[cfg(not(feature = "lambda-web"))]
    if std::path::Path::new(DEFAULT_DATABASE_FILE).exists() {
        log::info!(
            "No DATABASE_URL, using the default sqlite database './{DEFAULT_DATABASE_FILE}'"
        );
        return prefix + DEFAULT_DATABASE_FILE;
    } else if let Ok(tmp_file) = std::fs::File::create(DEFAULT_DATABASE_FILE) {
        log::info!("No DATABASE_URL provided, the current directory is writeable, creating {DEFAULT_DATABASE_FILE}");
        drop(tmp_file);
        std::fs::remove_file(DEFAULT_DATABASE_FILE).expect("removing temp file");
        return prefix + DEFAULT_DATABASE_FILE + "?mode=rwc";
    }

    log::warn!("No DATABASE_URL provided, and the current directory is not writeable. Using a temporary in-memory SQLite database. All the data created will be lost when this server shuts down.");
    prefix + ":memory:"
}

#[cfg(test)]
pub(crate) mod tests {
    use super::AppConfig;
    use std::net::SocketAddr;

    pub fn test_config() -> AppConfig {
        AppConfig {
            database_url: "sqlite::memory:".to_string(),
            max_database_pool_connections: None,
            database_connection_idle_timeout_seconds: None,
            database_connection_max_lifetime_seconds: None,
            listen_on: SocketAddr::from(([127, 0, 0, 1], 8282)),
            port: None,
        }
    }
}
