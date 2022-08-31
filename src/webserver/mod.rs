use crate::MIGRATIONS_DIR;
use sqlx::migrate::Migrator;
use sqlx::AnyPool;
use std::path::Path;

mod database;
pub mod http;

pub async fn apply_migrations(db: &AnyPool) -> anyhow::Result<()> {
    let migrations_dir = Path::new(MIGRATIONS_DIR);
    if !migrations_dir.exists() {
        log::debug!(
            "Not applying database migrations because '{}' does not exist",
            MIGRATIONS_DIR
        );
        return Ok(());
    }
    let migrator = Migrator::new(migrations_dir).await?;
    migrator.run(db).await?;
    Ok(())
}
