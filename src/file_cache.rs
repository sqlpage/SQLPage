use crate::webserver::ErrorWithStatus;
use crate::AppState;
use actix_web::http::StatusCode;
use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{
    AtomicU64,
    Ordering::{Acquire, Release},
};
use std::sync::Arc;
use std::time::SystemTime;

const MAX_STALE_CACHE_MS: u64 = 100;

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
    cache: Arc<DashMap<PathBuf, Cached<T>>>,
    /// Files that are loaded at the beginning of the program,
    /// and used as fallback when there is no match for the request in the file system
    static_files: HashMap<PathBuf, Cached<T>>,
}

impl<T: AsyncFromStrWithState> FileCache<T> {
    pub fn new() -> Self {
        Self {
            cache: Arc::default(),
            static_files: HashMap::new(),
        }
    }

    /// Adds a static file to the cache so that it will never be looked up from the disk
    pub fn add_static(&mut self, path: PathBuf, contents: T) {
        log::trace!("Adding static file {path:?} to the cache.");
        self.static_files.insert(path, Cached::new(contents));
    }

    pub async fn get(&self, app_state: &AppState, path: &PathBuf) -> anyhow::Result<Arc<T>> {
        if let Some(cached) = self.cache.get(path) {
            if !cached.needs_check() {
                log::trace!("Cache answer without filesystem lookup for {:?}", path);
                return Ok(Arc::clone(&cached.content));
            }
            match app_state
                .file_system
                .modified_since(app_state, path, cached.last_check_time())
                .await
            {
                Ok(false) => {
                    log::trace!("Cache answer with filesystem metadata read for {:?}", path);
                    cached.update_check_time();
                    return Ok(Arc::clone(&cached.content));
                }
                Ok(true) => log::trace!("{path:?} was changed, updating cache..."),
                Err(e) => log::trace!("Cannot read metadata of {path:?}, re-loading it: {e:#}"),
            }
        }
        // Read lock is released
        log::trace!("Loading and parsing {:?}", path);
        let file_contents = app_state.file_system.read_to_string(app_state, path).await;

        let parsed = match file_contents {
            Ok(contents) => {
                let value = T::from_str_with_state(app_state, &contents).await?;
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
                    log::trace!("File {path:?} not found, loading it from static files instead.");
                    let cached: Cached<T> = static_file.make_fresh();
                    Ok(cached)
                } else {
                    Err(e).with_context(|| format!("Couldn't load {path:?} into cache"))
                }
            }
            Err(e) => Err(e).with_context(|| format!("Couldn't load {path:?} into cache")),
        };

        match parsed {
            Ok(value) => {
                let new_val = Arc::clone(&value.content);
                self.cache.insert(path.clone(), value);
                log::trace!("{:?} loaded in cache", path);
                Ok(new_val)
            }
            Err(e) => {
                log::trace!(
                    "Evicting {path:?} from the cache because the following error occurred: {e}"
                );
                self.cache.remove(path);
                Err(e)
            }
        }
    }
}

#[async_trait(? Send)]
pub trait AsyncFromStrWithState: Sized {
    async fn from_str_with_state(app_state: &AppState, source: &str) -> anyhow::Result<Self>;
}
