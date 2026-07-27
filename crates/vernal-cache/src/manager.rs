//! 缓存管理器 trait — 对标 Spring 的 `org.springframework.cache.CacheManager`。

use std::sync::Arc;

use super::cache::Cache;

/// 缓存管理器 trait — 对标 Spring 的 `org.springframework.cache.CacheManager`。
///
/// 管理一组命名的 `Cache` 实例。
///
/// # Spring 方法映射
///
/// | Spring 方法 | Rust 方法 | 说明 |
/// |---|---|---|
/// | `getCache(String)` | `get_cache()` | 获取或创建指定名称的缓存 |
/// | `getCacheNames()` | `cache_names()` | 返回所有已知缓存名称 |
/// | `resetCaches()` | `reset_caches()` | 清空所有已注册缓存 |
pub trait CacheManager: Send + Sync {
    /// 获取指定名称的缓存。
    ///
    /// 对标 `getCache(String name)`。
    /// 如果缓存不存在且支持动态创建，则创建并返回；否则返回 `None`。
    fn get_cache(&self, name: &str) -> Option<Arc<dyn Cache>>;

    /// 获取所有已知缓存名称列表。
    ///
    /// 对标 `getCacheNames()`。
    fn cache_names(&self) -> Vec<String>;

    /// 清空所有已注册缓存。
    ///
    /// 对标 `resetCaches()`（Java 7.0.2+）。
    /// 默认实现：遍历 `cache_names()`，对每个缓存调用 `clear()`。
    fn reset_caches(&self) {
        let names = self.cache_names();
        for name in names {
            if let Some(cache) = self.get_cache(&name) {
                cache.clear();
            }
        }
    }
}
