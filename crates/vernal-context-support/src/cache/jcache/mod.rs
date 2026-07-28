//! JCache（JSR-107）适配 — 对标 `org.springframework.cache.jcache`。
//!
//! 使用 `cacache` 或类似 crate 作为底层实现。
//! 精简了 Spring 的 19 个拦截器类，保留核心 trait 和实现。

pub mod config;
pub mod interceptor;

mod jcache_cache;
mod jcache_cache_manager;
mod jcache_manager_factory_bean;

pub use jcache_cache::JCacheCache;
pub use jcache_cache_manager::JCacheCacheManager;
pub use jcache_manager_factory_bean::JCacheManagerFactoryBean;
