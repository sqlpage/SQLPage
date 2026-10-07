use crate::AppState;
use crate::filesystem::FileAccess;
use crate::webserver::ErrorWithStatus;
use crate::webserver::routing::FileStore;
use actix_web::http::StatusCode;
use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{
    AtomicU64,
    Ordering::{Acquire, Release},
};
use std::time::SystemTime;
use tokio::sync::RwLock;

#[derive(Default)]
struct Cached<T> {
    last_checked_at: AtomicU64,
    content: Arc<T>,
}

impl<T> Cached<T> {
    fn new(content: T) -> Self {
        let s = Self {
            last_checked_at: AtomicU64::new(0),
            content: Arc::new(content),
        };
        s.update_check_time();
        s
    }
    fn last_check_time(&self) -> DateTime<Utc> {
        let millis = self.last_checked_at.load(Acquire);
        let as_i64 = i64::try_from(millis).expect("file timestamp out of bound");
        Utc.timestamp_millis_opt(as_i64)
            .single()
            .expect("utc has a single mapping for every timestamp")
    }
    fn update_check_time(&self) {
        self.last_checked_at.store(Self::now_millis(), Release);
    }
    fn now_millis() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("invalid duration")
            .as_millis()
            .try_into()
            .expect("invalid date")
    }
    fn needs_check(&self, stale_cache_duration_ms: u64) -> bool {
        self.last_checked_at
            .load(Acquire)
            .saturating_add(stale_cache_duration_ms)
            < Self::now_millis()
    }
    /// Creates a new cached entry with the same content but a new check time set to now
    fn make_fresh(&self) -> Self {
        Self {
            last_checked_at: AtomicU64::from(Self::now_millis()),
            content: Arc::clone(&self.content),
        }
    }
}

pub struct FileCache<T: AsyncFromStrWithState> {
    cache: Arc<RwLock<HashMap<PathBuf, Arc<Cached<T>>>>>,
    /// Files that are loaded at the beginning of the program,
    /// and used as fallback when there is no match for the request in the file system
    static_files: HashMap<PathBuf, Cached<T>>,
}

impl<T: AsyncFromStrWithState> FileStore for FileCache<T> {
    async fn contains(&self, access: FileAccess<'_>) -> anyhow::Result<bool> {
        let path = access.path();
        Ok(self.cache.read().await.contains_key(path) || self.static_files.contains_key(path))
    }
}

impl<T: AsyncFromStrWithState> Default for FileCache<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: AsyncFromStrWithState> FileCache<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: Arc::default(),
            static_files: HashMap::new(),
        }
    }

    /// Adds a static file to the cache so that it will never be looked up from the disk
    pub fn add_static(&mut self, path: PathBuf, contents: T) {
        log::trace!("Adding static file {} to the cache.", path.display());
        self.static_files.insert(path, Cached::new(contents));
    }

    pub fn get_static(&self, path: &Path) -> anyhow::Result<Arc<T>> {
        self.static_files
            .get(path)
            .map(|cached| Arc::clone(&cached.content))
            .ok_or_else(|| anyhow::anyhow!("File {} not found in static files", path.display()))
    }

    /// Gets a file from the cache, or loads it from the file system if it's not there.
    pub async fn get(
        &self,
        app_state: &AppState,
        access: FileAccess<'_>,
    ) -> anyhow::Result<Arc<T>> {
        let path = access.path();

        log::trace!("Attempting to get from cache {}", path.display());
        // Keep a snapshot so metadata I/O cannot block cache insertions or evictions.
        let cached = self.cache.read().await.get(path).cloned();
        if let Some(cached) = cached {
            if !cached.needs_check(app_state.config.cache_stale_duration_ms()) {
                log::trace!(
                    "Cache answer without filesystem lookup for {}",
                    path.display()
                );
                return Ok(Arc::clone(&cached.content));
            }
            match app_state
                .file_system
                .modified_since(app_state, access, cached.last_check_time())
                .await
            {
                Ok(false) => {
                    log::trace!(
                        "Cache answer with filesystem metadata read for {}",
                        path.display()
                    );
                    cached.update_check_time();
                    return Ok(Arc::clone(&cached.content));
                }
                Ok(true) => log::trace!("{} was changed, updating cache...", path.display()),
                Err(e) => log::trace!(
                    "Cannot read metadata of {}, re-loading it: {:#}",
                    path.display(),
                    e
                ),
            }
        }
        log::trace!("Loading and parsing {}", path.display());
        let file_contents = app_state
            .file_system
            .read_to_string(app_state, access)
            .await;

        let parsed = match file_contents {
            Ok(contents) => {
                let value = T::from_str_with_state(app_state, &contents, path).await?;
                Ok(Cached::new(value))
            }
            // If a file is not found, we try to load it from the static files
            Err(e)
                if e.downcast_ref()
                    == Some(&ErrorWithStatus {
                        status: StatusCode::NOT_FOUND,
                    }) =>
            {
                if let Some(static_file) = self.static_files.get(path) {
                    log::trace!(
                        "File {} not found, loading it from static files instead.",
                        path.display()
                    );
                    let cached: Cached<T> = static_file.make_fresh();
                    Ok(cached)
                } else {
                    Err(e)
                        .with_context(|| format!("Couldn't load \"{}\" into cache", path.display()))
                }
            }
            Err(e) => {
                Err(e).with_context(|| format!("Couldn't load {} into cache", path.display()))
            }
        };

        match parsed {
            Ok(value) => {
                let new_val = Arc::clone(&value.content);
                log::trace!("Writing to cache {}", path.display());
                self.cache
                    .write()
                    .await
                    .insert(PathBuf::from(path), Arc::new(value));
                log::trace!("Done writing to cache {}", path.display());
                log::trace!("{} loaded in cache", path.display());
                Ok(new_val)
            }
            Err(e) => {
                log::trace!(
                    "Evicting {} from the cache because the following error occurred: {}",
                    path.display(),
                    e
                );
                log::trace!("Removing from cache {}", path.display());
                self.cache.write().await.remove(path);
                log::trace!("Done removing from cache {}", path.display());
                Err(e)
            }
        }
    }
}

#[async_trait(? Send)]
pub trait AsyncFromStrWithState: Sized {
    /// Parses the string into an object.
    async fn from_str_with_state(
        app_state: &AppState,
        source: &str,
        source_path: &Path,
    ) -> anyhow::Result<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::DbFsQueries;
    use crate::templates::{SplitTemplate, split_template};
    use crate::webserver::Database;
    use handlebars::{Template, template::TemplateElement};
    use sqlx::executor::Executor;
    use std::time::Duration;

    fn template(source: &str) -> SplitTemplate {
        split_template(Template::compile(source).unwrap())
    }

    async fn state_in(dir: &Path, database_files: bool) -> anyhow::Result<AppState> {
        let mut config = crate::app_config::tests::test_config();
        config.database_url = "sqlite::memory:".to_owned();
        config.max_database_pool_connections = Some(1);
        config.cache_stale_duration_ms = Some(1000);
        config.web_root = dir.to_owned();
        config.configuration_directory = dir.join("sqlpage");
        let db = Database::init(&config).await?;
        if database_files {
            db.connection
                .execute(DbFsQueries::get_create_table_sql(db.info.database_type))
                .await?;
        }
        AppState::init_with_db(&config, db).await
    }

    async fn mark_stale(cache: &FileCache<SplitTemplate>, path: &Path) {
        cache.cache.read().await[path]
            .last_checked_at
            .store(0, Release);
    }

    #[actix_web::test]
    async fn pending_metadata_check_allows_writes_without_restoring_old_entries()
    -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let state = state_in(dir.path(), true).await?;
        let path = Path::new("cached.handlebars");
        for replace in [false, true] {
            let cache = FileCache::new();
            let snapshot = Arc::new(Cached::new(template("original")));
            snapshot.last_checked_at.store(0, Release);
            cache
                .cache
                .write()
                .await
                .insert(path.to_owned(), Arc::clone(&snapshot));
            // Missing local metadata must await the held database connection.
            let connection = state.db.connection.acquire().await?;
            let lookup = cache.get(&state, FileAccess::unprivileged(path)?);
            tokio::pin!(lookup);
            assert!(futures_util::poll!(&mut lookup).is_pending());
            let replacement = Arc::new(Cached::new(template("replacement")));
            let replacement_checked_at = replacement.last_checked_at.load(Acquire);
            {
                let mut entries =
                    tokio::time::timeout(Duration::from_secs(1), cache.cache.write()).await?;
                let unrelated = PathBuf::from("unrelated.handlebars");
                entries.insert(unrelated.clone(), Arc::clone(&replacement));
                entries.remove(&unrelated);
                if replace {
                    entries.insert(path.to_owned(), Arc::clone(&replacement));
                } else {
                    entries.remove(path);
                }
            }
            drop(connection);
            let result = tokio::time::timeout(Duration::from_secs(1), &mut lookup).await??;
            assert!(Arc::ptr_eq(&result, &snapshot.content));
            assert!(snapshot.last_checked_at.load(Acquire) > 0);
            let entries = cache.cache.read().await;
            if replace {
                assert!(Arc::ptr_eq(&entries[path], &replacement));
                assert_eq!(
                    replacement.last_checked_at.load(Acquire),
                    replacement_checked_at
                );
            } else {
                assert!(
                    !entries.contains_key(path),
                    "eviction must remain effective"
                );
            }
        }
        Ok(())
    }

    #[actix_web::test]
    async fn refreshes_local_database_and_static_templates_in_order() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let mut state = state_in(dir.path(), true).await?;
        let mut cache = FileCache::new();
        let path = Path::new("file.handlebars");
        let access = FileAccess::unprivileged(path)?;
        let local_path = dir.path().join(path);
        cache.add_static(path.to_owned(), template("static"));
        sqlx::query::query("INSERT INTO sqlpage_files(path, contents) VALUES (?, ?)")
            .bind(path.to_str())
            .bind(b"database".as_slice())
            .execute(&state.db.connection)
            .await?;
        let mut previous = None;
        for source in ["first", "changed", "database", "static"] {
            match source {
                "database" => tokio::fs::remove_file(&local_path).await?,
                "static" => {
                    state
                        .db
                        .connection
                        .execute("DROP TABLE sqlpage_files")
                        .await?;
                    state.file_system =
                        crate::filesystem::FileSystem::init(dir.path(), &state.db).await;
                }
                _ => tokio::fs::write(&local_path, source).await?,
            }
            if previous.is_some() {
                mark_stale(&cache, path).await;
            }
            let loaded = cache.get(&state, access).await?;
            if let Some(old) = &previous {
                assert!(!Arc::ptr_eq(old, &loaded));
            }
            assert_eq!(
                loaded.before_list.elements,
                [TemplateElement::RawString(source.into())]
            );
            assert!(Arc::ptr_eq(&loaded, &cache.get(&state, access).await?));
            previous = Some(loaded);
        }
        assert!(Arc::ptr_eq(&previous.unwrap(), &cache.get_static(path)?));
        Ok(())
    }

    #[actix_web::test]
    async fn parse_errors_retain_entries_and_io_errors_evict_them() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let state = state_in(dir.path(), false).await?;
        let path = Path::new("file.handlebars");
        let access = FileAccess::unprivileged(path)?;
        let local_path = dir.path().join(path);
        for (failure, status, retain) in [
            ("parse", None, true),
            ("missing", Some(StatusCode::NOT_FOUND), false),
            ("metadata", Some(StatusCode::INTERNAL_SERVER_ERROR), false),
        ] {
            let mut cache = FileCache::new();
            if failure != "missing" {
                cache.add_static(path.to_owned(), template("static"));
            }
            tokio::fs::write(&local_path, "valid").await?;
            let initial = cache.get(&state, access).await?;
            mark_stale(&cache, path).await;
            match failure {
                "parse" => tokio::fs::write(&local_path, "{{#if missing}}").await?,
                "missing" => tokio::fs::remove_file(&local_path).await?,
                _ => {
                    tokio::fs::remove_file(&local_path).await?;
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(path, &local_path)?;
                    #[cfg(not(unix))]
                    continue;
                }
            }
            let error = cache
                .get(&state, access)
                .await
                .err()
                .expect("expected load failure");
            assert_eq!(
                error
                    .downcast_ref::<ErrorWithStatus>()
                    .map(|error| error.status),
                status
            );
            let entries = cache.cache.read().await;
            assert_eq!(entries.contains_key(path), retain, "{failure}");
            if retain {
                assert!(error.downcast_ref::<handlebars::TemplateError>().is_some());
                assert!(Arc::ptr_eq(&entries[path].content, &initial));
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_cache_duration() {
        let cached = Cached::new(());
        assert!(
            !cached.needs_check(1000),
            "Should not need check immediately after creation"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(
            !cached.needs_check(1000),
            "Should not need check before duration expires"
        );
        assert!(
            cached.needs_check(1),
            "Should need check after duration expires"
        );
    }
}
