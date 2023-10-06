use super::Database;
use crate::webserver::database::highlight_sql_error;
use crate::MIGRATIONS_DIR;
use anyhow;
use anyhow::Context;
use sqlx::migrate::MigrateError;
use sqlx::migrate::Migrator;

pub async fn apply(db: &Database) -> anyhow::Result<()> {
    let migrations_dir = std::env::current_dir()
        .unwrap_or_default()
        .join(MIGRATIONS_DIR);
    if !migrations_dir.exists() {
        log::info!(
            "Not applying database migrations because '{}' does not exist",
            migrations_dir.display()
        );
        return Ok(());
    }
    log::info!("Applying migrations from '{}'", migrations_dir.display());
    let migrator = Migrator::new(migrations_dir)
        .await
        .with_context(|| migration_err("preparing the database migration"))?;
    if migrator.migrations.is_empty() {
        log::info!("No migration found. \
        You can specify database operations to apply when the server first starts by creating files \
        in {MIGRATIONS_DIR}/<VERSION>_<DESCRIPTION>.sql \
        where <VERSION> is a number and <DESCRIPTION> is a short string.");
        return Ok(());
    }
    log::info!("Found {} migrations:", migrator.migrations.len());
    for m in migrator.iter() {
        log::info!(
            "\t[{:04}] {:?} {}",
            m.version,
            m.migration_type,
            m.description
        );
    }
    migrator.run(&db.connection).await.map_err(|err| {
        match err {
            MigrateError::Execute(n, source) => {
                let migration = migrator.iter().find(|&m| m.version == n).unwrap();
                highlight_sql_error("Error in the SQL migration", &migration.sql, source).context(
                    format!(
                        "Failed to apply migration [{:04}] {:?} {}",
                        migration.version, migration.migration_type, migration.description
                    ),
                )
            }
            source => anyhow::Error::new(source),
        }
        .context(format!(
            "Failed to apply database migrations from {MIGRATIONS_DIR:?}"
        ))
    })?;
    Ok(())
}

fn migration_err(operation: &'static str) -> String {
    format!(
        "An error occurred while {operation}.
        The path '{MIGRATIONS_DIR}' has to point to a directory, which contains valid SQL files
        with names using the format '<VERSION>_<DESCRIPTION>.sql',
        where <VERSION> is a positive number, and <DESCRIPTION> is a string.
        The current state of migrations will be stored in a table called _sqlx_migrations."
    )
}
