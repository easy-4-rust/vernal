//! JCache 配置支持 — 对标 `org.springframework.cache.jcache.config`。

mod abstract_jcache_configuration;
mod jcache_configurer;
mod jcache_configurer_support;
mod proxy_jcache_configuration;

pub use abstract_jcache_configuration::AbstractJCacheConfiguration;
pub use jcache_configurer::JCacheConfigurer;
pub use jcache_configurer_support::JCacheConfigurerSupport;
pub use proxy_jcache_configuration::ProxyJCacheConfiguration;
