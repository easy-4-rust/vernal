//! JCache 代理配置 — 对标 `org.springframework.cache.jcache.config.ProxyJCacheConfiguration`。

use super::abstract_jcache_configuration::AbstractJCacheConfiguration;

/// JCache 代理配置。
///
/// 对标 Spring 的 `ProxyJCacheConfiguration`，启用 @CacheResult 等注解支持。
pub struct ProxyJCacheConfiguration {
    // 对标 Spring 的内部配置，暂未读取（Java 镜像脚手架）。
    #[allow(dead_code)]
    inner: AbstractJCacheConfiguration,
}

impl ProxyJCacheConfiguration {
    /// 创建代理配置。
    pub fn new() -> Self {
        Self {
            inner: AbstractJCacheConfiguration::new(),
        }
    }
}

impl Default for ProxyJCacheConfiguration {
    fn default() -> Self {
        Self::new()
    }
}
