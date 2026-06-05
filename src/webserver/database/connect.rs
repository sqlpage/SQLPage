use std::time::Duration;

use super::Database;
use crate::{
    ON_CONNECT_FILE, ON_RESET_FILE,
    app_config::AppConfig,
    webserver::database::{DbInfo, DbKind, DbPool, SupportedDatabase},
};
use anyhow::Context;

impl Database {
    pub async fn init(config: &AppConfig) -> anyhow::Result<Self> {
        let database_url = &config.database_url;
        let db_kind = kind_from_database_url(database_url).with_context(|| {
            format!(
                "\"{database_url}\" is not a valid database URL. Please change the \"database_url\" option in the configuration file."
            )
        })?;
        if config.database_password.is_some() && matches!(db_kind, DbKind::Sqlite | DbKind::Odbc) {
            log::warn!(
                "Setting a separate database password is not supported for {db_kind:?}; include credentials in the database URL or connection string"
            );
        }
        log::debug!("Connecting to a {db_kind:?} database on {database_url}");
        let on_connect_sql = read_connection_handler(config, ON_CONNECT_FILE);
        let on_reset_sql = read_connection_handler(config, ON_RESET_FILE);
        if on_reset_sql.is_some() {
            log::warn!(
                "{ON_RESET_FILE} is currently ignored by the native driver pool because connections are not reused yet"
            );
        }

        let pool = DbPool::new(
            config,
            db_kind,
            default_max_connections(config, db_kind),
            on_connect_sql,
        );

        let mut retries = config.database_connection_retries;
        let mut conn = loop {
            match pool.acquire().await {
                Ok(c) => break c,
                Err(e) => {
                    if retries == 0 {
                        return Err(anyhow::Error::new(e)
                            .context(format!("Unable to open connection to {database_url}")));
                    }
                    log::warn!("Failed to connect to the database: {e:#}. Retrying in 5 seconds.");
                    retries -= 1;
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        };
        let dbms_name = conn.dbms_name().await?;
        let database_type = SupportedDatabase::from_dbms_name(&dbms_name);
        drop(conn);

        log::debug!("Initialized {dbms_name:?} database pool");
        Ok(Database {
            connection: pool,
            info: DbInfo {
                dbms_name,
                database_type,
                kind: db_kind,
            },
        })
    }
}

fn kind_from_database_url(url: &str) -> anyhow::Result<DbKind> {
    if url.starts_with("sqlite:") {
        Ok(DbKind::Sqlite)
    } else if url.starts_with("postgres:") || url.starts_with("postgresql:") {
        Ok(DbKind::Postgres)
    } else if url.starts_with("mysql:") || url.starts_with("mariadb:") {
        Ok(DbKind::MySql)
    } else if url.starts_with("mssql:") || url.starts_with("sqlserver:") {
        Ok(DbKind::Mssql)
    } else if url.starts_with("odbc:") || is_raw_odbc_connection_string(url) {
        Ok(DbKind::Odbc)
    } else {
        anyhow::bail!("unsupported database URL scheme")
    }
}

fn is_raw_odbc_connection_string(url: &str) -> bool {
    let trimmed = url.trim_start();
    trimmed.starts_with("Driver=") || trimmed.starts_with("DSN=")
}

fn default_max_connections(config: &AppConfig, kind: DbKind) -> u32 {
    if let Some(max) = config.max_database_pool_connections {
        return max;
    }
    match kind {
        DbKind::Postgres | DbKind::Odbc => 50,
        DbKind::MySql => 75,
        DbKind::Sqlite => {
            if config.database_url.contains(":memory:") {
                128
            } else {
                16
            }
        }
        DbKind::Mssql => 100,
    }
}

fn read_connection_handler(config: &AppConfig, file_name: &str) -> Option<String> {
    let file = config.configuration_directory.join(file_name);
    if !file.exists() {
        log::debug!(
            "Not creating a custom SQL connection handler because {} does not exist",
            file.display()
        );
        return None;
    }
    log::info!(
        "Creating a custom SQL connection handler from {}",
        file.display()
    );
    match std::fs::read_to_string(&file) {
        Ok(sql) => {
            log::trace!("The custom SQL connection handler is:\n{sql}");
            Some(sql)
        }
        Err(e) => {
            log::error!("Unable to read the file {}: {}", file.display(), e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_prefixed_odbc_urls() {
        assert_eq!(
            kind_from_database_url("odbc:Driver=Oracle 21 ODBC driver;Dbq=//localhost/FREEPDB1")
                .unwrap(),
            DbKind::Odbc
        );
    }

    #[test]
    fn detects_raw_odbc_connection_strings() {
        assert_eq!(
            kind_from_database_url(
                "Driver=Oracle 21 ODBC driver;Dbq=//127.0.0.1:1521/FREEPDB1;Uid=root;Pwd=Password123!"
            )
            .unwrap(),
            DbKind::Odbc
        );
        assert_eq!(
            kind_from_database_url("DSN=warehouse;Uid=user;Pwd=secret").unwrap(),
            DbKind::Odbc
        );
    }
}
