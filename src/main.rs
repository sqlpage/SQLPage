use sqlpage::{
    app_config::{self, AppConfig},
    webserver, AppState, Config,
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
    log::debug!("Starting with the following configuration: {app_config:?}");
    let state = AppState::init(&app_config).await?;
    webserver::database::migrations::apply(&state.db).await?;
    let listen_on = app_config.listen_on;
    log::info!("Starting server on {}", listen_on);
    let config = Config { listen_on };
    let (r, _) = tokio::join!(
        webserver::http::run_server(config, state),
        log_welcome_message(&app_config)
    );
    r
}

async fn log_welcome_message(config: &AppConfig) {
    // Don't show 0.0.0.0 as the host, show the actual IP address
    let http_addr = config.listen_on.to_string().replace(
        "0.0.0.0",
        std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
            .to_string()
            .as_str(),
    );

    log::info!(
        "Server started successfully.
    SQLPage is now running on http://{}/
    You can write your website's code in .sql files in {}.",
        http_addr,
        config.web_root.display()
    );
}

fn init_logging() {
    let env = env_logger::Env::new().default_filter_or("info");
    let mut logging = env_logger::Builder::from_env(env);
    logging.format_timestamp_millis();
    logging.init();
}
