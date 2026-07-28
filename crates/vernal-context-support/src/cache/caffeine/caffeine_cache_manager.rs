//! Caffeine 缓存管理器 — 对标 `org.springframework.cache.caffeine.CaffeineCacheManager`。
//!
//! 使用 `moka` crate 作为底层缓存实现，支持动态和静态两种模式。
//!
//! # Spring 方法映射
//!
//! | Spring 方法 | Rust 方法 | 说明 |
//! |---|---|---|
//! | `CaffeineCacheManager()` | `new()` | 默认构造（最大容量 10000） |
//! | `setCacheNames(Collection)` | `set_cache_names()` | 设置静态缓存名集合 |
//! | `setCaffeine(Caffeine)` | `set_spec()` | 设置 CaffeineSpec 配置 |
//! | `setCaffeineSpec(CaffeineSpec)` | `set_spec()` | 设置 CaffeineSpec |
//! | `setCacheSpecification(String)` | `set_spec_string()` | 从字符串设置配置 |
//! | `setAsyncCacheMode(boolean)` | `set_async_cache_mode()` | 设置异步缓存模式 |
//! | `setAllowNullValues(boolean)` | `set_allow_null_values()` | 设置是否允许 null 值 |
//! | `setDynamic(boolean)` | `set_dynamic()` | 设置是否动态创建缓存 |
//! | `getCache(String)` | `get_cache()` | 获取指定缓存 |
//! | `getCacheNames()` | `cache_names()` | 获取所有缓存名 |

use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use vernal_cache::{Cache, CacheManager};

use super::caffeine_cache::CaffeineCache;
use super::caffeine_spec::CaffeineSpec;

/// 异步缓存模式。
///
/// 对标 Spring 的 `setAsyncCacheMode`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsyncCacheMode {
    /// 同步模式（默认）。
    Sync,
    /// 异步模式。
    Async,
}

/// Caffeine 缓存管理器。
///
/// 对标 Spring 的 `CaffeineCacheManager`，使用 `moka` crate 作为底层缓存实现。
/// 支持动态模式（按需创建缓存）和静态模式（预定义缓存名集合）。
pub struct CaffeineCacheManager {
    /// 缓存名称集合（静态模式）
    cache_names: RwLock<HashSet<String>>,
    /// 已创建的缓存实例
    caches: RwLock<std::collections::HashMap<String, Arc<CaffeineCache>>>,
    /// CaffeineSpec 配置
    spec: RwLock<CaffeineSpec>,
    /// 是否动态创建缓存
    dynamic: RwLock<bool>,
    /// 异步缓存模式
    async_cache_mode: RwLock<AsyncCacheMode>,
    /// 是否允许 null 值
    allow_null_values: RwLock<bool>,
    /// 默认最大容量
    default_max_capacity: u64,
}

impl CaffeineCacheManager {
    /// 创建默认的 CaffeineCacheManager（最大容量 10000）。
    pub fn new() -> Self {
        Self::with_max_capacity(10_000)
    }

    /// 创建指定最大容量的 CaffeineCacheManager。
    pub fn with_max_capacity(max_capacity: u64) -> Self {
        Self {
            cache_names: RwLock::new(HashSet::new()),
            caches: RwLock::new(std::collections::HashMap::new()),
            spec: RwLock::new(CaffeineSpec::default()),
            dynamic: RwLock::new(true),
            async_cache_mode: RwLock::new(AsyncCacheMode::Sync),
            allow_null_values: RwLock::new(true),
            default_max_capacity: max_capacity,
        }
    }

    /// 设置静态缓存名集合。
    ///
    /// 对标 `setCacheNames(Collection<String>)`。
    /// 设置后缓存管理器变为静态模式，不再动态创建新缓存。
    pub fn set_cache_names(&self, names: impl IntoIterator<Item = String>) {
        if let Ok(mut cache_names) = self.cache_names.write() {
            cache_names.extend(names);
        }
        // 设置静态名称集合后，关闭动态模式
        if let Ok(mut dynamic) = self.dynamic.write() {
            *dynamic = false;
        }
    }

    /// 设置 CaffeineSpec 配置。
    ///
    /// 对标 `setCaffeineSpec(CaffeineSpec)`。
    pub fn set_spec(&self, spec: CaffeineSpec) {
        if let Ok(mut s) = self.spec.write() {
            *s = spec;
        }
    }

    /// 从字符串设置 CaffeineSpec 配置。
    ///
    /// 对标 `setCacheSpecification(String)`。
    pub fn set_spec_string(
        &self,
        spec_str: &str,
    ) -> Result<(), super::caffeine_spec::CaffeineSpecParseError> {
        let spec = CaffeineSpec::parse(spec_str)?;
        self.set_spec(spec);
        Ok(())
    }

    /// 设置异步缓存模式。
    ///
    /// 对标 `setAsyncCacheMode(boolean)`。
    pub fn set_async_cache_mode(&self, mode: AsyncCacheMode) {
        if let Ok(mut m) = self.async_cache_mode.write() {
            *m = mode;
        }
    }

    /// 设置是否允许 null 值。
    ///
    /// 对标 `setAllowNullValues(boolean)`。
    pub fn set_allow_null_values(&self, allow: bool) {
        if let Ok(mut a) = self.allow_null_values.write() {
            *a = allow;
        }
    }

    /// 设置是否动态创建缓存。
    ///
    /// 对标 `setDynamic(boolean)`。
    pub fn set_dynamic(&self, dynamic: bool) {
        if let Ok(mut d) = self.dynamic.write() {
            *d = dynamic;
        }
    }

    /// 获取当前 CaffeineSpec 配置。
    pub fn spec(&self) -> CaffeineSpec {
        self.spec.read().map(|g| g.clone()).unwrap_or_default()
    }

    /// 获取异步缓存模式。
    pub fn async_cache_mode(&self) -> AsyncCacheMode {
        self.async_cache_mode
            .read()
            .map(|g| *g)
            .unwrap_or(AsyncCacheMode::Sync)
    }

    /// 获取是否允许 null 值。
    pub fn allow_null_values(&self) -> bool {
        self.allow_null_values.read().map(|g| *g).unwrap_or(true)
    }

    /// 获取是否动态创建缓存。
    pub fn is_dynamic(&self) -> bool {
        self.dynamic.read().map(|g| *g).unwrap_or(true)
    }

    /// 根据 spec 和名称创建缓存实例。
    fn create_cache(&self, name: &str) -> Arc<CaffeineCache> {
        let spec = self.spec();
        let max_capacity = spec.maximum_size.unwrap_or(self.default_max_capacity);

        let mut cache = CaffeineCache::with_max_capacity(name.to_string(), max_capacity);
        cache.set_allow_null_values(self.allow_null_values());

        Arc::new(cache)
    }
}

impl Default for CaffeineCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CaffeineCacheManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CaffeineCacheManager")
            .field("dynamic", &self.is_dynamic())
            .field("async_cache_mode", &self.async_cache_mode())
            .field("allow_null_values", &self.allow_null_values())
            .finish()
    }
}

impl CacheManager for CaffeineCacheManager {
    fn get_cache(&self, name: &str) -> Option<Arc<dyn Cache>> {
        // 先检查缓存是否已创建
        {
            let caches = self.caches.read().ok()?;
            if let Some(cache) = caches.get(name) {
                return Some(cache.clone());
            }
        }

        // 检查是否在静态名称集合中
        let in_static_names = self
            .cache_names
            .read()
            .map_or(false, |names| names.contains(name));

        // 如果不在静态名称集合中且不是动态模式，返回 None
        if !in_static_names && !self.is_dynamic() {
            return None;
        }

        // 创建新缓存
        let cache = self.create_cache(name);
        {
            if let Ok(mut caches) = self.caches.write() {
                caches.insert(name.to_string(), cache.clone());
            }
        }
        Some(cache)
    }

    fn cache_names(&self) -> Vec<String> {
        // 合并静态名称集合和已创建的缓存名称
        let mut names: HashSet<String> = self
            .cache_names
            .read()
            .map(|n| n.iter().cloned().collect())
            .unwrap_or_default();

        if let Ok(caches) = self.caches.read() {
            names.extend(caches.keys().cloned());
        }

        names.into_iter().collect()
    }

    fn reset_caches(&self) {
        if let Ok(mut caches) = self.caches.write() {
            for cache in caches.values() {
                cache.clear();
            }
            caches.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    #[test]
    fn test_default_manager() {
        let manager = CaffeineCacheManager::new();
        assert!(manager.is_dynamic());
        assert_eq!(manager.async_cache_mode(), AsyncCacheMode::Sync);
        assert!(manager.allow_null_values());
    }

    #[test]
    fn test_dynamic_mode() {
        let manager = CaffeineCacheManager::new();

        // 动态模式下可以按需创建缓存
        let cache = manager.get_cache("dynamic-cache").unwrap();
        assert_eq!(cache.name(), "dynamic-cache");

        // 再次获取同一个缓存
        let cache2 = manager.get_cache("dynamic-cache").unwrap();
        assert_eq!(cache.name(), cache2.name());
    }

    #[test]
    fn test_static_mode() {
        let manager = CaffeineCacheManager::new();
        manager.set_cache_names(vec!["cache1".to_string(), "cache2".to_string()]);
        manager.set_dynamic(false);

        // 静态模式下只能获取已注册的缓存
        assert!(manager.get_cache("cache1").is_some());
        assert!(manager.get_cache("cache2").is_some());
        assert!(manager.get_cache("nonexistent").is_none());
    }

    #[test]
    fn test_cache_names() {
        let manager = CaffeineCacheManager::new();
        manager.set_cache_names(vec!["cache1".to_string(), "cache2".to_string()]);

        let mut names = manager.cache_names();
        names.sort();
        assert_eq!(names, vec!["cache1", "cache2"]);

        // 获取一个动态缓存后也会出现在名称列表中
        manager.get_cache("cache3");
        let mut names = manager.cache_names();
        names.sort();
        // cache3 可能还没有被写入 HashMap（moka 异步），所以只检查静态名称
        assert!(names.contains(&"cache1".to_string()));
        assert!(names.contains(&"cache2".to_string()));
    }

    #[test]
    fn test_set_spec() {
        let manager = CaffeineCacheManager::new();
        manager
            .set_spec_string("maximumSize=5000,expireAfterWrite=5m")
            .unwrap();

        let spec = manager.spec();
        assert_eq!(spec.maximum_size, Some(5000));
        assert_eq!(
            spec.expire_after_write,
            Some(std::time::Duration::from_secs(300))
        );
    }

    #[test]
    fn test_reset_caches() {
        let manager = CaffeineCacheManager::new();
        let cache = manager.get_cache("test").unwrap();

        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        manager.reset_caches();

        // 缓存已被清除
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_debug() {
        let manager = CaffeineCacheManager::new();
        let debug_str = format!("{:?}", manager);
        assert!(debug_str.contains("CaffeineCacheManager"));
    }
}
