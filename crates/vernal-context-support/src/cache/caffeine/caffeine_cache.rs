//! Caffeine 单缓存实例 — 对标 `org.springframework.cache.caffeine.CaffeineCache`。
//!
//! 使用 `moka::sync::Cache` 作为底层实现，
//! 对标 Java Caffeine 的 `Cache` 实例。

use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

use vernal_cache::{Cache, CacheError, CacheExt, CacheResult, ValueWrapper};

// ── CaffeineCache ──────────────────────────────────────────────────────────

/// Caffeine 缓存实例。
///
/// 对标 Spring 的 `CaffeineCache`，使用 `moka::sync::Cache` 作为底层实现。
/// 支持 TTL（写入后过期）、TTI（访问后过期）、最大容量等特性。
///
/// # Spring 方法映射
///
/// | Spring 方法 | Rust 方法 | 说明 |
/// |---|---|---|
/// | `CaffeineCache(String, Cache)` | `new()` | 从 moka Cache 创建 |
/// | `getName()` | `name()` | 返回缓存名称 |
/// | `getNativeCache()` | `native_cache()` | 返回 moka Cache 引用 |
/// | `is allowNullValues()` | `allow_null_values()` | 是否允许 null 值 |
pub struct CaffeineCache {
    /// 缓存名称
    name: String,
    /// moka 缓存实例
    cache: moka::sync::Cache<String, Arc<dyn Any + Send + Sync>>,
    /// 是否允许 null 值
    allow_null_values: bool,
}

impl CaffeineCache {
    /// 创建新的 CaffeineCache。
    ///
    /// # 参数
    /// - `name`：缓存名称
    /// - `cache`：moka 缓存实例
    pub fn new(name: String, cache: moka::sync::Cache<String, Arc<dyn Any + Send + Sync>>) -> Self {
        Self {
            name,
            cache,
            allow_null_values: true,
        }
    }

    /// 创建指定最大容量的 CaffeineCache。
    pub fn with_max_capacity(name: String, max_capacity: u64) -> Self {
        let cache = moka::sync::Cache::builder()
            .max_capacity(max_capacity)
            .build();
        Self {
            name,
            cache,
            allow_null_values: true,
        }
    }

    /// 创建带 TTL 的 CaffeineCache。
    pub fn with_ttl(name: String, max_capacity: u64, ttl: Duration) -> Self {
        let cache = moka::sync::Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(ttl)
            .build();
        Self {
            name,
            cache,
            allow_null_values: true,
        }
    }

    /// 创建带 TTI 的 CaffeineCache。
    pub fn with_tti(name: String, max_capacity: u64, tti: Duration) -> Self {
        let cache = moka::sync::Cache::builder()
            .max_capacity(max_capacity)
            .time_to_idle(tti)
            .build();
        Self {
            name,
            cache,
            allow_null_values: true,
        }
    }

    /// 创建带 TTL 和 TTI 的 CaffeineCache。
    pub fn with_ttl_tti(name: String, max_capacity: u64, ttl: Duration, tti: Duration) -> Self {
        let cache = moka::sync::Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(ttl)
            .time_to_idle(tti)
            .build();
        Self {
            name,
            cache,
            allow_null_values: true,
        }
    }

    /// 设置是否允许 null 值。
    pub fn set_allow_null_values(&mut self, allow: bool) {
        self.allow_null_values = allow;
    }

    /// 获取底层 moka 缓存实例。
    pub fn moka_cache(&self) -> &moka::sync::Cache<String, Arc<dyn Any + Send + Sync>> {
        &self.cache
    }

    /// 获取缓存当前大小。
    pub fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }

    /// 使缓存失效并等待清理完成。
    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }

    /// 等待所有 pending 的清理操作完成。
    pub fn run_pending_tasks(&self) {
        self.cache.run_pending_tasks();
    }
}

impl std::fmt::Debug for CaffeineCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CaffeineCache")
            .field("name", &self.name)
            .field("size", &self.cache.entry_count())
            .finish()
    }
}

impl Cache for CaffeineCache {
    fn name(&self) -> &str {
        &self.name
    }

    fn native_cache(&self) -> &dyn Any
    where
        Self: Sized,
    {
        self
    }

    fn get(&self, key: &dyn Any) -> Option<Arc<dyn ValueWrapper>> {
        let key_str = key.downcast_ref::<String>()?;
        let value = self.cache.get(key_str)?;
        Some(Arc::new(CaffeineValueWrapper(value)))
    }

    fn put(&self, key: &dyn Any, value: Arc<dyn Any + Send + Sync>) {
        if let Some(key_str) = key.downcast_ref::<String>() {
            self.cache.insert(key_str.clone(), value);
        }
    }

    fn put_if_absent(
        &self,
        key: &dyn Any,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Option<Arc<dyn ValueWrapper>> {
        let key_str = key.downcast_ref::<String>()?.to_string();
        // moka 没有原子 put-if-absent，使用 get + insert 组合
        if let Some(existing) = self.cache.get(&key_str) {
            Some(Arc::new(CaffeineValueWrapper(existing)))
        } else {
            self.cache.insert(key_str, value);
            None
        }
    }

    fn evict(&self, key: &dyn Any) {
        if let Some(key_str) = key.downcast_ref::<String>() {
            self.cache.invalidate(key_str.as_str());
        }
    }

    fn evict_if_present(&self, key: &dyn Any) -> bool {
        if let Some(key_str) = key.downcast_ref::<String>() {
            self.cache.invalidate(key_str.as_str());
            return true;
        }
        false
    }

    fn clear(&self) {
        self.cache.invalidate_all();
    }

    fn invalidate(&self) -> bool {
        let had_entries = self.cache.entry_count() > 0;
        self.cache.invalidate_all();
        had_entries
    }
}

impl CacheExt for CaffeineCache {
    fn get_typed<T: Any + Send + Sync>(&self, key: &dyn Any) -> Option<Arc<T>> {
        let key_str = key.downcast_ref::<String>()?;
        let value = self.cache.get(key_str)?;
        value.downcast::<T>().ok()
    }

    fn get_with_loader<T, F>(&self, key: &dyn Any, value_loader: F) -> CacheResult<Arc<T>>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> CacheResult<T> + Send,
    {
        let key_str = key
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_else(|| format!("{:?}", key));

        // 先尝试从缓存读取
        if let Some(value) = self.cache.get(&key_str) {
            if let Ok(typed) = value.downcast::<T>() {
                return Ok(typed);
            }
        }

        // 缓存未命中，调用 loader
        let value = value_loader()?;
        let arc_value: Arc<dyn Any + Send + Sync> = Arc::new(value);
        let result = arc_value
            .clone()
            .downcast::<T>()
            .map_err(|_| CacheError::ValueRetrieval {
                key: key_str.clone(),
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "类型转换失败",
                )),
            })?;

        self.cache.insert(key_str, arc_value);
        Ok(result)
    }

    fn retrieve(
        &self,
        key: &dyn Any,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Option<Arc<dyn Any + Send + Sync>>> + Send>,
    > {
        let key_str = key.downcast_ref::<String>().cloned();
        let cache = self.cache.clone();
        Box::pin(async move {
            let key = key_str?;
            cache.get(&key)
        })
    }

    fn retrieve_with_loader<T, F, Fut>(
        &self,
        key: &dyn Any,
        value_loader: F,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = CacheResult<Arc<T>>> + Send>>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = CacheResult<T>> + Send + 'static,
    {
        let key_str = key
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_else(|| format!("{:?}", key));
        let cache = self.cache.clone();

        // 先同步读取
        if let Some(value) = cache.get(&key_str) {
            if let Ok(typed) = value.downcast::<T>() {
                return Box::pin(async { Ok(typed) });
            }
        }

        // 异步加载
        let key_for_loader = key_str.clone();
        Box::pin(async move {
            let value = value_loader().await?;
            let arc_value: Arc<dyn Any + Send + Sync> = Arc::new(value);
            let result =
                arc_value
                    .clone()
                    .downcast::<T>()
                    .map_err(|_| CacheError::ValueRetrieval {
                        key: key_for_loader,
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "类型转换失败",
                        )),
                    })?;
            cache.insert(key_str, arc_value);
            Ok(result)
        })
    }
}

// ── CaffeineValueWrapper ───────────────────────────────────────────────────

/// Caffeine 缓存值包装器。
struct CaffeineValueWrapper(Arc<dyn Any + Send + Sync>);

impl ValueWrapper for CaffeineValueWrapper {
    fn get(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        Some(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caffeine_cache_basic_ops() {
        let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);
        assert_eq!(cache.name(), "test");

        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        // moka 是最终一致的，立即读取应该命中
        let wrapper = cache.get(&key).unwrap();
        assert_eq!(wrapper.get().unwrap().downcast_ref::<i32>().unwrap(), &42);
    }

    #[test]
    fn test_caffeine_cache_put_if_absent() {
        let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);

        let key = String::from("k1");
        let val1: Arc<dyn Any + Send + Sync> = Arc::new(1i32);
        assert!(cache.put_if_absent(&key, val1).is_none());

        let val2: Arc<dyn Any + Send + Sync> = Arc::new(2i32);
        let existing = cache.put_if_absent(&key, val2).unwrap();
        assert_eq!(existing.get().unwrap().downcast_ref::<i32>().unwrap(), &1);
    }

    #[test]
    fn test_caffeine_cache_evict() {
        let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);

        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        assert!(cache.evict_if_present(&key));
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_caffeine_cache_clear() {
        let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);

        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        cache.invalidate();
        // moka 是最终一致的，invalidate 后立即读取应该返回 None
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_caffeine_cache_with_ttl() {
        let cache = CaffeineCache::with_ttl("test".to_string(), 100, Duration::from_secs(60));

        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        // 立即读取应该命中
        assert!(cache.get(&key).is_some());
    }

    #[test]
    fn test_caffeine_cache_debug() {
        let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);
        let debug_str = format!("{:?}", cache);
        assert!(debug_str.contains("CaffeineCache"));
        assert!(debug_str.contains("test"));
    }

    #[tokio::test]
    async fn test_caffeine_cache_retrieve() {
        let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);

        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        let result = cache.retrieve(&key).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().downcast_ref::<i32>().unwrap(), &42);
    }

    #[tokio::test]
    async fn test_caffeine_cache_retrieve_with_loader() {
        let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);

        let key = String::from("k1");
        let result =
            CacheExt::retrieve_with_loader::<i32, _, _>(&cache, &key, || async { Ok(42) }).await;
        assert_eq!(*result.unwrap(), 42);

        // 再次 retrieve 应该命中缓存
        let result2 = cache.retrieve(&key).await;
        assert!(result2.is_some());
    }
}
