//! 缓存解析适配器 — 对标 `CacheResolverAdapter`。
use std::sync::Arc;
use vernal_cache::{Cache, CacheManager};

/// 缓存解析适配器。
pub struct CacheResolverAdapter {
    cache_manager: Arc<dyn CacheManager>,
}
impl CacheResolverAdapter {
    /// 创建缓存解析适配器。
    pub fn new(cache_manager: Arc<dyn CacheManager>) -> Self {
        Self { cache_manager }
    }
    /// 解析缓存名称并返回对应的缓存实例。
    pub fn resolve_cache(&self, cache_name: &str) -> Option<Arc<dyn Cache>> {
        self.cache_manager.get_cache(cache_name)
    }
}
