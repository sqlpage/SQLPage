use sqlpage::{app_config, webserver, AppState, Config};

#[actix_web::main]
async fn main() {
    init_logging();
    if let Err(e) = start().await {
        log::error!("{:?}", e);
        std::process::exit(1);
    }
}

async fn start() -> anyhow::Result<()> {
    let config = app_config::load()?;
    log::debug!("Starting with the following configuration: {config:?}");
    let state = AppState::init(&config).await?;
    webserver::apply_migrations(&state.db).await?;
    let listen_on = config.listen_on;
    log::info!("Starting server on {}", listen_on);
    let config = Config { listen_on };
    webserver::http::run_server(config, state).await?;
    Ok(())
}

fn init_logging() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
}
