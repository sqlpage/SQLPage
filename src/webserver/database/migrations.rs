use super::Database;
use super::{DatabasePool, DbKind};
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
        log::debug!(
            "No migration found in {}. \
        You can specify database operations to apply when the server first starts by creating files \
        in {MIGRATIONS_DIR}/<VERSION>_<DESCRIPTION>.sql \
        where <VERSION> is a number and <DESCRIPTION> is a short string.",
            migrations_dir.display()
        );
        return Ok(());
    }
    log::info!("Found {} migrations:", migrator.migrations.len());
    for m in migrator.iter() {
        log::info!("\t{}", DisplayMigration(m));
    }
    if db.info.kind == DbKind::Odbc {
        anyhow::bail!(
            "ODBC migrations are not supported by sqlx-odbc. Apply the migrations manually or use a native SQLPage backend for managed migrations."
        );
    }
    run_migrator(&migrator, &db.connection).await.map_err(|err| {
        match err {
            MigrateError::Execute(source) => anyhow::Error::new(source),
            source => anyhow::Error::new(source),
        }
        .context(format!(
            "Failed to apply database migrations from {MIGRATIONS_DIR:?}"
        ))
    })?;
    Ok(())
}

async fn run_migrator(migrator: &Migrator, pool: &DatabasePool) -> Result<(), MigrateError> {
    match pool {
        DatabasePool::Sqlite(pool) => migrator.run(pool).await,
        DatabasePool::Postgres(pool) => migrator.run(pool).await,
        DatabasePool::MySql(pool) => migrator.run(pool).await,
        DatabasePool::Mssql(pool) => migrator.run(pool).await,
        DatabasePool::Odbc(_) => unreachable!("ODBC migrations are checked before run_migrator"),
    }
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
