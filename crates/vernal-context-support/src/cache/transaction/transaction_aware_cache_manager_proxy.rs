//! 事务感知缓存管理器代理 — 对标 `org.springframework.cache.transaction.TransactionAwareCacheManagerProxy`。
//!
//! 代理 `CacheManager`，将每个返回的 `Cache` 包装为 `TransactionAwareCacheDecorator`。
//! 这样所有通过此代理获取的缓存都会自动与事务同步。

use std::sync::Arc;

use vernal_cache::{Cache, CacheManager};

use super::transaction_aware_cache_decorator::TransactionAwareCacheDecorator;

/// 事务感知缓存管理器代理。
///
/// 对标 Spring 的 `TransactionAwareCacheManagerProxy`。
/// 代理 `CacheManager`，将每个返回的 `Cache` 包装为 `TransactionAwareCacheDecorator`。
///
/// # Spring 方法映射
///
/// | Spring 方法 | Rust 方法 | 说明 |
/// |---|---|---|
/// | `TransactionAwareCacheManagerProxy(CacheManager)` | `new()` | 构造代理 |
/// | `getTargetCacheManager()` | `target_cache_manager()` | 返回被代理的 CacheManager |
/// | `getCache(String)` | `get_cache()` | 返回事务感知的 Cache |
/// | `getCacheNames()` | `cache_names()` | 委托给目标 CacheManager |
pub struct TransactionAwareCacheManagerProxy {
    target_cache_manager: Arc<dyn CacheManager>,
}

impl TransactionAwareCacheManagerProxy {
    /// 创建事务感知缓存管理器代理。
    ///
    /// # 参数
    /// - `target_cache_manager`：被代理的缓存管理器
    pub fn new(target_cache_manager: Arc<dyn CacheManager>) -> Self {
        Self {
            target_cache_manager,
        }
    }

    /// 返回被代理的缓存管理器。
    ///
    /// 对标 `getTargetCacheManager()`。
    pub fn target_cache_manager(&self) -> &Arc<dyn CacheManager> {
        &self.target_cache_manager
    }
}

impl CacheManager for TransactionAwareCacheManagerProxy {
    fn get_cache(&self, name: &str) -> Option<Arc<dyn Cache>> {
        // 获取目标缓存并包装为事务感知装饰器
        self.target_cache_manager
            .get_cache(name)
            .map(|cache| Arc::new(TransactionAwareCacheDecorator::new(cache)) as Arc<dyn Cache>)
    }

    fn cache_names(&self) -> Vec<String> {
        self.target_cache_manager.cache_names()
    }

    fn reset_caches(&self) {
        self.target_cache_manager.reset_caches();
    }
}

impl std::fmt::Debug for TransactionAwareCacheManagerProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionAwareCacheManagerProxy").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::RwLock;

    struct TestCacheManager {
        caches: RwLock<HashMap<String, Arc<dyn Cache>>>,
    }

    impl TestCacheManager {
        fn new() -> Self {
            Self {
                caches: RwLock::new(HashMap::new()),
            }
        }

        fn register(&self, name: &str, cache: Arc<dyn Cache>) {
            self.caches.write().unwrap().insert(name.to_string(), cache);
        }
    }

    impl CacheManager for TestCacheManager {
        fn get_cache(&self, name: &str) -> Option<Arc<dyn Cache>> {
            self.caches.read().unwrap().get(name).cloned()
        }

        fn cache_names(&self) -> Vec<String> {
            self.caches.read().unwrap().keys().cloned().collect()
        }
    }

    #[test]
    fn test_proxy_returns_transaction_aware_cache() {
        let manager = Arc::new(TestCacheManager::new());
        let cache = Arc::new(vernal_cache::SimpleCache::new("test"));
        manager.register("test", cache);

        let proxy = TransactionAwareCacheManagerProxy::new(manager);

        // 通过代理获取缓存，应该返回 TransactionAwareCacheDecorator
        let cache = proxy.get_cache("test").unwrap();
        assert_eq!(cache.name(), "test");

        // 写入应该被包装为事务感知操作
        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);
    }

    #[test]
    fn test_proxy_cache_names() {
        let manager = Arc::new(TestCacheManager::new());
        let cache1 = Arc::new(vernal_cache::SimpleCache::new("cache1"));
        let cache2 = Arc::new(vernal_cache::SimpleCache::new("cache2"));
        manager.register("cache1", cache1);
        manager.register("cache2", cache2);

        let proxy = TransactionAwareCacheManagerProxy::new(manager);
        let mut names = proxy.cache_names();
        names.sort();
        assert_eq!(names, vec!["cache1", "cache2"]);
    }

    #[test]
    fn test_proxy_missing_cache() {
        let manager = Arc::new(TestCacheManager::new());
        let proxy = TransactionAwareCacheManagerProxy::new(manager);
        assert!(proxy.get_cache("nonexistent").is_none());
    }

    #[test]
    fn test_proxy_target_cache_manager() {
        let manager = Arc::new(TestCacheManager::new());
        let proxy = TransactionAwareCacheManagerProxy::new(manager.clone());
        assert_eq!(
            proxy.target_cache_manager().cache_names(),
            manager.cache_names()
        );
    }

    #[test]
    fn test_proxy_debug() {
        let manager = Arc::new(TestCacheManager::new());
        let proxy = TransactionAwareCacheManagerProxy::new(manager);
        let debug_str = format!("{:?}", proxy);
        assert!(debug_str.contains("TransactionAwareCacheManagerProxy"));
    }

    use std::any::Any;
}
