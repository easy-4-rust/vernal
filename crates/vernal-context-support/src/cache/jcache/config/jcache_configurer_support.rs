//! JCache 配置 SPI 默认实现 — 对标 `org.springframework.cache.jcache.config.JCacheConfigurerSupport`。

use super::jcache_configurer::JCacheConfigurer;
use std::sync::Arc;
use vernal_cache::CacheManager;

/// JCache 配置 SPI 默认实现。
///
/// 对标 Spring 的 `JCacheConfigurerSupport`，提供 `JCacheConfigurer` 的默认实现。
pub struct JCacheConfigurerSupport;

impl JCacheConfigurer for JCacheConfigurerSupport {
    fn cache_manager(&self) -> Option<Arc<dyn CacheManager>> {
        None
    }
}
