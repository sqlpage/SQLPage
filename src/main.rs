use clap::Parser;
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

    let cli = app_config::Cli::parse();

    if let Some(command) = cli.command {
        match command {
            app_config::Commands::CreateMigration { migration_name } => {
                create_migration_file(&migration_name)?;
                return Ok(());
            }
        }
    }

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

fn create_migration_file(migration_name: &str) -> anyhow::Result<()> {
    use chrono::Utc;
    use std::fs;
    use std::path::Path;

    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let snake_case_name = migration_name
        .replace(|c: char| !c.is_alphanumeric(), "_")
        .to_lowercase();
    let file_name = format!("{}_{}.sql", timestamp, snake_case_name);
    let migrations_dir = Path::new("sqlpage/migrations");

    if !migrations_dir.exists() {
        fs::create_dir_all(migrations_dir)?;
    }

    let mut unique_file_name = file_name.clone();
    let mut counter = 1;

    while migrations_dir.join(&unique_file_name).exists() {
        unique_file_name = format!("{}_{}_{}.sql", timestamp, snake_case_name, counter);
        counter += 1;
    }

    let file_path = migrations_dir.join(unique_file_name);
    fs::write(file_path, "-- Write your migration here\n")?;

    println!("Migration created successfully.");
    Ok(())
}
