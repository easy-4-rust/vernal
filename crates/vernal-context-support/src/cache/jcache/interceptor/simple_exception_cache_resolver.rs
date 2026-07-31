//! 异常缓存解析器 — 对标 `SimpleExceptionCacheResolver`。
use std::sync::Arc;
use vernal_cache::{Cache, CacheManager};

/// 异常缓存解析器。
pub struct SimpleExceptionCacheResolver {
    cache_manager: Arc<dyn CacheManager>,
}
impl SimpleExceptionCacheResolver {
    /// 创建异常缓存解析器。
    pub fn new(cache_manager: Arc<dyn CacheManager>) -> Self {
        Self { cache_manager }
    }
    /// 解析异常缓存名称并返回对应的缓存实例。
    pub fn resolve_exception_cache(&self, cache_name: &str) -> Option<Arc<dyn Cache>> {
        self.cache_manager.get_cache(cache_name)
    }
}
