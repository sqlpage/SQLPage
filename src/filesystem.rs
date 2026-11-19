use crate::webserver::ErrorWithStatus;
use crate::webserver::database::Parameter;
use crate::webserver::database::SupportedDatabase;
use crate::webserver::{Database, StatusCodeResultExt, make_placeholder};
use crate::{AppState, TEMPLATES_DIR};
use anyhow::Context;
use chrono::{DateTime, Utc};
use std::fmt::Write;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy)]
#[must_use]
pub struct FileAccess<'a> {
    path: &'a Path,
    privileged: bool,
}

impl<'a> FileAccess<'a> {
    /// Creates access for an untrusted, HTTP-facing path after validating it.
    pub fn unprivileged(path: &'a Path) -> anyhow::Result<Self> {
        validate_unprivileged_path(path)?;
        Ok(Self {
            path,
            privileged: false,
        })
    }

    /// Creates access for a trusted internal path.
    pub const fn privileged(path: &'a Path) -> Self {
        Self {
            path,
            privileged: true,
        }
    }

    #[must_use]
    pub const fn path(&self) -> &'a Path {
        self.path
    }
}

pub(crate) struct FileSystem {
    local_root: PathBuf,
    db_fs_queries: Option<DbFsQueries>,
}

impl FileSystem {
    pub(crate) async fn init(local_root: impl Into<PathBuf>, db: &Database) -> Self {
        Self {
            local_root: local_root.into(),
            db_fs_queries: match DbFsQueries::init(db).await {
                Ok(q) => Some(q),
                Err(e) => {
                    log::debug!(
                        "Using local filesystem only, could not initialize on-database filesystem. \
                        You can host sql files directly in your database by creating the following table: \n\
                        {} \n\
                        The error while trying to use the database file system is: {e:#}",
                        DbFsQueries::get_create_table_sql(db.info.database_type)
                    );
                    None
                }
            },
        }
    }

    pub(crate) async fn modified_since(
        &self,
        app_state: &AppState,
        access: FileAccess<'_>,
        since: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        let path = access.path();
        let local_path = self.safe_local_path(app_state, access);
        let local_result = file_modified_since_local(&local_path, since).await;
        log::trace!(
            "Local file {} modified since {since:?} ? {local_result:?}",
            local_path.display()
        );
        match (local_result, &self.db_fs_queries) {
            (Ok(modified), _) => Ok(modified),
            (Err(e), Some(db_fs)) if is_path_missing_error(&e) => {
                // no local file, try the database
                db_fs
                    .file_modified_since_in_db(app_state, path, since)
                    .await
            }
            (Err(e), _) => {
                let status = io_error_status(&local_path, &e).await;
                Err(e).with_status(status).with_context(|| {
                    format!("Unable to read local file metadata for {}", path.display())
                })
            }
        }
    }

    pub(crate) async fn read_to_string(
        &self,
        app_state: &AppState,
        access: FileAccess<'_>,
    ) -> anyhow::Result<String> {
        let path = access.path();
        let bytes = self.read_file(app_state, access).await?;
        String::from_utf8(bytes).map_err(|utf8_err| {
            let invalid_idx = utf8_err.utf8_error().valid_up_to();
            let bytes = utf8_err.into_bytes();
            let valid_prefix = String::from_utf8_lossy(&bytes[..invalid_idx]);
            let line_num = valid_prefix.lines().count();
            let mut bad_seq = valid_prefix.lines().last().unwrap_or_default().to_string();
            let bad_char_idx = bad_seq.len() + 1;
            for b in bytes[invalid_idx..].iter().take(8) {
                write!(&mut bad_seq, "\\x{b:02X}").unwrap();
            }

            let display_path = path.display();
            anyhow::format_err!(
                "SQLPage expects all sql files to be encoded in UTF-8. \n\
                In \"{display_path}\", around line {line_num} character {bad_char_idx}, the following invalid UTF-8 byte sequence was found: \n\
                \"{bad_seq}\". \n\
                Please convert the file to UTF-8.",
            )
        })
    }

    pub(crate) async fn read_file(
        &self,
        app_state: &AppState,
        access: FileAccess<'_>,
    ) -> anyhow::Result<Vec<u8>> {
        let path = access.path();
        let local_path = self.safe_local_path(app_state, access);
        log::debug!(
            "Reading file {} from {}",
            path.display(),
            local_path.display()
        );
        let local_result = tokio::fs::read(&local_path).await;
        match (local_result, &self.db_fs_queries) {
            (Ok(f), _) => Ok(f),
            (Err(e), Some(db_fs)) if is_path_missing_error(&e) => {
                // no local file, try the database
                db_fs.read_file(app_state, path.as_ref()).await
            }
            (Err(e), _) => {
                let status = io_error_status(&local_path, &e).await;
                Err(e)
                    .with_status(status)
                    .with_context(|| format!("Unable to read local file {}", path.display()))
            }
        }
    }

    fn safe_local_path(&self, app_state: &AppState, access: FileAccess<'_>) -> PathBuf {
        let path = access.path();
        if access.privileged {
            // Templates requests are always made to the static TEMPLATES_DIR, because this is where they are stored in the database
            // but when serving them from the filesystem, we need to serve them from the `SQLPAGE_CONFIGURATION_DIRECTORY/templates` directory
            if let Ok(template_path) = path.strip_prefix(TEMPLATES_DIR) {
                let normalized = app_state
                    .config
                    .configuration_directory
                    .join("templates")
                    .join(template_path);
                log::trace!(
                    "Normalizing template path {} to {}",
                    path.display(),
                    normalized.display()
                );
                return normalized;
            }
        }
        self.local_root.join(path)
    }

    pub(crate) async fn file_exists(
        &self,
        app_state: &AppState,
        access: FileAccess<'_>,
    ) -> anyhow::Result<bool> {
        let path = access.path();
        let safe_path = self.safe_local_path(app_state, access);
        let local_exists = match tokio::fs::try_exists(&safe_path).await {
            Ok(exists) => exists,
            Err(e) if is_path_missing_error(&e) => false,
            Err(e) => {
                let status = io_error_status(&safe_path, &e).await;
                return Err(e).with_status(status).with_context(|| {
                    format!("Unable to check if {} exists locally", path.display())
                });
            }
        };

        // If not in local fs and we have db_fs, check database
        if !local_exists {
            log::debug!(
                "File {} not found in local filesystem, checking database",
                path.display()
            );
            if let Some(db_fs) = &self.db_fs_queries {
                return db_fs.file_exists(app_state, path).await;
            }
        }
        Ok(local_exists)
    }
}

/// Rejects paths that an untrusted HTTP request must never reach: the reserved
/// `sqlpage/` prefix, dotfiles, parent-directory traversal and absolute/root paths.
fn validate_unprivileged_path(path: &Path) -> anyhow::Result<()> {
    for (i, component) in path.components().enumerate() {
        if let Component::Normal(c) = component {
            let name = c.as_encoded_bytes();
            if i == 0 && c.eq_ignore_ascii_case("sqlpage") {
                return Err(ErrorWithStatus {
                    status: actix_web::http::StatusCode::FORBIDDEN,
                })
                .with_context(
                    || "The /sqlpage/ path prefix is reserved for internal use. It is not public.",
                );
            }
            if name.starts_with(b".") {
                return Err(ErrorWithStatus {
                    status: actix_web::http::StatusCode::FORBIDDEN,
                })
                .with_context(|| "Directory traversal is not allowed");
            }
            // SQL Server ignores trailing spaces when comparing file-store paths.
            // Windows can also alias trailing dots and NTFS stream names to the
            // same file. Reject these spellings before routing can serve a SQL
            // file as an ordinary static asset.
            if name.ends_with(b" ")
                || cfg!(windows) && (name.ends_with(b".") || name.contains(&b':'))
            {
                return Err(ErrorWithStatus {
                    status: actix_web::http::StatusCode::FORBIDDEN,
                })
                .with_context(|| "Noncanonical file paths are not allowed");
            }
        } else {
            return Err(ErrorWithStatus {
                status: actix_web::http::StatusCode::FORBIDDEN,
            })
            .with_context(|| {
                format!(
                    "Unsupported path: {}. Path component '{component:?}' is not allowed.",
                    path.display()
                )
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod unprivileged_path_tests {
    use super::FileAccess;
    use std::path::Path;

    #[test]
    fn rejects_trailing_space_aliases() {
        assert!(FileAccess::unprivileged(Path::new("index.sql ")).is_err());
        assert!(FileAccess::unprivileged(Path::new("folder /index.sql")).is_err());
        assert!(FileAccess::unprivileged(Path::new("index.sql")).is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn rejects_windows_trailing_dot_aliases() {
        assert!(FileAccess::unprivileged(Path::new("index.sql.")).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn rejects_windows_ntfs_stream_aliases() {
        assert!(FileAccess::unprivileged(Path::new("index.sql::$DATA")).is_err());
    }
}

fn is_path_missing_error(error: &std::io::Error) -> bool {
    matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory)
}

async fn io_error_status(local_path: &Path, error: &std::io::Error) -> actix_web::http::StatusCode {
    match error.kind() {
        ErrorKind::NotFound | ErrorKind::NotADirectory => actix_web::http::StatusCode::NOT_FOUND,
        // Reading a directory reports IsADirectory on unix but PermissionDenied on
        // windows, so the path itself has to be inspected to tell the two apart.
        ErrorKind::IsADirectory | ErrorKind::PermissionDenied
            if is_local_directory(local_path).await =>
        {
            actix_web::http::StatusCode::NOT_FOUND
        }
        ErrorKind::PermissionDenied => actix_web::http::StatusCode::FORBIDDEN,
        _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn is_local_directory(path: &Path) -> bool {
    tokio::fs::metadata(path)
        .await
        .is_ok_and(|metadata| metadata.is_dir())
}

async fn file_modified_since_local(path: &Path, since: DateTime<Utc>) -> tokio::io::Result<bool> {
    tokio::fs::metadata(path)
        .await
        .and_then(|m| m.modified())
        .map(|modified_at| DateTime::<Utc>::from(modified_at) > since)
}

pub struct DbFsQueries {
    was_modified: String,
    read_file: String,
    exists: String,
}

impl DbFsQueries {
    #[must_use]
    pub fn get_create_table_sql(dbms: SupportedDatabase) -> &'static str {
        match dbms {
            SupportedDatabase::Mssql => {
                "CREATE TABLE sqlpage_files(path NVARCHAR(255) NOT NULL PRIMARY KEY, contents VARBINARY(MAX), last_modified DATETIME2(3) NOT NULL DEFAULT CURRENT_TIMESTAMP);"
            }
            SupportedDatabase::Postgres => {
                "CREATE TABLE IF NOT EXISTS sqlpage_files(path VARCHAR(255) NOT NULL PRIMARY KEY, contents BYTEA, last_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP);"
            }
            SupportedDatabase::Snowflake => {
                "CREATE TABLE IF NOT EXISTS sqlpage_files(path VARCHAR(255) NOT NULL PRIMARY KEY, contents VARBINARY, last_modified TIMESTAMP_TZ DEFAULT CONVERT_TIMEZONE('UTC', CURRENT_TIMESTAMP()));"
            }
            _ => {
                "CREATE TABLE IF NOT EXISTS sqlpage_files(path VARCHAR(255) NOT NULL PRIMARY KEY, contents BLOB, last_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP);"
            }
        }
    }

    async fn init(db: &Database) -> anyhow::Result<Self> {
        log::debug!("Initializing database filesystem queries");
        Self::check_table_available(db).await?;
        let path_param = make_placeholder(db.info.kind, 1);
        let timestamp_param = make_placeholder(db.info.kind, 2);
        Ok(Self {
            was_modified: format!(
                "SELECT 1 FROM sqlpage_files WHERE path = {path_param} AND last_modified >= {timestamp_param}"
            ),
            read_file: format!("SELECT contents FROM sqlpage_files WHERE path = {path_param}"),
            exists: format!("SELECT 1 FROM sqlpage_files WHERE path = {path_param}"),
        })
    }

    async fn check_table_available(db: &Database) -> anyhow::Result<()> {
        db.connection
            .execute("SELECT path, contents, last_modified FROM sqlpage_files WHERE 1 = 0")
            .await
            .context("Unable to access sqlpage_files")
    }

    async fn file_modified_since_in_db(
        &self,
        app_state: &AppState,
        path: &Path,
        since: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        // SQLite's CURRENT_TIMESTAMP uses a space separator. Preserve the
        // same lexical ordering for existing timestamp strings.
        let sqlite_timestamp;
        let timestamp =
            if app_state.db.info.kind == crate::webserver::database::DatabaseDriver::Sqlite {
                sqlite_timestamp = since.format("%F %T%.f%:z").to_string();
                Parameter::Text(Some(&sqlite_timestamp))
            } else {
                Parameter::Timestamp(since)
            };
        app_state
            .db
            .exists(
                &self.was_modified,
                &[Parameter::Text(Some(&path.to_string_lossy())), timestamp],
            )
            .await
            .with_context(|| {
                format!(
                    "Unable to check when {} was last modified in the database",
                    path.display()
                )
            })
    }

    async fn read_file(&self, app_state: &AppState, path: &Path) -> anyhow::Result<Vec<u8>> {
        app_state
            .db
            .fetch_bytes(
                &self.read_file,
                &[Parameter::Text(Some(&path.to_string_lossy()))],
            )
            .await
            .map_err(anyhow::Error::from)
            .and_then(|file| {
                file.ok_or_else(|| {
                    ErrorWithStatus {
                        status: actix_web::http::StatusCode::NOT_FOUND,
                    }
                    .into()
                })
            })
            .with_context(|| format!("Unable to read {} from the database", path.display()))
    }

    async fn file_exists(&self, app_state: &AppState, path: &Path) -> anyhow::Result<bool> {
        app_state
            .db
            .exists(
                &self.exists,
                &[Parameter::Text(Some(&path.to_string_lossy()))],
            )
            .await
            .with_context(|| {
                format!(
                    "Unable to check if {} exists in the database",
                    path.display()
                )
            })
    }
}

#[actix_web::test]
async fn test_sql_file_read_utf8() -> anyhow::Result<()> {
    use crate::app_config;
    let config = app_config::tests::test_config();
    let state = AppState::init(&config).await?;

    // Oracle has specific issues with implicit timestamp conversions and empty strings in this test setup
    // so we skip it for Oracle to avoid complex workarounds in the main codebase.
    if config.database_url.contains("Oracle") {
        log::warn!(
            "Skipping test_sql_file_read_utf8 for Oracle due to date format/implicit conversion issues"
        );
        return Ok(());
    }

    let create_table_sql = DbFsQueries::get_create_table_sql(state.db.info.database_type);
    let db = &state.db;
    let conn = &db.connection;
    // Other tests share this database, so never drop a table their initialized state may use.
    if conn
        .execute("SELECT 1 FROM sqlpage_files WHERE 1 = 0")
        .await
        .is_err()
    {
        log::debug!("Creating table sqlpage_files: {create_table_sql}");
        conn.execute(create_table_sql).await?;
    }
    conn.execute("DELETE FROM sqlpage_files WHERE path = 'unit test file.txt'")
        .await?;

    let dbms = db.info.kind;
    let insert_sql = format!(
        "INSERT INTO sqlpage_files(path, contents) VALUES ({}, {})",
        make_placeholder(dbms, 1),
        make_placeholder(dbms, 2)
    );
    db.execute_with(
        &insert_sql,
        &[
            Parameter::Text(Some("unit test file.txt")),
            Parameter::Bytes("Héllö world! 😀".as_bytes()),
        ],
    )
    .await?;

    let fs = FileSystem::init("/", db).await;
    let actual = fs
        .read_to_string(
            &state,
            FileAccess::unprivileged("unit test file.txt".as_ref())?,
        )
        .await?;
    assert_eq!(actual, "Héllö world! 😀");

    let one_hour_ago = Utc::now() - chrono::Duration::hours(1);
    let one_hour_future = Utc::now() + chrono::Duration::hours(1);

    let was_modified = fs
        .modified_since(
            &state,
            FileAccess::unprivileged("unit test file.txt".as_ref())?,
            one_hour_ago,
        )
        .await?;

    assert!(was_modified, "File should be modified since one hour ago");

    let was_modified = fs
        .modified_since(
            &state,
            FileAccess::unprivileged("unit test file.txt".as_ref())?,
            one_hour_future,
        )
        .await?;
    assert!(
        !was_modified,
        "File should not be modified since one hour in the future"
    );

    Ok(())
}

#[actix_web::test]
async fn test_local_file_modification_time() -> anyhow::Result<()> {
    let config = crate::app_config::tests::test_config();
    let state = AppState::init(&config).await?;
    let fs = FileSystem::init(".", &state.db).await;
    let committed_file = || FileAccess::unprivileged("tests/it_works.txt".as_ref());

    assert!(
        fs.modified_since(&state, committed_file()?, DateTime::UNIX_EPOCH)
            .await?
    );
    assert!(
        !fs.modified_since(
            &state,
            committed_file()?,
            Utc::now() + chrono::Duration::hours(1)
        )
        .await?
    );
    Ok(())
}
