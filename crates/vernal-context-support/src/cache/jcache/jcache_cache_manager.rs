//! JCache 缓存管理器 — 对标 `org.springframework.cache.jcache.JCacheCacheManager`。

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::jcache_cache::JCacheCache;
use vernal_cache::{Cache, CacheManager};

/// JCache 缓存管理器。
///
/// 对标 Spring 的 `JCacheCacheManager`，管理一组 JCache 缓存实例。
pub struct JCacheCacheManager {
    caches: RwLock<HashMap<String, Arc<JCacheCache>>>,
}

impl JCacheCacheManager {
    /// 创建 JCache 缓存管理器。
    pub fn new() -> Self {
        Self {
            caches: RwLock::new(HashMap::new()),
        }
    }

    /// 注册缓存。
    pub fn register_cache(&self, name: String, cache: Arc<JCacheCache>) {
        if let Ok(mut caches) = self.caches.write() {
            caches.insert(name, cache);
        }
    }
}

impl Default for JCacheCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for JCacheCacheManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JCacheCacheManager").finish()
    }
}

impl CacheManager for JCacheCacheManager {
    fn get_cache(&self, name: &str) -> Option<Arc<dyn Cache>> {
        self.caches
            .read()
            .ok()?
            .get(name)
            .cloned()
            .map(|c| c as Arc<dyn Cache>)
    }

    fn cache_names(&self) -> Vec<String> {
        self.caches
            .read()
            .map(|c| c.keys().cloned().collect())
            .unwrap_or_default()
    }
}
