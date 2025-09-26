use crate::webserver::database::SupportedDatabase;
use crate::webserver::ErrorWithStatus;
use crate::webserver::{make_placeholder, Database};
use crate::{AppState, TEMPLATES_DIR};
use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::any::{AnyStatement, AnyTypeInfo};
use sqlx::postgres::types::PgTimeTz;
use sqlx::{Postgres, Statement, Type};
use std::fmt::Write;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

pub(crate) struct FileSystem {
    local_root: PathBuf,
    db_fs_queries: Option<DbFsQueries>,
}

impl FileSystem {
    pub async fn init(local_root: impl Into<PathBuf>, db: &Database) -> Self {
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

    pub async fn modified_since(
        &self,
        app_state: &AppState,
        path: &Path,
        since: DateTime<Utc>,
        priviledged: bool,
    ) -> anyhow::Result<bool> {
        let local_path = self.safe_local_path(app_state, path, priviledged)?;
        let local_result = file_modified_since_local(&local_path, since).await;
        match (local_result, &self.db_fs_queries) {
            (Ok(modified), _) => Ok(modified),
            (Err(e), Some(db_fs)) if e.kind() == ErrorKind::NotFound => {
                // no local file, try the database
                db_fs
                    .file_modified_since_in_db(app_state, path, since)
                    .await
            }
            (Err(e), _) => Err(e).with_context(|| {
                format!("Unable to read local file metadata for {}", path.display())
            }),
        }
    }

    pub async fn read_to_string(
        &self,
        app_state: &AppState,
        path: &Path,
        priviledged: bool,
    ) -> anyhow::Result<String> {
        let bytes = self.read_file(app_state, path, priviledged).await?;
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

    /**
     * Priviledged files are the ones that are in sqlpage's config directory.
     */
    pub async fn read_file(
        &self,
        app_state: &AppState,
        path: &Path,
        priviledged: bool,
    ) -> anyhow::Result<Vec<u8>> {
        let local_path = self.safe_local_path(app_state, path, priviledged)?;
        log::debug!(
            "Reading file {} from {}",
            path.display(),
            local_path.display()
        );
        let local_result = tokio::fs::read(&local_path).await;
        match (local_result, &self.db_fs_queries) {
            (Ok(f), _) => Ok(f),
            (Err(e), Some(db_fs)) if e.kind() == ErrorKind::NotFound => {
                // no local file, try the database
                db_fs.read_file(app_state, path.as_ref()).await
            }
            (Err(e), None) if e.kind() == ErrorKind::NotFound => Err(ErrorWithStatus {
                status: actix_web::http::StatusCode::NOT_FOUND,
            }
            .into()),
            (Err(e), _) => {
                Err(e).with_context(|| format!("Unable to read local file {}", path.display()))
            }
        }
    }

    fn safe_local_path(
        &self,
        app_state: &AppState,
        path: &Path,
        priviledged: bool,
    ) -> anyhow::Result<PathBuf> {
        if priviledged {
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
                return Ok(normalized);
            }
        } else {
            for (i, component) in path.components().enumerate() {
                if let Component::Normal(c) = component {
                    if i == 0 && c.eq_ignore_ascii_case("sqlpage") {
                        return Err(ErrorWithStatus {
                            status: actix_web::http::StatusCode::FORBIDDEN,
                        })
                        .with_context(|| {
                            "The /sqlpage/ path prefix is reserved for internal use. It is not public."
                        });
                    }
                    if c.as_encoded_bytes().starts_with(b".") {
                        return Err(ErrorWithStatus {
                            status: actix_web::http::StatusCode::FORBIDDEN,
                        })
                        .with_context(|| "Directory traversal is not allowed");
                    }
                } else {
                    anyhow::bail!(
                    "Unsupported path: {path:?}. Path component '{component:?}' is not allowed."
                );
                }
            }
        }
        Ok(self.local_root.join(path))
    }

    pub(crate) async fn file_exists(
        &self,
        app_state: &AppState,
        path: &Path,
    ) -> anyhow::Result<bool> {
        let local_exists = match self.safe_local_path(app_state, path, false) {
            Ok(safe_path) => tokio::fs::try_exists(safe_path).await?,
            Err(e) => return Err(e),
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

async fn file_modified_since_local(path: &Path, since: DateTime<Utc>) -> tokio::io::Result<bool> {
    tokio::fs::metadata(path)
        .await
        .and_then(|m| m.modified())
        .map(|modified_at| DateTime::<Utc>::from(modified_at) > since)
}

pub struct DbFsQueries {
    was_modified: AnyStatement<'static>,
    read_file: AnyStatement<'static>,
    exists: AnyStatement<'static>,
}

impl DbFsQueries {
    #[must_use]
    pub fn get_create_table_sql(dbms: SupportedDatabase) -> &'static str {
        match dbms {
            SupportedDatabase::Mssql => "CREATE TABLE sqlpage_files(path NVARCHAR(255) NOT NULL PRIMARY KEY, contents VARBINARY(MAX), last_modified DATETIME2(3) NOT NULL DEFAULT CURRENT_TIMESTAMP);",
            SupportedDatabase::Postgres => "CREATE TABLE IF NOT EXISTS sqlpage_files(path VARCHAR(255) NOT NULL PRIMARY KEY, contents BYTEA, last_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP);",
            _ => "CREATE TABLE IF NOT EXISTS sqlpage_files(path VARCHAR(255) NOT NULL PRIMARY KEY, contents BLOB, last_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP);",
        }
    }

    async fn init(db: &Database) -> anyhow::Result<Self> {
        log::debug!("Initializing database filesystem queries");
        Ok(Self {
            was_modified: Self::make_was_modified_query(db).await?,
            read_file: Self::make_read_file_query(db).await?,
            exists: Self::make_exists_query(db).await?,
        })
    }

    async fn make_was_modified_query(db: &Database) -> anyhow::Result<AnyStatement<'static>> {
        let was_modified_query = format!(
            "SELECT 1 from sqlpage_files WHERE last_modified >= {} AND path = {}",
            make_placeholder(db.info.kind, 1),
            make_placeholder(db.info.kind, 2)
        );
        let param_types: &[AnyTypeInfo; 2] = &[
            PgTimeTz::type_info().into(),
            <str as Type<Postgres>>::type_info().into(),
        ];
        log::debug!("Preparing the database filesystem was_modified_query: {was_modified_query}");
        db.prepare_with(&was_modified_query, param_types).await
    }

    async fn make_read_file_query(db: &Database) -> anyhow::Result<AnyStatement<'static>> {
        let read_file_query = format!(
            "SELECT contents from sqlpage_files WHERE path = {}",
            make_placeholder(db.info.kind, 1),
        );
        let param_types: &[AnyTypeInfo; 1] = &[<str as Type<Postgres>>::type_info().into()];
        log::debug!("Preparing the database filesystem read_file_query: {read_file_query}");
        db.prepare_with(&read_file_query, param_types).await
    }

    async fn make_exists_query(db: &Database) -> anyhow::Result<AnyStatement<'static>> {
        let exists_query = format!(
            "SELECT 1 from sqlpage_files WHERE path = {}",
            make_placeholder(db.info.kind, 1),
        );
        let param_types: &[AnyTypeInfo; 1] = &[<str as Type<Postgres>>::type_info().into()];
        db.prepare_with(&exists_query, param_types).await
    }

    async fn file_modified_since_in_db(
        &self,
        app_state: &AppState,
        path: &Path,
        since: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        let query = self
            .was_modified
            .query_as::<(i32,)>()
            .bind(since)
            .bind(path.display().to_string());
        log::trace!(
            "Checking if file {} was modified since {} by executing query: \n\
            {}\n\
            with parameters: {:?}",
            path.display(),
            since,
            self.was_modified.sql(),
            (since, path)
        );
        query
            .fetch_optional(&app_state.db.connection)
            .await
            .map(|modified| modified == Some((1,)))
            .with_context(|| {
                format!(
                    "Unable to check when {} was last modified in the database",
                    path.display()
                )
            })
    }

    async fn read_file(&self, app_state: &AppState, path: &Path) -> anyhow::Result<Vec<u8>> {
        log::debug!("Reading file {} from the database", path.display());
        self.read_file
            .query_as::<(Vec<u8>,)>()
            .bind(path.display().to_string())
            .fetch_optional(&app_state.db.connection)
            .await
            .map_err(anyhow::Error::from)
            .and_then(|modified| {
                if let Some((modified,)) = modified {
                    Ok(modified)
                } else {
                    Err(ErrorWithStatus {
                        status: actix_web::http::StatusCode::NOT_FOUND,
                    }
                    .into())
                }
            })
            .with_context(|| format!("Unable to read {} from the database", path.display()))
    }

    async fn file_exists(&self, app_state: &AppState, path: &Path) -> anyhow::Result<bool> {
        let query = self
            .exists
            .query_as::<(i32,)>()
            .bind(path.display().to_string());
        log::trace!(
            "Checking if file {} exists by executing query: \n\
            {}\n\
            with parameters: {:?}",
            path.display(),
            self.exists.sql(),
            (path,)
        );
        let result = query.fetch_optional(&app_state.db.connection).await;
        log::debug!("DB File exists result: {result:?}");
        result.map(|result| result.is_some()).with_context(|| {
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
    use sqlx::Executor;
    let config = app_config::tests::test_config();
    let state = AppState::init(&config).await?;
    let create_table_sql = DbFsQueries::get_create_table_sql(state.db.info.database_type);
    state
        .db
        .connection
        .execute(format!("DROP TABLE IF EXISTS sqlpage_files; {create_table_sql}").as_str())
        .await?;

    let dbms = state.db.info.kind;
    let insert_sql = format!(
        "INSERT INTO sqlpage_files(path, contents) VALUES ({}, {})",
        make_placeholder(dbms, 1),
        make_placeholder(dbms, 2)
    );
    sqlx::query(&insert_sql)
        .bind("unit test file.txt")
        .bind("Héllö world! 😀".as_bytes())
        .execute(&state.db.connection)
        .await?;

    let fs = FileSystem::init("/", &state.db).await;
    let actual = fs
        .read_to_string(&state, "unit test file.txt".as_ref(), false)
        .await?;
    assert_eq!(actual, "Héllö world! 😀");

    let one_hour_ago = Utc::now() - chrono::Duration::hours(1);
    let one_hour_future = Utc::now() + chrono::Duration::hours(1);

    let was_modified = fs
        .modified_since(&state, "unit test file.txt".as_ref(), one_hour_ago, false)
        .await?;
    assert!(was_modified, "File should be modified since one hour ago");

    let was_modified = fs
        .modified_since(
            &state,
            "unit test file.txt".as_ref(),
            one_hour_future,
            false,
        )
        .await?;
    assert!(
        !was_modified,
        "File should not be modified since one hour in the future"
    );

    Ok(())
}
