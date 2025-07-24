use crate::webserver::content_security_policy::ContentSecurityPolicyTemplate;
use crate::webserver::routing::RoutingConfig;
use anyhow::Context;
use clap::Parser;
use config::Config;
use openidconnect::IssuerUrl;
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
    #[clap(short = 'd', long)]
    pub config_dir: Option<PathBuf>,
    /// The path to the configuration file.
    #[clap(short = 'c', long)]
    pub config_file: Option<PathBuf>,

    /// Subcommands for additional functionality.
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

/// Enum for subcommands.
#[derive(Parser)]
pub enum Commands {
    /// Create a new migration file.
    CreateMigration {
        /// Name of the migration.
        migration_name: String,
    },
}

#[cfg(not(feature = "lambda-web"))]
const DEFAULT_DATABASE_FILE: &str = "sqlpage.db";

impl AppConfig {
    pub fn from_cli(cli: &Cli) -> anyhow::Result<Self> {
        let mut config = if let Some(config_file) = &cli.config_file {
            if !config_file.is_file() {
                return Err(anyhow::anyhow!(
                    "Configuration file does not exist: {:?}",
                    config_file
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

        log::debug!("Loaded configuration: {config:#?}");
        log::info!(
            "Configuration loaded from {}",
            config.configuration_directory.display()
        );

        Ok(config)
    }

    fn validate(&self) -> anyhow::Result<()> {
        if !self.web_root.is_dir() {
            return Err(anyhow::anyhow!(
                "Web root is not a valid directory: {:?}",
                self.web_root
            ));
        }
        if !self.configuration_directory.is_dir() {
            return Err(anyhow::anyhow!(
                "Configuration directory is not a valid directory: {:?}",
                self.configuration_directory
            ));
        }
        if self.database_connection_acquire_timeout_seconds <= 0.0 {
            return Err(anyhow::anyhow!(
                "Database connection acquire timeout must be positive"
            ));
        }
        if let Some(max_connections) = self.max_database_pool_connections {
            if max_connections == 0 {
                return Err(anyhow::anyhow!(
                    "Maximum database pool connections must be greater than 0"
                ));
            }
        }
        if let Some(idle_timeout) = self.database_connection_idle_timeout_seconds {
            if idle_timeout < 0.0 {
                return Err(anyhow::anyhow!(
                    "Database connection idle timeout must be non-negative"
                ));
            }
        }
        if let Some(max_lifetime) = self.database_connection_max_lifetime_seconds {
            if max_lifetime < 0.0 {
                return Err(anyhow::anyhow!(
                    "Database connection max lifetime must be non-negative"
                ));
            }
        }
        anyhow::ensure!(self.max_pending_rows > 0, "max_pending_rows cannot be null");
        Ok(())
    }
}

pub fn load_from_cli() -> anyhow::Result<AppConfig> {
    let cli = Cli::parse();
    AppConfig::from_cli(&cli)
}

pub fn load_from_env() -> anyhow::Result<AppConfig> {
    let config_dir = configuration_directory();
    load_from_directory(&config_dir)
        .with_context(|| format!("Unable to load configuration from {}", config_dir.display()))
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct AppConfig {
    #[serde(default = "default_database_url")]
    pub database_url: String,
    /// A separate field for the database password. If set, this will override any password specified in the `database_url`.
    #[serde(default)]
    pub database_password: Option<String>,
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

    /// The base URL of the `OpenID` Connect provider.
    /// Required when enabling Single Sign-On through an OIDC provider.
    pub oidc_issuer_url: Option<IssuerUrl>,
    /// The client ID assigned to `SQLPage` when registering with the OIDC provider.
    /// Defaults to `sqlpage`.
    #[serde(default = "default_oidc_client_id")]
    pub oidc_client_id: String,
    /// The client secret for authenticating `SQLPage` to the OIDC provider.
    /// Required when enabling Single Sign-On through an OIDC provider.
    pub oidc_client_secret: Option<String>,
    /// Space-separated list of [scopes](https://openid.net/specs/openid-connect-core-1_0.html#ScopeClaims) to request during OIDC authentication.
    /// Defaults to "openid email profile"
    #[serde(default = "default_oidc_scopes")]
    pub oidc_scopes: String,

    pub oidc_skip_endpoints: Vec<String>,

    /// A domain name to use for the HTTPS server. If this is set, the server will perform all the necessary
    /// steps to set up an HTTPS server automatically. All you need to do is point your domain name to the
    /// server's IP address.
    ///
    /// It will listen on port 443 for HTTPS connections,
    /// and will automatically request a certificate from Let's Encrypt
    /// using the ACME protocol (requesting a TLS-ALPN-01 challenge).
    pub https_domain: Option<String>,

    /// The hostname where your application is publicly accessible (e.g., "myapp.example.com").
    /// This is used for OIDC redirect URLs. If not set, `https_domain` will be used instead.
    pub host: Option<String>,

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
    /// This prevents a single request from using up all available memory.
    #[serde(default = "default_max_pending_rows")]
    pub max_pending_rows: usize,

    /// Whether to compress the http response body when the client supports it.
    #[serde(default = "default_compress_responses")]
    pub compress_responses: bool,

    /// Content-Security-Policy header to send to the client.
    /// If not set, a default policy allowing
    ///  - scripts from the same origin,
    ///  - script elements with the `nonce="{{@csp_nonce}}"` attribute,
    #[serde(default)]
    pub content_security_policy: ContentSecurityPolicyTemplate,

    /// Whether `sqlpage.fetch` should load trusted certificates from the operating system's certificate store
    /// By default, it loads Mozilla's root certificates that are embedded in the `SQLPage` binary, or the ones pointed to by the
    /// `SSL_CERT_FILE` and `SSL_CERT_DIR` environment variables.
    #[serde(default = "default_system_root_ca_certificates")]
    pub system_root_ca_certificates: bool,

    /// Maximum depth of recursion allowed in the `run_sql` function.
    #[serde(default = "default_max_recursion_depth")]
    pub max_recursion_depth: u8,

    #[serde(default = "default_markdown_allow_dangerous_html")]
    pub markdown_allow_dangerous_html: bool,

    #[serde(default = "default_markdown_allow_dangerous_protocol")]
    pub markdown_allow_dangerous_protocol: bool,
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

    log::warn!("No DATABASE_URL provided, and {} is not writeable. Using a temporary in-memory SQLite database. All the data created will be lost when this server shuts down.", configuration_directory.display());
    prefix + ":memory:?cache=shared"
}

#[cfg(any(test, not(feature = "lambda-web")))]
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
    true
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

#[must_use]
pub fn test_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string())
}

#[cfg(test)]
pub mod tests {
    pub use super::test_database_url;
    use super::AppConfig;

    #[must_use]
    pub fn test_config() -> AppConfig {
        serde_json::from_str::<AppConfig>(
            &serde_json::json!({
                "database_url": test_database_url(),
                "listen_on": "localhost:8080"
            })
            .to_string(),
        )
        .unwrap()
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
            normalize_site_prefix(
                &(0..=0x7F).map(char::from).collect::<String>()
            ),
            "/%00%01%02%03%04%05%06%07%08%0B%0C%0E%0F%10%11%12%13%14%15%16%17%18%19%1A%1B%1C%1D%1E%1F%20!%22%23$%&'()*+,-./0123456789:;%3C=%3E%3F@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~%7F/"
        );
    }

    #[test]
    fn test_cli_argument_parsing() {
        let cli = Cli::parse_from([
            "sqlpage",
            "--web-root",
            "/path/to/web",
            "--config-dir",
            "/path/to/config",
            "--config-file",
            "/path/to/config.json",
        ]);

        assert_eq!(cli.web_root, Some(PathBuf::from("/path/to/web")));
        assert_eq!(cli.config_dir, Some(PathBuf::from("/path/to/config")));
        assert_eq!(cli.config_file, Some(PathBuf::from("/path/to/config.json")));
    }

    #[test]
    fn test_sqlpage_prefixed_env_variable_parsing() {
        let _lock = ENV_LOCK
            .lock()
            .expect("Another test panicked while holding the lock");
        env::set_var("SQLPAGE_CONFIGURATION_DIRECTORY", "/path/to/config");

        let config = load_from_env().unwrap();

        assert_eq!(
            config.configuration_directory,
            PathBuf::from("/path/to/config"),
            "Configuration directory should match the SQLPAGE_CONFIGURATION_DIRECTORY env var"
        );

        env::remove_var("SQLPAGE_CONFIGURATION_DIRECTORY");
    }

    #[test]
    fn test_config_priority() {
        let _lock = ENV_LOCK
            .lock()
            .expect("Another test panicked while holding the lock");
        env::set_var("SQLPAGE_WEB_ROOT", "/");

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

        env::remove_var("SQLPAGE_WEB_ROOT");
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

        env::set_var("SQLPAGE_WEB_ROOT", env_web_dir.to_str().unwrap());

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

        env::remove_var("SQLPAGE_WEB_ROOT");
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_default_values() {
        let _lock = ENV_LOCK
            .lock()
            .expect("Another test panicked while holding the lock");
        env::remove_var("SQLPAGE_CONFIGURATION_DIRECTORY");
        env::remove_var("SQLPAGE_WEB_ROOT");

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
