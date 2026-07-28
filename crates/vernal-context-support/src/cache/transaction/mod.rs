//! 事务感知缓存装饰器 — 对标 `org.springframework.cache.transaction`。
//!
//! 包含：
//! - `TransactionAwareCacheDecorator`：将 put/evict/clear 操作延迟到事务 after-commit
//! - `TransactionAwareCacheManagerProxy`：装饰 CacheManager，返回事务感知 Cache

mod transaction_aware_cache_decorator;
mod transaction_aware_cache_manager_proxy;

pub use transaction_aware_cache_decorator::{
    ImmediateCallbackRegistrar, TransactionAwareCacheDecorator, TransactionCallbackRegistrar,
    TransactionStatus,
};
pub use transaction_aware_cache_manager_proxy::TransactionAwareCacheManagerProxy;
