//! JCache 配置 SPI — 对标 `org.springframework.cache.jcache.config.JCacheConfigurer`。

use std::sync::Arc;
use vernal_cache::CacheManager;

/// JCache 配置 SPI。
///
/// 对标 Spring 的 `JCacheConfigurer`，提供自定义 JCache 配置的扩展点。
pub trait JCacheConfigurer: Send + Sync {
    /// 获取缓存管理器。
    fn cache_manager(&self) -> Option<Arc<dyn CacheManager>>;

    /// 获取缓存解析器。
    fn cache_resolver(&self) -> Option<Box<dyn std::any::Any + Send + Sync>> {
        None
    }
}
