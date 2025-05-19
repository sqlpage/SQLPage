use sqlpage::{
    app_config,
    webserver::{self, Database},
    AppState,
};

#[actix_web::main]
async fn main() {
    init_logging();
    if let Err(e) = start().await {
        log::error!("{e:?}");
        std::process::exit(1);
    }
}

async fn start() -> anyhow::Result<()> {
    let app_config = app_config::load_from_cli()?;
    let db = Database::init(&app_config).await?;
    webserver::database::migrations::apply(&app_config, &db).await?;
    let state = AppState::init_with_db(&app_config, db).await?;
    log::debug!("Starting server...");
    webserver::http::run_server(&app_config, state).await?;
    log::info!("Server stopped gracefully. Goodbye!");
    Ok(())
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
        Err(e) => log::error!("Error loading .env file: {e}"),
    }
}
