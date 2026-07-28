//! 缓存支持模块 — 对标 `org.springframework.cache`。
//!
//! 包含：
//! - `transaction`：事务感知缓存装饰器
//! - `caffeine`：Caffeine 缓存适配（moka 后端）
//! - `jcache`：JCache（JSR-107）适配

pub mod transaction;

#[cfg(feature = "cache")]
pub mod caffeine;

#[cfg(feature = "cache")]
pub mod jcache;

// Re-export CacheManager from vernal-cache
pub use vernal_cache::CacheManager;
