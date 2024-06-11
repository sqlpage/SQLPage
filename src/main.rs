use sqlpage::{
    app_config::{self, AppConfig},
    webserver, AppState,
};

#[actix_web::main]
async fn main() {
    init_logging();
    if let Err(e) = start().await {
        log::error!("{:?}", e);
        std::process::exit(1);
    }
}

async fn start() -> anyhow::Result<()> {
    let app_config = app_config::load()?;
    log::debug!("Starting with the following configuration: {app_config:#?}");
    let state = AppState::init(&app_config).await?;
    webserver::database::migrations::apply(&app_config, &state.db).await?;
    log::debug!("Starting server...");
    let (r, _) = tokio::join!(
        webserver::http::run_server(&app_config, state),
        log_welcome_message(&app_config)
    );
    r
}

async fn log_welcome_message(config: &AppConfig) {
    let address_message = if let Some(unix_socket) = &config.unix_socket {
        format!("unix socket {unix_socket:?}")
    } else if let Some(domain) = &config.https_domain {
        format!("https://{}", domain)
    } else {
        let listen_on = config.listen_on();
        let msg = if listen_on.ip().is_unspecified() {
            // let the user know the service is publicly accessible
            " (accessible on all networks of this computer)"
        } else {
            ""
        };
        format!("http://{listen_on}{msg}")
    };

    log::info!(
        "SQLPage v{} started successfully.
    Now listening on {}
    You can write your website's code in .sql files in {}.",
        env!("CARGO_PKG_VERSION"),
        address_message,
        config.web_root.display()
    );
}

fn init_logging() {
    let load_env = dotenvy::dotenv();

    let env =
        env_logger::Env::new().default_filter_or("sqlpage=info,actix_web::middleware::logger=info");
    let mut logging = env_logger::Builder::from_env(env);
    logging.format_timestamp_millis();
    logging.init();

    match load_env {
        Ok(path) => log::info!("Loaded environment variables from {path:?}"),
        Err(dotenvy::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => log::debug!(
            "No .env file found, using only environment variables and configuration files"
        ),
        Err(e) => log::error!("Error loading .env file: {}", e),
    }
}
