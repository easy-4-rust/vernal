//! 缓存解析适配器 — 对标 `CacheResolverAdapter`。
use std::sync::Arc;
use vernal_cache::{Cache, CacheManager};

pub struct CacheResolverAdapter {
    cache_manager: Arc<dyn CacheManager>,
}
impl CacheResolverAdapter {
    pub fn new(cache_manager: Arc<dyn CacheManager>) -> Self {
        Self { cache_manager }
    }
    pub fn resolve_cache(&self, cache_name: &str) -> Option<Arc<dyn Cache>> {
        self.cache_manager.get_cache(cache_name)
    }
}
