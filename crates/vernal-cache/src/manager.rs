//! 缓存管理器 trait。

use std::sync::Arc;

use super::cache::Cache;

/// 缓存管理器 trait。
///
/// 对标 Spring 的 `CacheManager`。
pub trait CacheManager: Send + Sync {
    /// 获取缓存名称列表。
    fn cache_names(&self) -> Vec<String>;

    /// 获取指定名称的缓存。
    fn get_cache(&self, name: &str) -> Option<Arc<dyn Cache>>;
}
