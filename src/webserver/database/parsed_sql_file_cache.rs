use crate::webserver::database::ParsedSqlFile;
use crate::AppState;
use anyhow::Context;
use std::path::PathBuf;

use std::collections::HashMap;
use std::sync::atomic::{
    AtomicU64,
    Ordering::{Acquire, Release},
};
use std::sync::{Arc, RwLock};
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
    fn last_check_time(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH
            + Duration::from_millis(self.last_checked_at.load(Acquire) * MAX_STALE_CACHE_MS)
    }
    fn update_check_time(&self) {
        self.last_checked_at.store(Self::elapsed() as u64, Release);
    }
    fn elapsed() -> u128 {
        (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH))
            .unwrap()
            .as_millis()
            / (MAX_STALE_CACHE_MS as u128)
    }
    fn needs_check(&self) -> bool {
        self.last_check_time() + Duration::from_millis(MAX_STALE_CACHE_MS) <= SystemTime::now()
    }
}

#[derive(Default)]
pub struct FileCache<T> {
    cache: Arc<RwLock<HashMap<PathBuf, Cached<T>>>>,
}

impl FileCache<ParsedSqlFile> {
    pub async fn get(
        &self,
        app_state: &AppState,
        path: &PathBuf,
    ) -> anyhow::Result<Arc<ParsedSqlFile>> {
        {
            let read_lock = self.cache.read().expect("lock");
            if let Some(cached) = read_lock.get(path) {
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
        }
        // Read lock is released
        log::trace!("Loading and parsing {:?}", path);
        let file_contents = std::fs::read_to_string(path)
            .with_context(|| format!("Reading {:?} to load it in cache", path));
        let parsed = match file_contents {
            Ok(contents) => Ok(ParsedSqlFile::new(&app_state.db, &contents).await),
            Err(e) => Err(e),
        };

        let mut write_lock = self.cache.write().expect("write lock");
        match parsed {
            Ok(item) => {
                let value = Cached::new(item);
                let new_val = Arc::clone(&value.content);
                write_lock.insert(path.to_owned(), value);
                log::trace!("{:?} loaded in cache", path);
                Ok(new_val)
            }
            Err(e) => {
                log::trace!("Evicting {:?} from the cache", path);
                write_lock.remove(path);
                Err(e)
            }
        }
    }
}
