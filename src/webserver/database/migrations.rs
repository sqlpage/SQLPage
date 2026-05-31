use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Context;
use sha2::{Digest, Sha384};

use super::error_highlighting::display_db_error;
use super::{Database, DbKind, DbParam, make_placeholder};
use crate::MIGRATIONS_DIR;

#[derive(Debug)]
struct Migration {
    version: i64,
    description: String,
    path: PathBuf,
    sql: String,
    checksum: Vec<u8>,
}

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
    let migrations = load_migrations(&migrations_dir)
        .with_context(|| migration_err("preparing the database migration"))?;
    if migrations.is_empty() {
        log::debug!(
            "No migration found in {}. \
        You can specify database operations to apply when the server first starts by creating files \
        in {MIGRATIONS_DIR}/<VERSION>_<DESCRIPTION>.sql \
        where <VERSION> is a number and <DESCRIPTION> is a short string.",
            migrations_dir.display()
        );
        return Ok(());
    }
    log::info!("Found {} migrations:", migrations.len());
    for migration in &migrations {
        log::info!("\t{}", DisplayMigration(migration));
    }

    let mut conn = db.connection.acquire().await?;
    ensure_migrations_table(&mut conn, db.info.kind).await?;
    for migration in migrations {
        let applied = migration_row(&mut conn, db, migration.version).await?;
        if let Some(applied_checksum) = applied {
            anyhow::ensure!(
                applied_checksum == migration.checksum,
                "Migration {} has already been applied, but its checksum changed",
                DisplayMigration(&migration)
            );
            continue;
        }

        let start = Instant::now();
        if let Err(err) = conn.execute_batch(&migration.sql).await {
            return Err(
                display_db_error(&migration.path, &migration.sql, err).context(format!(
                    "Failed to apply {} migration {}",
                    db,
                    DisplayMigration(&migration)
                )),
            );
        }
        let execution_time = i64::try_from(start.elapsed().as_millis()).unwrap_or(i64::MAX);
        record_migration(&mut conn, db, &migration, execution_time).await?;
    }
    Ok(())
}

fn load_migrations(migrations_dir: &Path) -> anyhow::Result<Vec<Migration>> {
    let mut migrations = Vec::new();
    for entry in std::fs::read_dir(migrations_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("sql") {
            continue;
        }
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid migration file name: {}", path.display()))?;
        let Some((version, description)) = file_name.split_once('_') else {
            anyhow::bail!("Invalid migration file name: {}", path.display());
        };
        let version = version.parse::<i64>().with_context(|| {
            format!("Invalid migration version in file name: {}", path.display())
        })?;
        let sql = std::fs::read_to_string(&path)
            .with_context(|| format!("Unable to read migration {}", path.display()))?;
        let checksum = Sha384::digest(sql.as_bytes()).to_vec();
        migrations.push(Migration {
            version,
            description: description.to_string(),
            path,
            sql,
            checksum,
        });
    }
    migrations.sort_by_key(|migration| migration.version);
    Ok(migrations)
}

async fn ensure_migrations_table(
    conn: &mut super::DbConnection,
    kind: DbKind,
) -> anyhow::Result<()> {
    let sql = match kind {
        DbKind::Sqlite => {
            "CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            checksum BLOB NOT NULL,
            execution_time BIGINT NOT NULL
        )"
        }
        DbKind::Postgres => {
            "CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
            success BOOLEAN NOT NULL,
            checksum BYTEA NOT NULL,
            execution_time BIGINT NOT NULL
        )"
        }
        DbKind::MySql | DbKind::Odbc => {
            "CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description VARCHAR(255) NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            checksum BLOB NOT NULL,
            execution_time BIGINT NOT NULL
        )"
        }
        DbKind::Mssql => {
            "IF OBJECT_ID(N'_sqlx_migrations', N'U') IS NULL
            CREATE TABLE _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description NVARCHAR(255) NOT NULL,
            installed_on DATETIME2 NOT NULL DEFAULT SYSUTCDATETIME(),
            success BIT NOT NULL,
            checksum VARBINARY(MAX) NOT NULL,
            execution_time BIGINT NOT NULL
        )"
        }
    };
    conn.execute_command(sql, &[]).await?;
    Ok(())
}

async fn migration_row(
    conn: &mut super::DbConnection,
    db: &Database,
    version: i64,
) -> anyhow::Result<Option<Vec<u8>>> {
    let sql = format!(
        "SELECT checksum FROM _sqlx_migrations WHERE version = {}",
        make_placeholder(db.info.kind, 1)
    );
    let row = conn
        .fetch_optional(&sql, &[DbParam::Integer(version)])
        .await?;
    Ok(row.and_then(|row| match row.values.first() {
        Some(super::driver::DbValue::Bytes(bytes)) => Some(bytes.clone()),
        Some(super::driver::DbValue::Text(text)) => Some(text.as_bytes().to_vec()),
        _ => None,
    }))
}

async fn record_migration(
    conn: &mut super::DbConnection,
    db: &Database,
    migration: &Migration,
    execution_time: i64,
) -> anyhow::Result<()> {
    let sql = format!(
        "INSERT INTO _sqlx_migrations(version, description, success, checksum, execution_time) VALUES ({}, {}, {}, {}, {})",
        make_placeholder(db.info.kind, 1),
        make_placeholder(db.info.kind, 2),
        make_placeholder(db.info.kind, 3),
        make_placeholder(db.info.kind, 4),
        make_placeholder(db.info.kind, 5),
    );
    conn.execute_command(
        &sql,
        &[
            DbParam::Integer(migration.version),
            DbParam::Text(migration.description.clone()),
            DbParam::Bool(true),
            DbParam::Bytes(migration.checksum.clone()),
            DbParam::Integer(execution_time),
        ],
    )
    .await?;
    Ok(())
}

struct DisplayMigration<'a>(&'a Migration);

impl std::fmt::Display for DisplayMigration<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:04}] {}", self.0.version, self.0.description)
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
