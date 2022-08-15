mod http;
mod render;

use render::AllTemplates;
use sqlx::any::AnyConnectOptions;
use sqlx::ConnectOptions;

pub struct AppState {
    db: sqlx::AnyPool,
    all_templates: AllTemplates,
    listen_on: std::net::SocketAddr,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Connect to the database
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://site.db?mode=rwc".to_string());

    let mut connect_options: AnyConnectOptions =
        database_url.parse().expect("Invalid database URL");
    connect_options.log_statements(log::LevelFilter::Trace);
    connect_options.log_slow_statements(
        log::LevelFilter::Warn,
        std::time::Duration::from_millis(250),
    );
    let db = sqlx::AnyPool::connect_with(connect_options)
        .await
        .expect("Failed to connect to database");

    log::info!("Connected to database: {database_url}");

    let listen_on = std::env::var("LISTEN_ON")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse::<std::net::SocketAddr>()
        .expect("LISTEN_ON must be a valid IP:PORT");

    log::info!("Starting server on {}", listen_on);

    let all_templates = AllTemplates::init();
    let state = AppState {
        db,
        listen_on,
        all_templates,
    };
    http::run_server(state).await
}
