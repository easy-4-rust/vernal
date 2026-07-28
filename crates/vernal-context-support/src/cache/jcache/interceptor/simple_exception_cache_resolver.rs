//! 异常缓存解析器 — 对标 `SimpleExceptionCacheResolver`。
use std::sync::Arc;
use vernal_cache::{Cache, CacheManager};

pub struct SimpleExceptionCacheResolver {
    cache_manager: Arc<dyn CacheManager>,
}
impl SimpleExceptionCacheResolver {
    pub fn new(cache_manager: Arc<dyn CacheManager>) -> Self {
        Self { cache_manager }
    }
    pub fn resolve_exception_cache(&self, cache_name: &str) -> Option<Arc<dyn Cache>> {
        self.cache_manager.get_cache(cache_name)
    }
}
