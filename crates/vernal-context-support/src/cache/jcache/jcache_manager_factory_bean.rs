//! JCache 管理器工厂 — 对标 `org.springframework.cache.jcache.JCacheManagerFactoryBean`。

use super::jcache_cache_manager::JCacheCacheManager;
use std::sync::Arc;

/// JCache 管理器工厂 Bean。
///
/// 对标 Spring 的 `JCacheManagerFactoryBean`，创建并配置 `JCacheCacheManager`。
pub struct JCacheManagerFactoryBean {
    cache_manager: Option<Arc<JCacheCacheManager>>,
}

impl JCacheManagerFactoryBean {
    /// 创建工厂 Bean。
    pub fn new() -> Self {
        Self {
            cache_manager: None,
        }
    }

    /// 初始化并返回缓存管理器。
    pub fn after_properties_set(&mut self) {
        self.cache_manager = Some(Arc::new(JCacheCacheManager::new()));
    }

    /// 获取缓存管理器。
    pub fn object(&self) -> Option<Arc<JCacheCacheManager>> {
        self.cache_manager.clone()
    }
}

impl Default for JCacheManagerFactoryBean {
    fn default() -> Self {
        Self::new()
    }
}
