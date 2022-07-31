mod http;
mod render;

use render::AllTemplates;

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

    let db = sqlx::AnyPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    log::info!("Connected to database: {database_url}");

    let listen_on = std::env::var("LISTEN_ON")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse::<std::net::SocketAddr>()
        .expect("LISTEN_ON must be a valid IP:PORT");

    let all_templates = AllTemplates::init();
    let state = AppState { db, listen_on, all_templates };
    http::run_server(state).await
}
