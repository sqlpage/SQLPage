use crate::cli::arguments::{Cli, parse_cli};
use crate::webserver::content_security_policy::ContentSecurityPolicyTemplate;
use crate::webserver::routing::RoutingConfig;
use actix_web::http::Uri;
use anyhow::Context;
use config::Config;
use openidconnect::IssuerUrl;
use percent_encoding::AsciiSet;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

#[cfg(not(feature = "lambda-web"))]
const DEFAULT_DATABASE_FILE: &str = "sqlpage.db";

impl AppConfig {
    pub fn from_cli(cli: &Cli) -> anyhow::Result<Self> {
        let mut config = if let Some(config_file) = &cli.config_file {
            if !config_file.is_file() {
                return Err(anyhow::anyhow!(
                    "Configuration file does not exist: {}",
                    config_file.display()
                ));
            }
            log::debug!("Loading configuration from file: {}", config_file.display());
            load_from_file(config_file)?
        } else if let Some(config_dir) = &cli.config_dir {
            log::debug!(
                "Loading configuration from directory: {}",
                config_dir.display()
            );
            load_from_directory(config_dir)?
        } else {
            log::debug!("Loading configuration from environment");
            load_from_env()?
        };
        if let Some(web_root) = &cli.web_root {
            log::debug!(
                "Setting web root to value from the command line: {}",
                web_root.display()
            );
            config.web_root.clone_from(web_root);
        }
        if let Some(config_dir) = &cli.config_dir {
            config.configuration_directory.clone_from(config_dir);
        }

        config.configuration_directory = std::fs::canonicalize(&config.configuration_directory)
            .unwrap_or_else(|_| config.configuration_directory.clone());

        if !config.configuration_directory.exists() {
            log::info!(
                "Configuration directory does not exist, creating it: {}",
                config.configuration_directory.display()
            );
            std::fs::create_dir_all(&config.configuration_directory).with_context(|| {
                format!(
                    "Failed to create configuration directory in {}",
                    config.configuration_directory.display()
                )
            })?;
        }

        if config.database_url.is_empty() {
            log::debug!(
                "Creating default database in {}",
                config.configuration_directory.display()
            );
            config.database_url = create_default_database(&config.configuration_directory);
        }

        config
            .validate()
            .context("The provided configuration is invalid")?;

        config.resolve_timeouts();

        log::debug!("Loaded configuration: {config:#?}");
        log::info!(
            "Configuration loaded from {}",
            config.configuration_directory.display()
        );

        Ok(config)
    }

    fn resolve_timeouts(&mut self) {
        let is_sqlite = self.database_url.starts_with("sqlite:");
        self.database_connection_idle_timeout = resolve_timeout(
            self.database_connection_idle_timeout,
            if is_sqlite {
                None
            } else {
                Some(Duration::from_mins(30))
            },
        );
        self.database_connection_max_lifetime = resolve_timeout(
            self.database_connection_max_lifetime,
            if is_sqlite {
                None
            } else {
                Some(Duration::from_hours(1))
            },
        );
    }

    fn validate(&self) -> anyhow::Result<()> {
        if !self.web_root.is_dir() {
            return Err(anyhow::anyhow!(
                "Web root is not a valid directory: {}",
                self.web_root.display()
            ));
        }
        if !self.configuration_directory.is_dir() {
            return Err(anyhow::anyhow!(
                "Configuration directory is not a valid directory: {}",
                self.configuration_directory.display()
            ));
        }
        if self.database_connection_acquire_timeout_seconds <= 0.0 {
            return Err(anyhow::anyhow!(
                "Database connection acquire timeout must be positive"
            ));
        }
        if let Some(max_connections) = self.max_database_pool_connections
            && max_connections == 0
        {
            return Err(anyhow::anyhow!(
                "Maximum database pool connections must be greater than 0"
            ));
        }
        anyhow::ensure!(self.max_pending_rows > 0, "max_pending_rows cannot be null");

        for path in &self.oidc_protected_paths {
            if !path.starts_with('/') {
                return Err(anyhow::anyhow!(
                    "All protected paths must start with '/', but found: '{path}'"
                ));
            }
        }

        for path in &self.oidc_public_paths {
            if !path.starts_with('/') {
                return Err(anyhow::anyhow!(
                    "All public paths must start with '/', but found: '{path}'"
                ));
            }
        }

        Ok(())
    }
}

pub fn load_config() -> anyhow::Result<AppConfig> {
    let cli = parse_cli()?;
    AppConfig::from_cli(&cli)
}

pub fn load_from_env() -> anyhow::Result<AppConfig> {
    let config_dir = configuration_directory();
    load_from_directory(&config_dir)
        .with_context(|| format!("Unable to load configuration from {}", config_dir.display()))
}

include!(concat!(env!("OUT_DIR"), "/app_config.rs"));

impl AppConfig {
    #[must_use]
    pub fn cache_stale_duration_ms(&self) -> u64 {
        self.cache_stale_duration_ms
            .unwrap_or_else(|| if self.environment.is_prod() { 1000 } else { 0 })
    }

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

impl RoutingConfig for AppConfig {
    fn prefix(&self) -> &str {
        &self.site_prefix
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
    log::debug!("Loading configuration from {}", cannonical.display());
    let config_file = directory.join("sqlpage");
    let mut app_config = load_from_file(&config_file)?;
    app_config.configuration_directory = directory.into();
    Ok(app_config)
}

/// Parses and loads the configuration from the given file.
pub fn load_from_file(config_file: &Path) -> anyhow::Result<AppConfig> {
    log::debug!("Loading configuration from file: {}", config_file.display());
    let config = Config::builder()
        .add_source(config::File::from(config_file).required(false))
        .add_source(env_config())
        .add_source(env_config().prefix("SQLPAGE"))
        .build()
        .with_context(|| {
            format!(
                "Unable to build configuration loader for {}",
                config_file.display()
            )
        })?;
    log::trace!("Configuration sources: {:#?}", config.cache);
    let app_config = config
        .try_deserialize::<AppConfig>()
        .context("Failed to load the configuration")?;
    Ok(app_config)
}

fn env_config() -> config::Environment {
    config::Environment::default()
        .try_parsing(true)
        .list_separator(" ")
        .with_list_parse_key("sqlite_extensions")
}

fn deserialize_port<'de, D>(deserializer: D) -> Result<Option<u16>, D::Error>
where
    D: Deserializer<'de>,
{
    // deserializes both 8080 and "tcp://1.1.1.1:9090"
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PortOrUrl {
        Port(u16),
        Url(String),
    }
    let port_or_url: Option<PortOrUrl> = Deserialize::deserialize(deserializer)?;
    match port_or_url {
        Some(PortOrUrl::Port(p)) => Ok(Some(p)),
        Some(PortOrUrl::Url(u)) => {
            if let Ok(u) = Uri::from_str(&u) {
                log::warn!(
                    "{u} is not a valid value for the SQLPage port number. Ignoring this error since kubernetes may set the SQLPAGE_PORT env variable to a service URI when there is a service named sqlpage. Rename your service to avoid this warning."
                );
                Ok(None)
            } else {
                Err(D::Error::custom(format!(
                    "Invalid port number: {u}. Expected a number between {} and {}",
                    u16::MIN,
                    u16::MAX
                )))
            }
        }
        None => Ok(None),
    }
}

fn deserialize_socket_addr<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<SocketAddr>, D::Error> {
    let host_str: Option<String> = Deserialize::deserialize(deserializer)?;
    host_str
        .map(|h| {
            parse_socket_addr(&h).map_err(|e| {
                D::Error::custom(anyhow::anyhow!("Failed to parse socket address {h:?}: {e}"))
            })
        })
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
    const TO_ENCODE: AsciiSet = percent_encoding::CONTROLS
        .add(b' ')
        .add(b'"')
        .add(b'#')
        .add(b'<')
        .add(b'>')
        .add(b'?');

    let prefix = prefix.trim_start_matches('/').trim_end_matches('/');
    if prefix.is_empty() {
        return default_site_prefix();
    }
    let encoded_prefix = percent_encoding::percent_encode(prefix.as_bytes(), &TO_ENCODE);

    let invalid_chars = ["%09", "%0A", "%0D"];

    std::iter::once("/")
        .chain(encoded_prefix.filter(|c| !invalid_chars.contains(c)))
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
        .with_context(|| format!("Resolving host '{host_str}'"))
}

#[cfg(test)]
fn default_database_url() -> String {
    "sqlite://:memory:?cache=shared".to_owned()
}
#[cfg(not(test))]
fn default_database_url() -> String {
    // When using a custom configuration directory, the default database URL
    // will be set later in `AppConfig::from_cli`.
    String::new()
}

fn create_default_database(configuration_directory: &Path) -> String {
    let prefix = "sqlite://".to_owned();

    #[cfg(not(feature = "lambda-web"))]
    {
        let config_dir = cannonicalize_if_possible(configuration_directory);
        let old_default_db_path = PathBuf::from(DEFAULT_DATABASE_FILE);
        let default_db_path = config_dir.join(DEFAULT_DATABASE_FILE);
        if let Ok(true) = old_default_db_path.try_exists() {
            log::warn!(
                "Your sqlite database in {} is publicly accessible through your web server. Please move it to {}.",
                old_default_db_path.display(),
                default_db_path.display()
            );
            return prefix + old_default_db_path.to_str().unwrap();
        } else if let Ok(true) = default_db_path.try_exists() {
            log::debug!(
                "Using the default database file in {}",
                default_db_path.display()
            );
            return prefix + &encode_uri(&default_db_path);
        }
        // Create the default database file if we can
        if let Ok(tmp_file) = std::fs::File::create(&default_db_path) {
            log::info!(
                "No DATABASE_URL provided, {} is writable, creating a new database file.",
                default_db_path.display()
            );
            drop(tmp_file);
            if let Err(e) = std::fs::remove_file(&default_db_path) {
                log::debug!(
                    "Unable to remove temporary probe file. It might have already been removed by another instance started concurrently: {e}"
                );
            }
            return prefix + &encode_uri(&default_db_path) + "?mode=rwc";
        }
    }

    log::warn!(
        "No DATABASE_URL provided, and {} is not writeable. Using a temporary in-memory SQLite database. All the data created will be lost when this server shuts down.",
        configuration_directory.display()
    );
    prefix + ":memory:?cache=shared"
}

#[cfg(any(test, not(feature = "lambda-web")))]
fn encode_uri(path: &Path) -> std::borrow::Cow<'_, str> {
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
        log::error!("Unable to get current directory: {e}");
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

/// If the sending queue exceeds this number of outgoing messages, an error will be thrown
/// This prevents a single request from using up all available memory
fn default_max_pending_rows() -> usize {
    256
}

fn default_compress_responses() -> bool {
    false
}

fn default_system_root_ca_certificates() -> bool {
    std::env::var("SSL_CERT_FILE").is_ok_and(|x| !x.is_empty())
        || std::env::var("SSL_CERT_DIR").is_ok_and(|x| !x.is_empty())
}

fn default_max_recursion_depth() -> u8 {
    10
}

fn default_markdown_allow_dangerous_html() -> bool {
    false
}

fn default_markdown_allow_dangerous_protocol() -> bool {
    false
}

fn default_oidc_client_id() -> String {
    "sqlpage".to_string()
}

fn default_oidc_scopes() -> String {
    "openid email profile".to_string()
}

fn default_oidc_protected_paths() -> Vec<String> {
    vec!["/".to_string()]
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

fn deserialize_duration_seconds<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds: Option<f64> = Option::deserialize(deserializer)?;
    match seconds {
        None => Ok(None),
        Some(s) if s <= 0.0 || !s.is_finite() => Ok(Some(Duration::ZERO)),
        Some(s) => Ok(Some(Duration::from_secs_f64(s))),
    }
}

fn resolve_timeout(config_val: Option<Duration>, default: Option<Duration>) -> Option<Duration> {
    match config_val {
        Some(v) if v.is_zero() => None,
        Some(v) => Some(v),
        None => default,
    }
}

#[must_use]
pub fn test_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string())
}

#[cfg(test)]
pub mod tests {
    use super::AppConfig;
    pub use super::test_database_url;

    #[must_use]
    pub fn test_config() -> AppConfig {
        let mut config = serde_json::from_str::<AppConfig>(
            &serde_json::json!({
                "database_url": test_database_url(),
                "listen_on": "localhost:8080"
            })
            .to_string(),
        )
        .unwrap();
        config.resolve_timeouts();
        config
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
        assert_eq!(normalize_site_prefix("*-+/:;,?%\"'{"), "/*-+/:;,%3F%%22'{/");
        assert_eq!(
            normalize_site_prefix(&(0..=0x7F).map(char::from).collect::<String>()),
            "/%00%01%02%03%04%05%06%07%08%0B%0C%0E%0F%10%11%12%13%14%15%16%17%18%19%1A%1B%1C%1D%1E%1F%20!%22%23$%&'()*+,-./0123456789:;%3C=%3E%3F@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~%7F/"
        );
    }

    #[test]
    fn test_sqlpage_prefixed_env_variable_parsing() {
        let _lock = ENV_LOCK
            .lock()
            .expect("Another test panicked while holding the lock");
        unsafe {
            env::set_var("SQLPAGE_CONFIGURATION_DIRECTORY", "/path/to/config");
        }

        let config = load_from_env().unwrap();

        assert_eq!(
            config.configuration_directory,
            PathBuf::from("/path/to/config"),
            "Configuration directory should match the SQLPAGE_CONFIGURATION_DIRECTORY env var"
        );

        unsafe {
            env::remove_var("SQLPAGE_CONFIGURATION_DIRECTORY");
        }
    }

    #[test]
    fn test_k8s_env_var_ignored() {
        let _lock = ENV_LOCK
            .lock()
            .expect("Another test panicked while holding the lock");
        unsafe {
            env::set_var("SQLPAGE_PORT", "tcp://10.0.0.1:8080");
        }

        let config = load_from_env().unwrap();
        assert_eq!(config.port, None);

        unsafe {
            env::remove_var("SQLPAGE_PORT");
        }
    }

    #[test]
    fn test_valid_port_env_var() {
        let _lock = ENV_LOCK
            .lock()
            .expect("Another test panicked while holding the lock");
        unsafe {
            env::set_var("SQLPAGE_PORT", "9000");
        }

        let config = load_from_env().unwrap();
        assert_eq!(config.port, Some(9000));

        unsafe {
            env::remove_var("SQLPAGE_PORT");
        }
    }

    #[test]
    fn test_config_priority() {
        let _lock = ENV_LOCK
            .lock()
            .expect("Another test panicked while holding the lock");
        unsafe {
            env::set_var("SQLPAGE_WEB_ROOT", "/");
        }

        let cli = Cli {
            web_root: Some(PathBuf::from(".")),
            config_dir: None,
            config_file: None,
            command: None,
        };

        let config = AppConfig::from_cli(&cli).unwrap();

        assert_eq!(
            config.web_root,
            PathBuf::from("."),
            "CLI argument should take precedence over environment variable"
        );

        unsafe {
            env::remove_var("SQLPAGE_WEB_ROOT");
        }
    }

    #[test]
    fn test_config_file_priority() {
        let _lock = ENV_LOCK
            .lock()
            .expect("Another test panicked while holding the lock");
        let temp_dir = std::env::temp_dir().join("sqlpage_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let config_file_path = temp_dir.join("sqlpage.json");
        let config_web_dir = temp_dir.join("config/web");
        let env_web_dir = temp_dir.join("env/web");
        let cli_web_dir = temp_dir.join("cli/web");
        std::fs::create_dir_all(&config_web_dir).unwrap();
        std::fs::create_dir_all(&env_web_dir).unwrap();
        std::fs::create_dir_all(&cli_web_dir).unwrap();

        let config_content = serde_json::json!({
            "web_root": config_web_dir.to_str().unwrap()
        })
        .to_string();
        std::fs::write(&config_file_path, config_content).unwrap();

        unsafe {
            env::set_var("SQLPAGE_WEB_ROOT", env_web_dir.to_str().unwrap());
        }

        let cli = Cli {
            web_root: None,
            config_dir: None,
            config_file: Some(config_file_path.clone()),
            command: None,
        };

        let config = AppConfig::from_cli(&cli).unwrap();

        assert_eq!(
            config.web_root, env_web_dir,
            "Environment variable should override config file"
        );
        assert_eq!(
            config.configuration_directory,
            cannonicalize_if_possible(&PathBuf::from("./sqlpage")),
            "Configuration directory should be default when not overridden"
        );

        let cli_with_web_root = Cli {
            web_root: Some(cli_web_dir.clone()),
            config_dir: None,
            config_file: Some(config_file_path),
            command: None,
        };

        let config = AppConfig::from_cli(&cli_with_web_root).unwrap();
        assert_eq!(
            config.web_root, cli_web_dir,
            "CLI argument should take precedence over environment variable and config file"
        );
        assert_eq!(
            config.configuration_directory,
            cannonicalize_if_possible(&PathBuf::from("./sqlpage")),
            "Configuration directory should remain unchanged"
        );

        unsafe {
            env::remove_var("SQLPAGE_WEB_ROOT");
        }
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn example_json_configuration_deserializes() {
        serde_json::from_str::<AppConfig>(include_str!("../sqlpage/sqlpage.json")).unwrap();
    }

    #[test]
    fn test_default_values() {
        let _lock = ENV_LOCK
            .lock()
            .expect("Another test panicked while holding the lock");
        unsafe {
            env::remove_var("SQLPAGE_CONFIGURATION_DIRECTORY");
        }
        unsafe {
            env::remove_var("SQLPAGE_WEB_ROOT");
        }

        let cli = Cli {
            web_root: None,
            config_dir: None,
            config_file: None,
            command: None,
        };

        let config = AppConfig::from_cli(&cli).unwrap();

        assert_eq!(
            config.web_root,
            default_web_root(),
            "Web root should default to current directory when not specified"
        );
        assert_eq!(
            config.configuration_directory,
            cannonicalize_if_possible(&PathBuf::from("./sqlpage")),
            "Configuration directory should default to ./sqlpage when not specified"
        );
    }
}
