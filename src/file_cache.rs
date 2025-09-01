use crate::webserver::routing::FileStore;
use crate::webserver::ErrorWithStatus;
use crate::AppState;
use actix_web::http::StatusCode;
use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{
    AtomicU64,
    Ordering::{Acquire, Release},
};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// The maximum time in milliseconds that a file can be cached before its freshness is checked
/// (in production mode)
const MAX_STALE_CACHE_MS: u64 = 150;

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
        self.last_checked_at
            .load(Acquire)
            .saturating_mul(MAX_STALE_CACHE_MS)
            .try_into()
            .ok()
            .and_then(|millis| Utc.timestamp_millis_opt(millis).single())
            .expect("file timestamp out of bound")
    }
    fn update_check_time(&self) {
        self.last_checked_at.store(Self::elapsed(), Release);
    }
    fn elapsed() -> u64 {
        let timestamp_millis = (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH))
            .expect("invalid duration")
            .as_millis();
        let elapsed_intervals = timestamp_millis / u128::from(MAX_STALE_CACHE_MS);
        u64::try_from(elapsed_intervals).expect("invalid date")
    }
    fn needs_check(&self) -> bool {
        self.last_checked_at
            .load(Acquire)
            .saturating_add(MAX_STALE_CACHE_MS)
            < Self::elapsed()
    }
    /// Creates a new cached entry with the same content but a new check time set to now
    fn make_fresh(&self) -> Self {
        Self {
            last_checked_at: AtomicU64::from(Self::elapsed()),
            content: Arc::clone(&self.content),
        }
    }
}

pub struct FileCache<T: AsyncFromStrWithState> {
    cache: Arc<RwLock<HashMap<PathBuf, Cached<T>>>>,
    /// Files that are loaded at the beginning of the program,
    /// and used as fallback when there is no match for the request in the file system
    static_files: HashMap<PathBuf, Cached<T>>,
}

impl<T: AsyncFromStrWithState> FileStore for FileCache<T> {
    async fn contains(&self, path: &Path) -> anyhow::Result<bool> {
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

    /// Gets a file from the cache, or loads it from the file system if it's not there
    /// This is a privileged operation; it should not be used for user-provided paths
    pub async fn get(&self, app_state: &AppState, path: &Path) -> anyhow::Result<Arc<T>> {
        self.get_with_privilege(app_state, path, true).await
    }

    pub fn get_static(&self, path: &Path) -> anyhow::Result<Arc<T>> {
        self.static_files
            .get(path)
            .map(|cached| Arc::clone(&cached.content))
            .ok_or_else(|| anyhow::anyhow!("File {} not found in static files", path.display()))
    }

    /// Gets a file from the cache, or loads it from the file system if it's not there
    /// The privileged parameter is used to determine whether the access should be denied
    /// if the file is in the sqlpage/ config directory
    pub async fn get_with_privilege(
        &self,
        app_state: &AppState,
        path: &Path,
        privileged: bool,
    ) -> anyhow::Result<Arc<T>> {
        log::trace!("Attempting to get from cache {}", path.display());
        if let Some(cached) = self.cache.read().await.get(path) {
            if app_state.config.environment.is_prod() && !cached.needs_check() {
                log::trace!(
                    "Cache answer without filesystem lookup for {}",
                    path.display()
                );
                return Ok(Arc::clone(&cached.content));
            }
            match app_state
                .file_system
                .modified_since(app_state, path, cached.last_check_time(), privileged)
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
        // Read lock is released
        log::trace!("Loading and parsing {}", path.display());
        let file_contents = app_state
            .file_system
            .read_to_string(app_state, path, privileged)
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
                self.cache.write().await.insert(PathBuf::from(path), value);
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
