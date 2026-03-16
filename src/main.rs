use sqlpage::{
    app_config::AppConfig,
    cli, telemetry,
    webserver::{self, Database},
    AppState,
};

#[actix_web::main]
async fn main() {
    if let Err(e) = init_logging() {
        eprintln!("Failed to initialize logging/telemetry: {e:#}");
        std::process::exit(1);
    }
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
    telemetry::shutdown_telemetry();
    Ok(())
}

fn init_logging() -> anyhow::Result<()> {
    let load_env = dotenvy::dotenv();

    let otel_active = telemetry::init_telemetry()?;

    match load_env {
        Ok(path) => log::info!("Loaded environment variables from {path:?}"),
        Err(dotenvy::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => log::debug!(
            "No .env file found, using only environment variables and configuration files"
        ),
        Err(e) => log::error!("Error loading .env file: {e}"),
    }

    if otel_active {
        log::info!("OpenTelemetry tracing enabled (OTEL_EXPORTER_OTLP_ENDPOINT is set)");
    }

    Ok(())
}
