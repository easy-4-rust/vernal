//! 缓存 trait — 对标 Spring 的 `org.springframework.cache.Cache`。
//!
//! 该模块定义了 Vernal 缓存抽象层，包含：
//! - `Cache` trait：缓存读写接口（dyn-compatible）
//! - `ValueWrapper`：缓存值包装器
//! - `CacheError`：缓存错误类型
//! - `SimpleCache`：基于 HashMap 的简单缓存实现（仅测试用）

use std::any::Any;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// ── 缓存错误类型 ──────────────────────────────────────────────────────────

/// 缓存错误类型。
///
/// 对标 Spring 的 `IllegalStateException` / `IllegalArgumentException` / `CacheException`。
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    /// 缓存值加载失败。
    #[error("缓存值加载失败（key={key}）：{source}")]
    ValueRetrieval {
        /// 加载失败的 key
        key: String,
        /// 底层异常
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// 缓存操作失败。
    #[error("缓存操作失败：{0}")]
    OperationFailed(String),

    /// 缓存不支持异步操作。
    #[error("缓存不支持异步操作：{0}")]
    AsyncUnsupported(String),
}

/// 缓存操作结果类型别名。
pub type CacheResult<T> = Result<T, CacheError>;

// ── ValueWrapper ───────────────────────────────────────────────────────────

/// 缓存值包装器。
///
/// 对标 Spring 的 `Cache.ValueWrapper`。
/// 用于从 `Cache::get` 返回缓存值，支持空值语义。
pub trait ValueWrapper: Send + Sync {
    /// 返回实际的缓存值。
    ///
    /// 返回 `None` 表示缓存中存储的是显式 null 值。
    fn get(&self) -> Option<Arc<dyn Any + Send + Sync>>;
}

/// 简单值包装器实现。
pub struct SimpleValueWrapper {
    value: Option<Arc<dyn Any + Send + Sync>>,
}

impl SimpleValueWrapper {
    /// 创建新的值包装器。
    pub fn new(value: Option<Arc<dyn Any + Send + Sync>>) -> Self {
        Self { value }
    }
}

impl ValueWrapper for SimpleValueWrapper {
    fn get(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.value.clone()
    }
}

impl fmt::Debug for SimpleValueWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SimpleValueWrapper")
            .field("has_value", &self.value.is_some())
            .finish()
    }
}

// ── Cache trait（dyn-compatible）───────────────────────────────────────────

/// 缓存 trait — 对标 Spring 的 `org.springframework.cache.Cache`。
///
/// 提供缓存读写接口。此 trait 是 dyn-compatible 的（可以作为 trait object 使用）。
///
/// # Spring 方法映射
///
/// | Spring 方法 | Rust 方法 | 说明 |
/// |---|---|---|
/// | `getName()` | `name()` | 返回缓存名称 |
/// | `getNativeCache()` | `native_cache()` | 返回底层原生缓存对象 |
/// | `get(Object)` | `get()` | 按 key 读取，返回 ValueWrapper |
/// | `put(Object, Object)` | `put()` | 写入缓存 |
/// | `putIfAbsent(Object, Object)` | `put_if_absent()` | 原子 put-if-absent |
/// | `evict(Object)` | `evict()` | 失效缓存条目 |
/// | `evictIfPresent(Object)` | `evict_if_present()` | 条件失效 |
/// | `clear()` | `clear()` | 清空缓存 |
/// | `invalidate()` | `invalidate()` | 原子清空 |
pub trait Cache: Send + Sync {
    /// 获取缓存名称。
    ///
    /// 对标 `getName()`。
    fn name(&self) -> &str;

    /// 获取底层原生缓存对象。
    ///
    /// 对标 `getNativeCache()`。
    /// 默认返回 `self` 的 Any trait 对象引用（需要 Self: Sized）。
    fn native_cache(&self) -> &dyn Any
    where
        Self: Sized;

    /// 按 key 读取缓存值，返回 ValueWrapper 包装。
    ///
    /// 对标 `get(Object key)`。
    /// 返回 `None` 表示缓存未命中。
    fn get(&self, key: &dyn Any) -> Option<Arc<dyn ValueWrapper>>;

    /// 写入缓存值。
    ///
    /// 对标 `put(Object key, Object value)`。
    fn put(&self, key: &dyn Any, value: Arc<dyn Any + Send + Sync>);

    /// 原子 put-if-absent。
    ///
    /// 对标 `putIfAbsent(Object key, Object value)`（Java 4.1+）。
    /// 返回已存在的值（如果 key 已存在），否则写入新值并返回 `None`。
    fn put_if_absent(
        &self,
        key: &dyn Any,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Option<Arc<dyn ValueWrapper>> {
        // 默认实现：get -> put，非原子
        if let Some(existing) = self.get(key) {
            Some(existing)
        } else {
            self.put(key, value);
            None
        }
    }

    /// 失效缓存条目。
    ///
    /// 对标 `evict(Object key)`。
    fn evict(&self, key: &dyn Any);

    /// 条件失效缓存条目。
    ///
    /// 对标 `evictIfPresent(Object key)`（Java 5.2+）。
    /// 返回 `true` 表示条目存在并被失效。
    fn evict_if_present(&self, key: &dyn Any) -> bool {
        self.evict(key);
        false
    }

    /// 清空缓存。
    ///
    /// 对标 `clear()`。
    fn clear(&self);

    /// 原子清空缓存。
    ///
    /// 对标 `invalidate()`（Java 5.2+）。
    /// 返回 `true` 表示缓存中有条目被清除。
    fn invalidate(&self) -> bool {
        self.clear();
        false
    }
}

// ── CacheExt 扩展 trait（泛型方法）────────────────────────────────────────

/// 缓存扩展 trait — 提供泛型方法。
///
/// 因为 `Cache` trait 需要 dyn-compatible（可以作为 trait object），
/// 泛型方法被分离到此 trait 中。
///
/// # Spring 方法映射
///
/// | Spring 方法 | Rust 方法 | 说明 |
/// |---|---|---|
/// | `get(Object, Class<T>)` | `get_typed()` | 按 key 和类型读取 |
/// | `get(Object, Callable<T>)` | `get_with_loader()` | 缺失时通过 Callable 加载 |
/// | `retrieve(Object)` | `retrieve()` | 异步读取 |
/// | `retrieve(Object, Supplier)` | `retrieve_with_loader()` | 异步读取 + 异步加载 |
pub trait CacheExt: Cache {
    /// 按 key 和类型读取缓存值。
    ///
    /// 对标 `get(Object key, Class<T> type)`（Java 4.0+）。
    /// 返回 `None` 表示缓存未命中或类型不匹配。
    fn get_typed<T: Any + Send + Sync>(&self, key: &dyn Any) -> Option<Arc<T>>;

    /// 按 key 读取缓存值，缺失时通过 valueLoader 加载。
    ///
    /// 对标 `get(Object key, Callable<T> valueLoader)`（Java 4.3+）。
    /// 如果缓存未命中，调用 `valueLoader` 计算值，写入缓存并返回。
    /// 加载失败时抛出 `CacheError::ValueRetrieval`。
    fn get_with_loader<T, F>(&self, key: &dyn Any, value_loader: F) -> CacheResult<Arc<T>>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> CacheResult<T> + Send;

    /// 异步读取缓存值。
    ///
    /// 对标 `retrieve(Object key)`（Java 6.1+）。
    /// 返回一个 Future，解析为缓存值或 `None`（未命中）。
    fn retrieve(
        &self,
        key: &dyn Any,
    ) -> Pin<Box<dyn Future<Output = Option<Arc<dyn Any + Send + Sync>>> + Send>>;

    /// 异步读取缓存值，缺失时通过 valueLoader 异步加载。
    ///
    /// 对标 `retrieve(Object key, Supplier<CompletableFuture<T>> valueLoader)`（Java 6.1+）。
    fn retrieve_with_loader<T, F, Fut>(
        &self,
        key: &dyn Any,
        value_loader: F,
    ) -> Pin<Box<dyn Future<Output = CacheResult<Arc<T>>> + Send>>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = CacheResult<T>> + Send + 'static;
}

// ── 简单 Cache 实现 ───────────────────────────────────────────────────────

/// 基于 HashMap 的简单缓存实现（用于测试）。
///
/// 仅用于单元测试，生产环境应使用 `CaffeineCache`（moka 后端）。
pub struct SimpleCache {
    name: String,
    store: Arc<std::sync::RwLock<std::collections::HashMap<String, Arc<dyn Any + Send + Sync>>>>,
}

impl SimpleCache {
    /// 创建新的简单缓存。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            store: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

impl fmt::Debug for SimpleCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SimpleCache")
            .field("name", &self.name)
            .finish()
    }
}

impl Cache for SimpleCache {
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
        Some(Arc::new(SimpleValueWrapper::new(Some(value.clone()))))
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
                    return Some(Arc::new(SimpleValueWrapper::new(Some(existing.clone()))));
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

impl CacheExt for SimpleCache {
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
    ) -> Pin<Box<dyn Future<Output = Option<Arc<dyn Any + Send + Sync>>> + Send>> {
        let result = self.get(key);
        Box::pin(async { result.and_then(|vw| vw.get()) })
    }

    fn retrieve_with_loader<T, F, Fut>(
        &self,
        key: &dyn Any,
        value_loader: F,
    ) -> Pin<Box<dyn Future<Output = CacheResult<Arc<T>>> + Send>>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = CacheResult<T>> + Send + 'static,
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
        let key_owned = key_display.clone();
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
                store.insert(key_owned, arc_value);
            }
            Ok(result)
        })
    }
}

// ── 类型化缓存值封装 ──────────────────────────────────────────────────────

/// 类型化缓存值。
///
/// 用于将具体类型值包装为 `Arc<dyn Any + Send + Sync>`。
pub struct TypedCacheValue<T: Send + Sync> {
    value: T,
}

impl<T: Send + Sync + 'static> TypedCacheValue<T> {
    /// 创建类型化缓存值。
    pub fn new(value: T) -> Self {
        Self { value }
    }

    /// 获取内部值的引用。
    pub fn inner(&self) -> &T {
        &self.value
    }

    /// 获取内部值的所有权。
    pub fn into_inner(self) -> T {
        self.value
    }

    /// 转换为 Arc<dyn Any + Send + Sync>。
    pub fn into_arc(self) -> Arc<dyn Any + Send + Sync> {
        Arc::new(self.value)
    }
}

impl<T: Send + Sync + Clone + 'static> TypedCacheValue<T> {
    /// 从 Arc<dyn Any + Send + Sync> 尝试克隆提取值。
    pub fn from_arc(arc: &Arc<dyn Any + Send + Sync>) -> Option<T> {
        arc.downcast_ref::<T>().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_cache_get_put() {
        let cache = SimpleCache::new("test");
        assert_eq!(cache.name(), "test");

        let key = String::from("k1");
        assert!(cache.get(&key).is_none());

        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        let wrapper = cache.get(&key).unwrap();
        let retrieved = wrapper.get().unwrap();
        assert_eq!(retrieved.downcast_ref::<i32>().unwrap(), &42);
    }

    #[test]
    fn simple_cache_put_if_absent() {
        let cache = SimpleCache::new("test");
        let key = String::from("k1");

        // 第一次 put_if_absent 返回 None（key 不存在）
        let val1: Arc<dyn Any + Send + Sync> = Arc::new(1i32);
        assert!(cache.put_if_absent(&key, val1).is_none());

        // 第二次 put_if_absent 返回已存在的值
        let val2: Arc<dyn Any + Send + Sync> = Arc::new(2i32);
        let existing = cache.put_if_absent(&key, val2).unwrap();
        assert_eq!(existing.get().unwrap().downcast_ref::<i32>().unwrap(), &1);

        // 值仍然是第一次的
        let wrapper = cache.get(&key).unwrap();
        assert_eq!(wrapper.get().unwrap().downcast_ref::<i32>().unwrap(), &1);
    }

    #[test]
    fn simple_cache_evict() {
        let cache = SimpleCache::new("test");
        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        assert!(cache.evict_if_present(&key));
        assert!(cache.get(&key).is_none());

        // 不存在的 key 返回 false
        assert!(!cache.evict_if_present(&key));
    }

    #[test]
    fn simple_cache_clear() {
        let cache = SimpleCache::new("test");
        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        assert!(cache.invalidate());
        assert!(cache.get(&key).is_none());

        // 空缓存 invalidate 返回 false
        assert!(!cache.invalidate());
    }

    #[test]
    fn simple_cache_get_typed() {
        let cache = SimpleCache::new("test");
        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        let typed = CacheExt::get_typed::<i32>(&cache, &key).unwrap();
        assert_eq!(*typed, 42);

        // 类型不匹配返回 None
        let wrong = CacheExt::get_typed::<String>(&cache, &key);
        assert!(wrong.is_none());
    }

    #[test]
    fn simple_cache_get_with_loader() {
        let cache = SimpleCache::new("test");
        let key = String::from("k1");

        let result = CacheExt::get_with_loader::<i32, _>(&cache, &key, || Ok(42));
        assert_eq!(*result.unwrap(), 42);

        // 再次获取应该命中缓存
        let result2 = CacheExt::get_with_loader::<i32, _>(&cache, &key, || Ok(99));
        assert_eq!(*result2.unwrap(), 42);
    }

    #[tokio::test]
    async fn simple_cache_retrieve() {
        let cache = SimpleCache::new("test");
        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        cache.put(&key, value);

        let result = cache.retrieve(&key).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().downcast_ref::<i32>().unwrap(), &42);

        // 未命中
        let key2 = String::from("missing");
        assert!(cache.retrieve(&key2).await.is_none());
    }

    #[tokio::test]
    async fn simple_cache_retrieve_with_loader() {
        let cache = SimpleCache::new("test");
        let key = String::from("k1");

        let result =
            CacheExt::retrieve_with_loader::<i32, _, _>(&cache, &key, || async { Ok(42) }).await;
        assert_eq!(*result.unwrap(), 42);
    }
}
