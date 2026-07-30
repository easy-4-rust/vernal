//! CacheManager — 缓存管理器。
use crate::cache::{Cache, SimpleCache};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 缓存管理器。
#[derive(Default)]
pub struct CacheManager {
    caches: Arc<Mutex<HashMap<String, Arc<dyn Cache>>>>,
}
impl CacheManager {
    pub fn new() -> Self { Self::default() }
    pub fn get_cache(&self, name: &str) -> Option<Arc<dyn Cache>> {
        self.caches.lock().unwrap().get(name).cloned()
    }
    pub fn create_cache(&self, name: impl Into<String>) {
        let name = name.into();
        self.caches.lock().unwrap().insert(name, Arc::new(SimpleCache::new()));
    }
    pub fn cache_names(&self) -> Vec<String> {
        self.caches.lock().unwrap().keys().cloned().collect()
    }
}

impl std::fmt::Debug for CacheManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheManager").field("cache_count", &self.caches.lock().unwrap().len()).finish()
    }
}
