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
    use crate::webserver::Database;
    use sqlx::executor::Executor;
    use std::time::Duration;
    use tempfile::TempDir;

    #[async_trait(?Send)]
    impl AsyncFromStrWithState for String {
        async fn from_str_with_state(
            _app_state: &AppState,
            source: &str,
            _source_path: &Path,
        ) -> anyhow::Result<Self> {
            anyhow::ensure!(source != "invalid", "invalid cached content");
            Ok(source.to_owned())
        }
    }

    async fn state_in(dir: &TempDir, database_files: bool) -> anyhow::Result<AppState> {
        let mut config = crate::app_config::tests::test_config();
        // These tests deliberately use an isolated one-connection pool to control I/O.
        config.database_url = "sqlite::memory:".to_owned();
        config.max_database_pool_connections = Some(1);
        config.cache_stale_duration_ms = Some(1000);
        config.web_root = dir.path().to_owned();
        config.configuration_directory = dir.path().join("sqlpage");
        let db = Database::init(&config).await?;
        if database_files {
            db.connection
                .execute(DbFsQueries::get_create_table_sql(db.info.database_type))
                .await?;
        }
        AppState::init_with_db(&config, db).await
    }

    async fn mark_stale(cache: &FileCache<String>, path: &Path) {
        cache.cache.read().await[path]
            .last_checked_at
            .store(0, Release);
    }

    #[actix_web::test]
    async fn pending_metadata_check_allows_writes_without_restoring_old_entries()
    -> anyhow::Result<()> {
        let dir = TempDir::new()?;
        let state = state_in(&dir, true).await?;
        let path = Path::new("cached.txt");
        let unrelated = Path::new("unrelated.txt");
        for replace in [false, true] {
            let cache = FileCache::<String>::new();
            let snapshot = Arc::new(Cached::new("original".to_owned()));
            snapshot.last_checked_at.store(0, Release);
            cache
                .cache
                .write()
                .await
                .insert(path.to_owned(), Arc::clone(&snapshot));

            // No local file exists: metadata must wait for this database connection.
            let connection = state.db.connection.acquire().await?;
            let lookup = cache.get(&state, FileAccess::unprivileged(path)?);
            tokio::pin!(lookup);
            assert!(
                tokio::time::timeout(Duration::from_millis(50), &mut lookup)
                    .await
                    .is_err(),
                "metadata lookup should wait for the held connection"
            );

            let replacement = Arc::new(Cached::new("replacement".to_owned()));
            let replacement_checked_at = replacement.last_checked_at.load(Acquire);
            {
                let mut entries = tokio::time::timeout(Duration::from_secs(1), cache.cache.write())
                    .await
                    .expect("pending metadata must not block cache writes");
                entries.insert(
                    unrelated.to_owned(),
                    Arc::new(Cached::new("unrelated".to_owned())),
                );
                entries.remove(unrelated);
                if replace {
                    entries.insert(path.to_owned(), Arc::clone(&replacement));
                } else {
                    entries.remove(path);
                }
            }

            drop(connection);
            let result = tokio::time::timeout(Duration::from_secs(1), &mut lookup).await??;
            assert_eq!(*result, "original");
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
    async fn refreshes_changed_files_and_falls_back_to_static_content() -> anyhow::Result<()> {
        let dir = TempDir::new()?;
        let state = state_in(&dir, false).await?;
        let mut cache = FileCache::<String>::new();
        let path = Path::new("file.txt");
        let local_path = dir.path().join(path);
        cache.add_static(path.to_owned(), "static".to_owned());
        tokio::fs::write(&local_path, "first").await?;

        let initial = cache.get(&state, FileAccess::unprivileged(path)?).await?;
        assert_eq!(*initial, "first");
        let unchanged = cache.get(&state, FileAccess::unprivileged(path)?).await?;
        assert!(Arc::ptr_eq(&initial, &unchanged));

        tokio::fs::write(&local_path, "changed").await?;
        mark_stale(&cache, path).await;
        let changed = cache.get(&state, FileAccess::unprivileged(path)?).await?;
        assert_eq!(*changed, "changed");
        assert!(!Arc::ptr_eq(&initial, &changed));

        tokio::fs::remove_file(&local_path).await?;
        mark_stale(&cache, path).await;
        let fallback = cache.get(&state, FileAccess::unprivileged(path)?).await?;
        assert_eq!(*fallback, "static");
        assert!(Arc::ptr_eq(&fallback, &cache.get_static(path)?));
        Ok(())
    }

    #[actix_web::test]
    async fn local_files_take_precedence_over_database_and_static_files() -> anyhow::Result<()> {
        let dir = TempDir::new()?;
        let state = state_in(&dir, true).await?;
        let mut cache = FileCache::<String>::new();
        let path = Path::new("file.txt");
        cache.add_static(path.to_owned(), "static".to_owned());
        sqlx::query::query("INSERT INTO sqlpage_files(path, contents) VALUES (?, ?)")
            .bind("file.txt")
            .bind(b"database".as_slice())
            .execute(&state.db.connection)
            .await?;
        let local_path = dir.path().join(path);
        tokio::fs::write(&local_path, "local").await?;

        let local = cache.get(&state, FileAccess::unprivileged(path)?).await?;
        assert_eq!(*local, "local");
        tokio::fs::remove_file(&local_path).await?;
        mark_stale(&cache, path).await;
        let database = cache.get(&state, FileAccess::unprivileged(path)?).await?;
        assert_eq!(*database, "database");
        Ok(())
    }

    #[actix_web::test]
    async fn missing_files_evict_entries_and_parse_errors_preserve_them() -> anyhow::Result<()> {
        let dir = TempDir::new()?;
        let state = state_in(&dir, false).await?;
        let cache = FileCache::<String>::new();
        let path = Path::new("file.txt");
        let local_path = dir.path().join(path);
        tokio::fs::write(&local_path, "valid").await?;
        let initial = cache.get(&state, FileAccess::unprivileged(path)?).await?;

        tokio::fs::write(&local_path, "invalid").await?;
        mark_stale(&cache, path).await;
        let error = cache
            .get(&state, FileAccess::unprivileged(path)?)
            .await
            .unwrap_err();
        assert_eq!(error.to_string(), "invalid cached content");
        assert!(Arc::ptr_eq(
            &cache.cache.read().await[path].content,
            &initial
        ));

        tokio::fs::remove_file(&local_path).await?;
        let error = cache
            .get(&state, FileAccess::unprivileged(path)?)
            .await
            .unwrap_err();
        assert_eq!(
            error.downcast_ref::<ErrorWithStatus>(),
            Some(&ErrorWithStatus {
                status: StatusCode::NOT_FOUND
            })
        );
        assert!(!cache.cache.read().await.contains_key(path));
        Ok(())
    }

    #[cfg(unix)]
    #[actix_web::test]
    async fn metadata_and_read_errors_do_not_use_static_fallback() -> anyhow::Result<()> {
        let dir = TempDir::new()?;
        let state = state_in(&dir, false).await?;
        let mut cache = FileCache::<String>::new();
        let path = Path::new("file.txt");
        let local_path = dir.path().join(path);
        cache.add_static(path.to_owned(), "static".to_owned());
        tokio::fs::write(&local_path, "valid").await?;
        cache.get(&state, FileAccess::unprivileged(path)?).await?;
        tokio::fs::remove_file(&local_path).await?;
        std::os::unix::fs::symlink(path, &local_path)?;
        mark_stale(&cache, path).await;

        let error = cache
            .get(&state, FileAccess::unprivileged(path)?)
            .await
            .unwrap_err();
        assert_eq!(
            error.downcast_ref::<ErrorWithStatus>(),
            Some(&ErrorWithStatus {
                status: StatusCode::INTERNAL_SERVER_ERROR
            })
        );
        assert!(!cache.cache.read().await.contains_key(path));
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
