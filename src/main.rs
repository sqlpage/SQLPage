use sqlpage::{
    app_config::AppConfig,
    cli,
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
    let cli = cli::arguments::parse_cli()?;
    let app_config = AppConfig::from_cli(&cli)?;

    if let Some(command) = cli.command {
        return command.execute(app_config).await;
    }

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
