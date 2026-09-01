use super::Database;
use super::error_highlighting::display_db_error;
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
    migrator.run(&db.connection).await.map_err(|err| {
        match err {
            MigrateError::Execute(n, source) => {
                let migration = failing_migration(&migrator.migrations, n)
                    .expect("sqlx reports the version of a migration it just ran");
                let source_file = migrations_dir.join(format!(
                    "{:04}_{}{}",
                    n,
                    migration.description.replace(' ', "_"),
                    migration.migration_type.suffix()
                ));
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

fn failing_migration(migrations: &[Migration], version: i64) -> Option<&Migration> {
    migrations
        .iter()
        .find(|m| m.version == version && !m.migration_type.is_down_migration())
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::migrate::MigrationType;
    use tempfile::TempDir;

    fn migration(version: i64, description: &str, migration_type: MigrationType) -> Migration {
        Migration::new(
            version,
            description.to_owned().into(),
            migration_type,
            String::new().into(),
        )
    }

    async fn apply_error(files: &[(&str, &str)]) -> String {
        let dir = TempDir::new().unwrap();
        let migrations_dir = dir.path().join(MIGRATIONS_DIR);
        std::fs::create_dir(&migrations_dir).unwrap();
        for (name, sql) in files {
            std::fs::write(migrations_dir.join(name), sql).unwrap();
        }
        let mut config = crate::app_config::tests::test_config();
        config.database_url = "sqlite::memory:".to_owned();
        config.configuration_directory = dir.path().to_owned();
        let db = Database::init(&config).await.unwrap();
        let error = apply(&config, &db)
            .await
            .expect_err("the migration must fail");
        format!("{error:#}")
    }

    #[actix_web::test]
    async fn only_the_failing_migration_is_named() {
        let error = apply_error(&[
            ("0001_ok.sql", "CREATE TABLE t(x);"),
            ("0002_bad_thing.sql", "SELECT * FROM does_not_exist;"),
        ])
        .await;
        assert!(error.contains("[0002] bad thing"), "{error}");
        assert!(error.contains("0002_bad_thing.sql"), "{error}");
        assert!(error.contains("does_not_exist"), "{error}");
        assert!(!error.contains("[0001] ok"), "{error}");
    }

    #[actix_web::test]
    async fn a_reversible_migration_reports_the_half_that_ran() {
        let error = apply_error(&[
            ("0001_add_new_users.up.sql", "SELECT * FROM does_not_exist;"),
            ("0001_add_new_users.down.sql", "SELECT 'the down half';"),
        ])
        .await;
        assert!(
            error.contains("[0001] (ReversibleUp) add new users"),
            "{error}"
        );
        assert!(error.contains("0001_add_new_users.up.sql"), "{error}");
        assert!(!error.contains("the down half"), "{error}");
    }

    #[test]
    fn the_down_half_is_never_the_failing_migration() {
        let up = migration(1, "x", MigrationType::ReversibleUp);
        let down = migration(1, "x", MigrationType::ReversibleDown);
        for pair in [[up.clone(), down.clone()], [down.clone(), up.clone()]] {
            assert_eq!(
                failing_migration(&pair, 1).map(|m| m.migration_type),
                Some(MigrationType::ReversibleUp)
            );
        }
        assert!(failing_migration(&[down], 1).is_none());
    }

    #[test]
    fn display_migration_shows_version_description_and_reversibility() {
        assert_eq!(
            DisplayMigration(&migration(7, "add users", MigrationType::Simple)).to_string(),
            "[0007] add users"
        );
        assert_eq!(
            DisplayMigration(&migration(7, "add users", MigrationType::ReversibleUp)).to_string(),
            "[0007] (ReversibleUp) add users"
        );
    }
}
