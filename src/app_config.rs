use anyhow::Context;
use clap::Parser;
use config::Config;
use percent_encoding::AsciiSet;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// The directory where the .sql files are located.
    #[clap(short, long)]
    pub web_root: Option<PathBuf>,
    /// The directory where the sqlpage.json configuration, the templates, and the migrations are located.
    #[clap(short, long)]
    pub config_dir: Option<PathBuf>,
}

#[cfg(not(feature = "lambda-web"))]
const DEFAULT_DATABASE_FILE: &str = "sqlpage.db";

impl AppConfig {
    pub fn from_cli(cli: &Cli) -> anyhow::Result<Self> {
        let mut config = if let Some(config_dir) = &cli.config_dir {
            load_from_directory(config_dir)?
        } else {
            load_from_env()?
        };
        if let Some(web_root) = &cli.web_root {
            config.web_root.clone_from(web_root);
        }
        Ok(config)
    }
}

pub fn load_from_cli() -> anyhow::Result<AppConfig> {
    let cli = Cli::parse();
    AppConfig::from_cli(&cli)
}

pub fn load_from_env() -> anyhow::Result<AppConfig> {
    let config_dir = configuration_directory();
    load_from_directory(&config_dir)
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct AppConfig {
    #[serde(default = "default_database_url")]
    pub database_url: String,
    pub max_database_pool_connections: Option<u32>,
    pub database_connection_idle_timeout_seconds: Option<f64>,
    pub database_connection_max_lifetime_seconds: Option<f64>,

    #[serde(default)]
    pub sqlite_extensions: Vec<String>,

    #[serde(default, deserialize_with = "deserialize_socket_addr")]
    pub listen_on: Option<SocketAddr>,
    pub port: Option<u16>,
    pub unix_socket: Option<PathBuf>,

    /// Number of times to retry connecting to the database after a failure when the server starts
    /// up. Retries will happen every 5 seconds. The default is 6 retries, which means the server
    /// will wait up to 30 seconds for the database to become available.
    #[serde(default = "default_database_connection_retries")]
    pub database_connection_retries: u32,

    /// Maximum number of seconds to wait before giving up when acquiring a database connection from the
    /// pool. The default is 10 seconds.
    #[serde(default = "default_database_connection_acquire_timeout_seconds")]
    pub database_connection_acquire_timeout_seconds: f64,

    /// The directory where the .sql files are located. Defaults to the current directory.
    #[serde(default = "default_web_root")]
    pub web_root: PathBuf,

    /// The directory where the sqlpage configuration file is located. Defaults to `./sqlpage`.
    #[serde(default = "configuration_directory")]
    pub configuration_directory: PathBuf,

    /// Set to true to allow the `sqlpage.exec` function to be used in SQL queries.
    /// This should be enabled only if you trust the users writing SQL queries, since it gives
    /// them the ability to execute arbitrary shell commands on the server.
    #[serde(default)]
    pub allow_exec: bool,

    /// Maximum size of uploaded files in bytes. The default is 10MiB (10 * 1024 * 1024 bytes)
    #[serde(default = "default_max_file_size")]
    pub max_uploaded_file_size: usize,

    /// A domain name to use for the HTTPS server. If this is set, the server will perform all the necessary
    /// steps to set up an HTTPS server automatically. All you need to do is point your domain name to the
    /// server's IP address.
    ///
    /// It will listen on port 443 for HTTPS connections,
    /// and will automatically request a certificate from Let's Encrypt
    /// using the ACME protocol (requesting a TLS-ALPN-01 challenge).
    pub https_domain: Option<String>,

    /// The email address to use when requesting a certificate from Let's Encrypt.
    /// Defaults to `contact@<https_domain>`.
    pub https_certificate_email: Option<String>,

    /// The directory to store the Let's Encrypt certificate in. Defaults to `./sqlpage/https`.
    #[serde(default = "default_https_certificate_cache_dir")]
    pub https_certificate_cache_dir: PathBuf,

    /// URL to the ACME directory. Defaults to the Let's Encrypt production directory.
    #[serde(default = "default_https_acme_directory_url")]
    pub https_acme_directory_url: String,

    /// Whether we should run in development or production mode. Used to determine
    /// whether to show error messages to the user.
    #[serde(default)]
    pub environment: DevOrProd,

    /// Serve the website from a sub path. For example, if you set this to `/sqlpage/`, the website will be
    /// served from `https://yourdomain.com/sqlpage/`. Defaults to `/`.
    /// This is useful if you want to serve the website on the same domain as other content, and
    /// you are using a reverse proxy to route requests to the correct server.
    #[serde(
        deserialize_with = "deserialize_site_prefix",
        default = "default_site_prefix"
    )]
    pub site_prefix: String,

    /// Maximum number of messages that can be stored in memory before sending them to the client.
    #[serde(default = "default_max_pending_rows")]
    pub max_pending_rows: usize,

    /// Whether to compress the http response body when the client supports it.
    #[serde(default = "default_compress_responses")]
    pub compress_responses: bool,

    /// Content-Security-Policy header to send to the client. If not set, a default policy allowing scripts from the same origin is used and from jsdelivr.net
    pub content_security_policy: Option<String>,

    /// Whether `sqlpage.fetch` should load trusted certificates from the operating system's certificate store
    /// By default, it loads Mozilla's root certificates that are embedded in the `SQLPage` binary, or the ones pointed to by the
    /// `SSL_CERT_FILE` and `SSL_CERT_DIR` environment variables.
    #[serde(default = "default_system_root_ca_certificates")]
    pub system_root_ca_certificates: bool,
}

impl AppConfig {
    #[must_use]
    pub fn listen_on(&self) -> SocketAddr {
        let mut addr = self.listen_on.unwrap_or_else(|| {
            if self.https_domain.is_some() {
                SocketAddr::from(([0, 0, 0, 0], 443))
            } else {
                SocketAddr::from(([0, 0, 0, 0], 8080))
            }
        });
        if let Some(port) = self.port {
            addr.set_port(port);
        }
        addr
    }
}

/// The directory where the `sqlpage.json` file is located.
/// Determined by the `SQLPAGE_CONFIGURATION_DIRECTORY` environment variable
fn configuration_directory() -> PathBuf {
    let env_var_name = "CONFIGURATION_DIRECTORY";
    // uppercase or lowercase, with or without the "SQLPAGE_" prefix
    for prefix in &["", "SQLPAGE_"] {
        let var = format!("{prefix}{env_var_name}");
        for t in [str::to_lowercase, str::to_uppercase] {
            let dir = t(&var);
            if let Ok(dir) = std::env::var(dir) {
                return PathBuf::from(dir);
            }
        }
    }
    PathBuf::from("./sqlpage")
}

fn cannonicalize_if_possible(path: &std::path::Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_owned())
}

/// Parses and loads the configuration from the `sqlpage.json` file in the current directory.
/// This should be called only once at the start of the program.
pub fn load_from_directory(directory: &Path) -> anyhow::Result<AppConfig> {
    let cannonical = cannonicalize_if_possible(directory);
    log::debug!("Loading configuration from {:?}", cannonical);
    let config_file = directory.join("sqlpage");
    let mut app_config = load_from_file(&config_file)?;
    app_config.configuration_directory = directory.into();
    Ok(app_config)
}

/// Parses and loads the configuration from the given file.
pub fn load_from_file(config_file: &Path) -> anyhow::Result<AppConfig> {
    let config = Config::builder()
        .add_source(config::File::from(config_file).required(false))
        .add_source(env_config())
        .add_source(env_config().prefix("SQLPAGE"))
        .build()?;
    log::trace!("Configuration sources: {}", config.cache);
    let app_config = config
        .try_deserialize::<AppConfig>()
        .with_context(|| "Unable to load configuration")?;
    log::debug!("Loaded configuration: {:#?}", app_config);
    Ok(app_config)
}

fn env_config() -> config::Environment {
    config::Environment::default()
        .try_parsing(true)
        .list_separator(" ")
        .with_list_parse_key("sqlite_extensions")
}

fn deserialize_socket_addr<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<SocketAddr>, D::Error> {
    let host_str: Option<String> = Deserialize::deserialize(deserializer)?;
    host_str
        .map(|h| parse_socket_addr(&h).map_err(D::Error::custom))
        .transpose()
}

fn deserialize_site_prefix<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let prefix: String = Deserialize::deserialize(deserializer)?;
    Ok(normalize_site_prefix(prefix.as_str()))
}

/// We standardize the site prefix to always be stored with both leading and trailing slashes.
/// We also percent-encode special characters in the prefix, but allow it to contain slashes (to allow
/// hosting on a sub-sub-path).
fn normalize_site_prefix(prefix: &str) -> String {
    const TO_ENCODE: AsciiSet = percent_encoding::NON_ALPHANUMERIC.remove(b'/');

    let prefix = prefix.trim_start_matches('/').trim_end_matches('/');
    if prefix.is_empty() {
        return default_site_prefix();
    }
    let encoded_prefix = percent_encoding::percent_encode(prefix.as_bytes(), &TO_ENCODE);

    std::iter::once("/")
        .chain(encoded_prefix)
        .chain(std::iter::once("/"))
        .collect::<String>()
}

#[test]
fn test_normalize_site_prefix() {
    assert_eq!(normalize_site_prefix(""), "/");
    assert_eq!(normalize_site_prefix("/"), "/");
    assert_eq!(normalize_site_prefix("a"), "/a/");
    assert_eq!(normalize_site_prefix("a/"), "/a/");
    assert_eq!(normalize_site_prefix("/a"), "/a/");
    assert_eq!(normalize_site_prefix("a/b"), "/a/b/");
    assert_eq!(normalize_site_prefix("a/b/"), "/a/b/");
    assert_eq!(normalize_site_prefix("a/b/c"), "/a/b/c/");
    assert_eq!(normalize_site_prefix("a b"), "/a%20b/");
    assert_eq!(normalize_site_prefix("a b/c"), "/a%20b/c/");
}

fn default_site_prefix() -> String {
    '/'.to_string()
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
    {
        let config_dir = cannonicalize_if_possible(&configuration_directory());
        let old_default_db_path = PathBuf::from(DEFAULT_DATABASE_FILE);
        let default_db_path = config_dir.join(DEFAULT_DATABASE_FILE);
        if let Ok(true) = old_default_db_path.try_exists() {
            log::warn!("Your sqlite database in {} is publicly accessible through your web server. Please move it to {}.", old_default_db_path.display(), default_db_path.display());
            return prefix + old_default_db_path.to_str().unwrap();
        } else if let Ok(true) = default_db_path.try_exists() {
            log::debug!(
                "Using the default database file in {}",
                default_db_path.display()
            );
            return prefix + &encode_uri(&default_db_path);
        }
        // Create the default database file if we can
        let _ = std::fs::create_dir_all(default_db_path.parent().unwrap()); // may already exist
        if let Ok(tmp_file) = std::fs::File::create(&default_db_path) {
            log::info!(
                "No DATABASE_URL provided, {} is writable, creating a new database file.",
                default_db_path.display()
            );
            drop(tmp_file);
            std::fs::remove_file(&default_db_path).expect("removing temp file");
            return prefix + &encode_uri(&default_db_path) + "?mode=rwc";
        }
    }

    log::warn!("No DATABASE_URL provided, and the current directory is not writeable. Using a temporary in-memory SQLite database. All the data created will be lost when this server shuts down.");
    prefix + ":memory:"
}

fn encode_uri(path: &Path) -> std::borrow::Cow<str> {
    const ASCII_SET: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'_')
        .remove(b'.')
        .remove(b':')
        .remove(b' ')
        .remove(b'/');
    let path_bytes = path.as_os_str().as_encoded_bytes();
    percent_encoding::percent_encode(path_bytes, ASCII_SET).into()
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

fn default_max_file_size() -> usize {
    5 * 1024 * 1024
}

fn default_https_certificate_cache_dir() -> PathBuf {
    default_web_root().join("sqlpage").join("https")
}

fn default_https_acme_directory_url() -> String {
    "https://acme-v02.api.letsencrypt.org/directory".to_string()
}

fn default_max_pending_rows() -> usize {
    256
}

fn default_compress_responses() -> bool {
    true
}

fn default_system_root_ca_certificates() -> bool {
    std::env::var("SSL_CERT_FILE").is_ok_and(|x| !x.is_empty())
        || std::env::var("SSL_CERT_DIR").is_ok_and(|x| !x.is_empty())
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DevOrProd {
    #[default]
    Development,
    Production,
}
impl DevOrProd {
    pub(crate) fn is_prod(self) -> bool {
        self == DevOrProd::Production
    }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_site_prefix() {
        assert_eq!(default_site_prefix(), "/".to_string());
    }

    #[test]
    fn test_encode_uri() {
        assert_eq!(
            encode_uri(Path::new("/hello world/xxx.db")),
            "/hello world/xxx.db"
        );
        assert_eq!(encode_uri(Path::new("é")), "%C3%A9");
        assert_eq!(encode_uri(Path::new("/a?b/c")), "/a%3Fb/c");
    }

    #[test]
    fn test_normalize_site_prefix() {
        assert_eq!(normalize_site_prefix(""), "/");
        assert_eq!(normalize_site_prefix("/"), "/");
        assert_eq!(normalize_site_prefix("a"), "/a/");
        assert_eq!(normalize_site_prefix("a/"), "/a/");
        assert_eq!(normalize_site_prefix("/a"), "/a/");
        assert_eq!(normalize_site_prefix("a/b"), "/a/b/");
        assert_eq!(normalize_site_prefix("a/b/"), "/a/b/");
        assert_eq!(normalize_site_prefix("a/b/c"), "/a/b/c/");
        assert_eq!(normalize_site_prefix("a b"), "/a%20b/");
        assert_eq!(normalize_site_prefix("a b/c"), "/a%20b/c/");
    }

    #[test]
    fn test_cli() {
        let cli = Cli::parse_from(&[
            "sqlpage",
            "--web-root",
            "/path/to/web",
            "--config-dir",
            "custom_config",
        ]);
        assert_eq!(cli.web_root, Some(PathBuf::from("/path/to/web")));
        assert_eq!(cli.config_dir, Some(PathBuf::from("custom_config")));
    }

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
