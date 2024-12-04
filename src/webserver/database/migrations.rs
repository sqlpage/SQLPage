use super::error_highlighting::display_db_error;
use super::Database;
use crate::MIGRATIONS_DIR;
use anyhow;
use anyhow::Context;
use sqlx::migrate::MigrateError;
use sqlx::migrate::Migration;
use sqlx::migrate::Migrator;

pub async fn apply(config: &crate::app_config::AppConfig, db: &Database) -> anyhow::Result<()> {
    let migrations_dir = config.configuration_directory.join(MIGRATIONS_DIR);
    if !migrations_dir.exists() {
        log::info!(
            "Not applying database migrations because '{}' does not exist",
            migrations_dir.display()
        );
        return Ok(());
    }
    log::debug!("Applying migrations from '{}'", migrations_dir.display());
    let migrator = Migrator::new(migrations_dir.clone())
        .await
        .with_context(|| migration_err("preparing the database migration"))?;
    if migrator.migrations.is_empty() {
        log::info!("No migration found in {}. \
        You can specify database operations to apply when the server first starts by creating files \
        in {MIGRATIONS_DIR}/<VERSION>_<DESCRIPTION>.sql \
        where <VERSION> is a number and <DESCRIPTION> is a short string.", migrations_dir.display());
        return Ok(());
    }
    log::info!("Found {} migrations:", migrator.migrations.len());
    for m in migrator.iter() {
        log::info!("\t{}", DisplayMigration(m));
    }
    migrator.run(&db.connection).await.map_err(|err| {
        match err {
            MigrateError::Execute(n, source) => {
                let migration = migrator.iter().find(|&m| m.version == n).unwrap();
                let source_file =
                    migrations_dir.join(format!("{:04}_{}.sql", n, migration.description));
                display_db_error(&source_file, &migration.sql, source).context(format!(
                    "Failed to apply {} migration {}",
                    db,
                    DisplayMigration(migration)
                ))
            }
            source => anyhow::Error::new(source),
        }
        .context(format!(
            "Failed to apply database migrations from {MIGRATIONS_DIR:?}"
        ))
    })?;
    Ok(())
}

struct DisplayMigration<'a>(&'a Migration);

impl std::fmt::Display for DisplayMigration<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Migration {
            version,
            migration_type,
            description,
            ..
        } = &self.0;
        write!(f, "[{version:04}]")?;
        if migration_type != &sqlx::migrate::MigrationType::Simple {
            write!(f, " ({migration_type:?})")?;
        }
        write!(f, " {description}")?;
        Ok(())
    }
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
