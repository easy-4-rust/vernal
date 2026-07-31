//! JCache 缓存实例 — 对标 `org.springframework.cache.jcache.JCacheCache`。
//!
//! 将 JSR-107 `javax.cache.Cache` 包装为 Spring `Cache` 接口。
//! 在 Rust 中使用 HashMap 作为简单的缓存后端。

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use vernal_cache::{Cache, CacheError, CacheExt, CacheResult, ValueWrapper};

/// JCache 缓存实例。
///
/// 对标 Spring 的 `JCacheCache`，将 JSR-107 Cache 包装为 Spring Cache 接口。
/// 在 Rust 中使用 RwLock<HashMap> 作为简单的缓存后端。
pub struct JCacheCache {
    /// 缓存名称
    name: String,
    /// 简单的缓存存储
    store: Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
    /// 是否允许 null 值
    // 对标 Spring 的 allowNullValues 属性，暂未读取（Java 镜像脚手架）。
    #[allow(dead_code)]
    allow_null_values: bool,
}

impl JCacheCache {
    /// 创建 JCache 缓存实例。
    pub fn new(name: String) -> Self {
        Self {
            name,
            store: Arc::new(RwLock::new(HashMap::new())),
            allow_null_values: true,
        }
    }

    /// 创建允许 null 值的 JCache 缓存实例。
    pub fn with_null_values(name: String, allow: bool) -> Self {
        Self {
            name,
            store: Arc::new(RwLock::new(HashMap::new())),
            allow_null_values: allow,
        }
    }

    /// 获取缓存大小。
    pub fn size(&self) -> usize {
        self.store.read().map_or(0, |s| s.len())
    }
}

impl std::fmt::Debug for JCacheCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JCacheCache")
            .field("name", &self.name)
            .field("size", &self.size())
            .finish()
    }
}

impl Cache for JCacheCache {
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
        let store = self.store.read().ok()?;
        let value = store.get(key_str)?;
        Some(Arc::new(JCacheValueWrapper(value.clone())))
    }

    fn put(&self, key: &dyn Any, value: Arc<dyn Any + Send + Sync>) {
        if let Some(key_str) = key.downcast_ref::<String>() {
            if let Ok(mut store) = self.store.write() {
                store.insert(key_str.clone(), value);
            }
        }
    }

    fn put_if_absent(
        &self,
        key: &dyn Any,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Option<Arc<dyn ValueWrapper>> {
        if let Some(key_str) = key.downcast_ref::<String>() {
            if let Ok(mut store) = self.store.write() {
                if let Some(existing) = store.get(key_str) {
                    return Some(Arc::new(JCacheValueWrapper(existing.clone())));
                }
                store.insert(key_str.clone(), value);
            }
        }
        None
    }

    fn evict(&self, key: &dyn Any) {
        if let Some(key_str) = key.downcast_ref::<String>() {
            if let Ok(mut store) = self.store.write() {
                store.remove(key_str);
            }
        }
    }

    fn evict_if_present(&self, key: &dyn Any) -> bool {
        if let Some(key_str) = key.downcast_ref::<String>() {
            if let Ok(mut store) = self.store.write() {
                return store.remove(key_str).is_some();
            }
        }
        false
    }

    fn clear(&self) {
        if let Ok(mut store) = self.store.write() {
            store.clear();
        }
    }

    fn invalidate(&self) -> bool {
        let had_entries = self.store.read().map_or(false, |s| !s.is_empty());
        if let Ok(mut store) = self.store.write() {
            store.clear();
        }
        had_entries
    }
}

impl CacheExt for JCacheCache {
    fn get_typed<T: Any + Send + Sync>(&self, key: &dyn Any) -> Option<Arc<T>> {
        let key_str = key.downcast_ref::<String>()?;
        let store = self.store.read().ok()?;
        let value = store.get(key_str)?;
        value.clone().downcast::<T>().ok()
    }

    fn get_with_loader<T, F>(&self, key: &dyn Any, value_loader: F) -> CacheResult<Arc<T>>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> CacheResult<T> + Send,
    {
        // 先尝试从缓存读取
        if let Some(result) = self.get_typed::<T>(key) {
            return Ok(result);
        }
        // 缓存未命中，调用 loader
        let value = value_loader()?;
        let key_display = key
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_else(|| format!("{:?}", key));
        let arc_value: Arc<dyn Any + Send + Sync> = Arc::new(value);
        let result = arc_value
            .clone()
            .downcast::<T>()
            .map_err(|_| CacheError::ValueRetrieval {
                key: key_display.clone(),
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "类型转换失败",
                )),
            })?;
        // 写入缓存
        if let Ok(mut store) = self.store.write() {
            store.insert(key_display, arc_value);
        }
        Ok(result)
    }

    fn retrieve(
        &self,
        key: &dyn Any,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Option<Arc<dyn Any + Send + Sync>>> + Send>,
    > {
        let result = self.get(key);
        Box::pin(async { result.and_then(|vw| vw.get()) })
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
        let key_display = key
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_else(|| format!("{:?}", key));
        // 先同步读取
        if let Some(existing) = self.get_typed::<T>(key) {
            return Box::pin(async { Ok(existing) });
        }
        // 异步加载 — clone Arc 让 async block 拥有所有权
        let store_clone = self.store.clone();
        let key_for_loader = key_display.clone();
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
            if let Ok(mut store) = store_clone.write() {
                store.insert(key_display, arc_value);
            }
            Ok(result)
        })
    }
}

/// JCache 缓存值包装器。
struct JCacheValueWrapper(Arc<dyn Any + Send + Sync>);

impl ValueWrapper for JCacheValueWrapper {
    fn get(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        Some(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jcache_cache_basic() {
        let cache = JCacheCache::new("test".to_string());
        assert_eq!(cache.name(), "test");
        assert_eq!(cache.size(), 0);

        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        assert_eq!(cache.size(), 1);
        let wrapper = cache.get(&key).unwrap();
        assert_eq!(wrapper.get().unwrap().downcast_ref::<i32>().unwrap(), &42);
    }

    #[test]
    fn test_jcache_cache_put_if_absent() {
        let cache = JCacheCache::new("test".to_string());
        let key = String::from("k1");

        let val1: Arc<dyn Any + Send + Sync> = Arc::new(1i32);
        assert!(cache.put_if_absent(&key, val1).is_none());

        let val2: Arc<dyn Any + Send + Sync> = Arc::new(2i32);
        let existing = cache.put_if_absent(&key, val2).unwrap();
        assert_eq!(existing.get().unwrap().downcast_ref::<i32>().unwrap(), &1);
    }

    #[test]
    fn test_jcache_cache_evict() {
        let cache = JCacheCache::new("test".to_string());
        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        assert!(cache.evict_if_present(&key));
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_jcache_cache_clear() {
        let cache = JCacheCache::new("test".to_string());
        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        assert!(cache.invalidate());
        assert_eq!(cache.size(), 0);
    }
}
