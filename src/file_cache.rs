use crate::AppState;
use anyhow::Context;
use async_trait::async_trait;
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::atomic::{
    AtomicU64,
    Ordering::{Acquire, Release},
};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

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
    /// Creates a cached entry that will never need checking
    fn new_static(content: T) -> Self {
        let this = Self::new(content);
        this.last_checked_at.store(u64::MAX, Release);
        this
    }
    fn last_check_time(&self) -> SystemTime {
        let millis = self
            .last_checked_at
            .load(Acquire)
            .saturating_mul(MAX_STALE_CACHE_MS);
        SystemTime::UNIX_EPOCH + Duration::from_millis(millis)
    }
    fn update_check_time(&self) {
        let elapsed = u64::try_from(Self::elapsed()).expect("too far in the future");
        self.last_checked_at.store(elapsed, Release);
    }
    fn elapsed() -> u128 {
        (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH))
            .unwrap()
            .as_millis()
            / u128::from(MAX_STALE_CACHE_MS)
    }
    fn needs_check(&self) -> bool {
        self.last_check_time() + Duration::from_millis(MAX_STALE_CACHE_MS) <= SystemTime::now()
    }
}

pub struct FileCache<T: AsyncFromStrWithState> {
    cache: Arc<DashMap<PathBuf, Cached<T>>>,
}

impl<T: AsyncFromStrWithState> FileCache<T> {
    pub fn new() -> Self {
        Self {
            cache: Arc::default(),
        }
    }

    /// Adds a static file to the cache so that it will never be looked up from the disk
    pub fn add_static(&mut self, path: PathBuf, contents: T) {
        log::trace!("Adding static file {path:?} to the cache.");
        let cached = Cached::new_static(contents);
        self.cache.insert(path, cached);
    }

    pub async fn get(&self, app_state: &AppState, path: &PathBuf) -> anyhow::Result<Arc<T>> {
        if let Some(cached) = self.cache.get(path) {
            if !cached.needs_check() {
                log::trace!("Cache answer without filesystem lookup for {:?}", path);
                return Ok(Arc::clone(&cached.content));
            }
            if let Ok(modified) = std::fs::metadata(path).and_then(|m| m.modified()) {
                if modified <= cached.last_check_time() {
                    log::trace!("Cache answer with filesystem metadata read for {:?}", path);
                    cached.update_check_time();
                    return Ok(Arc::clone(&cached.content));
                }
            }
        }
        // Read lock is released
        log::trace!("Loading and parsing {:?}", path);
        let file_contents = std::fs::read_to_string(path)
            .with_context(|| format!("Reading {path:?} to load it in cache"));
        let parsed = match file_contents {
            Ok(contents) => Ok(T::from_str_with_state(app_state, &contents).await?),
            Err(e) => Err(e),
        };

        match parsed {
            Ok(item) => {
                let value = Cached::new(item);
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
