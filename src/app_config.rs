use anyhow::Context;
use config::Config;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;

#[cfg(not(feature = "lambda-web"))]
const DEFAULT_DATABASE_FILE: &str = "sqlpage.db";

#[derive(Debug, Deserialize, PartialEq)]
pub struct AppConfig {
    #[serde(default = "default_database_url")]
    pub database_url: String,
    pub max_database_pool_connections: Option<u32>,
    pub database_connection_idle_timeout_seconds: Option<f64>,
    pub database_connection_max_lifetime_seconds: Option<f64>,

    #[serde(default)]
    pub sqlite_extensions: Vec<String>,

    #[serde(deserialize_with = "deserialize_socket_addr")]
    pub listen_on: SocketAddr,
    pub port: Option<u16>,

    /// Number of times to retry connecting to the database after a failure when the server starts
    /// up. Retries will happen every 5 seconds. The default is 6 retries, which means the server
    /// will wait up to 30 seconds for the database to become available.
    #[serde(default = "default_database_connection_retries")]
    pub database_connection_retries: u32,

    /// Maximum number of seconds to wait before giving up when acquiring a database connection from the
    /// pool. The default is 10 seconds.
    #[serde(default = "default_database_connection_acquire_timeout_seconds")]
    pub database_connection_acquire_timeout_seconds: f64,

    #[serde(default = "default_web_root")]
    pub web_root: PathBuf,
}

pub fn load() -> anyhow::Result<AppConfig> {
    let mut conf = Config::builder()
        .set_default("listen_on", "0.0.0.0:8080")?
        .add_source(config::File::with_name("sqlpage/sqlpage").required(false))
        .add_source(env_config())
        .add_source(env_config().prefix("SQLPAGE"))
        .build()?
        .try_deserialize::<AppConfig>()
        .with_context(|| "Unable to load configuration")?;
    if let Some(port) = conf.port {
        conf.listen_on.set_port(port);
    }
    Ok(conf)
}

fn env_config() -> config::Environment {
    config::Environment::default()
        .try_parsing(true)
        .list_separator(" ")
        .with_list_parse_key("sqlite_extensions")
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

fn default_database_connection_retries() -> u32 {
    6
}

fn default_database_connection_acquire_timeout_seconds() -> f64 {
    10.
}

fn default_web_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|e| {
        log::error!("Unable to get current directory: {}", e);
        PathBuf::from(&std::path::Component::CurDir)
    })
}

#[cfg(test)]
pub mod tests {
    use super::AppConfig;

    #[must_use]
    pub fn test_config() -> AppConfig {
        serde_json::from_str::<AppConfig>(
            r#"{
            "database_url": "sqlite::memory:",
            "listen_on": "localhost:8080"
        }"#,
        )
        .unwrap()
    }
}
