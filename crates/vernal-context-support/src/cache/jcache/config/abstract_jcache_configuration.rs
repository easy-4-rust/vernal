//! JCache 注解配置基类 — 对标 `org.springframework.cache.jcache.config.AbstractJCacheConfiguration`。

use super::jcache_configurer::JCacheConfigurer;

/// JCache 注解配置基类。
///
/// 对标 Spring 的 `AbstractJCacheConfiguration`，提供 JCache 配置的默认行为。
pub struct AbstractJCacheConfiguration {
    configurer: Option<Box<dyn JCacheConfigurer>>,
}

impl AbstractJCacheConfiguration {
    /// 创建配置。
    pub fn new() -> Self {
        Self { configurer: None }
    }

    /// 设置配置器。
    pub fn set_configurer(&mut self, configurer: Box<dyn JCacheConfigurer>) {
        self.configurer = Some(configurer);
    }
}

impl Default for AbstractJCacheConfiguration {
    fn default() -> Self {
        Self::new()
    }
}
