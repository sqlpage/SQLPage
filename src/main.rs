use sqlpage::{
    AppState,
    app_config::AppConfig,
    cli, telemetry,
    webserver::{self, Database},
};

mod service;

fn main() -> std::process::ExitCode {
    match main_result() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn main_result() -> anyhow::Result<()> {
    let mut cli = cli::arguments::parse_cli()?;
    #[cfg(windows)]
    if cli.service.is_some() {
        return service::windows::dispatch(cli);
    }

    actix_web::rt::System::new().block_on(async {
        if let Some(command) = cli.command.take() {
            let _ = dotenvy::dotenv();
            let config = AppConfig::from_cli(&cli)?;
            return command.execute(config).await;
        }
        let service = service::Service::console()?;
        run(cli, service).await
    })
}

async fn run(cli: cli::arguments::Cli, service: service::Service) -> anyhow::Result<()> {
    init_logging(&service)?;
    let result = start(cli, &service).await;
    if let Err(error) = &result {
        log::error!("{error:#}");
    }
    // Flush on startup failures as well as on normal shutdown, while the runtime lives.
    // Exporter shutdown may block, so keep the runtime free to finish exports.
    tokio::task::spawn_blocking(telemetry::shutdown_telemetry).await?;
    result
}

async fn start(cli: cli::arguments::Cli, service: &service::Service) -> anyhow::Result<()> {
    service.starting("Loading configuration")?;
    let app_config = AppConfig::from_cli(&cli)?;
    service.starting("Connecting to the database")?;
    let db = tokio::select! {
        biased;
        () = service.stop.cancelled() => {
            service.stopping()?;
            return Ok(());
        }
        result = Database::init(&app_config) => result?,
    };
    let pool = db.connection.clone();
    let initialize = async {
        service.starting("Applying database migrations")?;
        webserver::database::migrations::apply(&app_config, &db).await?;
        service.starting("Initializing application")?;
        AppState::init_with_db(&app_config, db).await
    };
    let state = tokio::select! {
        biased;
        () = service.stop.cancelled() => None,
        result = initialize => Some(result),
    };
    if !matches!(state, Some(Ok(_))) {
        pool.close().await;
    }
    let state = state.transpose()?;
    let Some(state) = state else {
        service.stopping()?;
        return Ok(());
    };

    let stopping_service = service.clone();
    let shutdown = async move {
        stopping_service.stop.cancelled().await;
        if let Err(error) = stopping_service.stopping() {
            log::error!("Unable to report service shutdown: {error:#}");
        }
    };
    let result =
        webserver::http::run_server_with_shutdown(&app_config, state, shutdown, || service.ready())
            .await;
    // Also close on bind failures, before the HTTP runner owns the server.
    pool.close().await;
    result?;
    log::info!("Server stopped gracefully. Goodbye!");
    Ok(())
}

fn init_logging(service: &service::Service) -> anyhow::Result<()> {
    let load_env = dotenvy::dotenv();

    let otel_active = service.init_telemetry()?;

    match load_env {
        Ok(path) => log::info!("Loaded environment variables from {}", path.display()),
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
