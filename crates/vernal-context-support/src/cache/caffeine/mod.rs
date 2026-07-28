//! Caffeine 缓存适配（moka 后端）— 对标 `org.springframework.cache.caffeine`。
//!
//! 使用 `moka` crate 作为 Rust 端的高性能本地缓存实现，
//! 对标 Java 生态的 Caffeine 缓存库。
//!
//! 包含：
//! - `CaffeineCache`：单个缓存实例
//! - `CaffeineCacheManager`：缓存管理器
//! - `CaffeineSpec`：CaffeineSpec 字符串配置解析
//! - `AsyncCacheMode`：异步缓存模式

mod caffeine_cache;
mod caffeine_cache_manager;
mod caffeine_spec;
mod moka_adapters;

pub use caffeine_cache::CaffeineCache;
pub use caffeine_cache_manager::{AsyncCacheMode, CaffeineCacheManager};
pub use caffeine_spec::{CaffeineSpec, CaffeineSpecParseError};
pub use moka_adapters::MokaCacheAdapter;
