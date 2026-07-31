//! 事务感知缓存装饰器 — 对标 `org.springframework.cache.transaction.TransactionAwareCacheDecorator`。
//!
//! 将 `put`、`evict` 和 `clear` 操作与 Spring 管理的事务同步，
//! 仅在事务成功提交的 after-commit 阶段才执行实际的缓存操作。
//! 如果没有活动的事务，则立即执行。
//!
//! # 注意
//!
//! `put_if_absent` 和 `evict_if_present` 这类即时操作**无法**延迟到 after-commit，
//! 在事务环境中请谨慎使用。
//!
//! # Spring 方法映射
//!
//! | Spring 方法 | Rust 方法 | 说明 |
//! |---|---|---|
//! | `TransactionAwareCacheDecorator(Cache)` | `new()` | 构造装饰器 |
//! | `getTargetCache()` | `target_cache()` | 返回被装饰的目标缓存 |
//! | `getName()` | `name()` | 委托给目标缓存 |
//! | `getNativeCache()` | `native_cache()` | 委托给目标缓存 |
//! | `get(Object)` | `get()` | 立即委托（读操作不延迟） |
//! | `put(Object, Object)` | `put()` | 延迟到 after-commit |
//! | `evict(Object)` | `evict()` | 延迟到 after-commit |
//! | `clear()` | `clear()` | 延迟到 after-commit |
//! | `putIfAbsent(Object, Object)` | `put_if_absent()` | 立即执行（即时操作） |
//! | `evictIfPresent(Object)` | `evict_if_present()` | 立即执行（即时操作） |
//! | `invalidate()` | `invalidate()` | 立即执行（即时操作） |

use std::any::Any;
use std::sync::Arc;
use std::sync::Mutex;

use vernal_cache::{Cache, CacheError, CacheExt, ValueWrapper};

// ── 事务状态 ───────────────────────────────────────────────────────────────

/// 事务状态。
///
/// 用于跟踪当前是否处于事务中。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    /// 无活动事务。
    NoTransaction,
    /// 事务已开始，尚未提交。
    Active,
    /// 事务已提交（after-commit 阶段）。
    Committed,
    /// 事务已回滚。
    RolledBack,
}

// ── 事务回调注册器 ─────────────────────────────────────────────────────────

/// 事务回调注册器 trait。
///
/// 在实际项目中，这会与 `vernal-context` 的事务管理器集成。
/// 默认实现提供立即执行的 fallback。
pub trait TransactionCallbackRegistrar: Send + Sync {
    /// 注册 afterCommit 回调。
    ///
    /// 如果当前有活动事务，回调将在事务提交后执行。
    /// 如果没有活动事务，回调将立即执行。
    fn after_commit(&self, callback: Box<dyn FnOnce() + Send + 'static>);

    /// 注册 afterCompletion 回调。
    ///
    /// 回调将在事务完成后（无论提交还是回滚）执行。
    fn after_completion(&self, callback: Box<dyn FnOnce(TransactionStatus) + Send + 'static>);

    /// 获取当前事务状态。
    fn status(&self) -> TransactionStatus;
}

/// 立即执行的回调注册器（无事务环境）。
pub struct ImmediateCallbackRegistrar;

impl TransactionCallbackRegistrar for ImmediateCallbackRegistrar {
    fn after_commit(&self, callback: Box<dyn FnOnce() + Send + 'static>) {
        // 无事务，立即执行
        callback();
    }

    fn after_completion(&self, callback: Box<dyn FnOnce(TransactionStatus) + Send + 'static>) {
        // 无事务，视为已提交
        callback(TransactionStatus::Committed);
    }

    fn status(&self) -> TransactionStatus {
        TransactionStatus::NoTransaction
    }
}

// ── TransactionAwareCacheDecorator ─────────────────────────────────────────

/// 事务感知缓存装饰器。
///
/// 对应 Spring 的 `TransactionAwareCacheDecorator`，其 `put` / `evict` / `clear`
/// 操作会延迟到事务成功提交后才执行；无事务时立即执行。
///
/// # 注意
///
/// `put_if_absent` 和 `evict_if_present` 这类即时操作**无法**延迟到 after-commit，
/// 在事务环境中请谨慎使用。
pub struct TransactionAwareCacheDecorator {
    /// 被装饰的目标缓存
    target_cache: Arc<dyn Cache>,
    /// 事务回调注册器
    registrar: Arc<dyn TransactionCallbackRegistrar>,
    /// 待执行的 after-commit 回调队列
    deferred_callbacks: Mutex<Vec<Box<dyn FnOnce() + Send>>>,
}

impl TransactionAwareCacheDecorator {
    /// 创建事务感知缓存装饰器（使用立即执行回调注册器）。
    ///
    /// # 参数
    /// - `target_cache`：被装饰的目标缓存
    pub fn new(target_cache: Arc<dyn Cache>) -> Self {
        Self {
            target_cache,
            registrar: Arc::new(ImmediateCallbackRegistrar),
            deferred_callbacks: Mutex::new(Vec::new()),
        }
    }

    /// 创建事务感知缓存装饰器（使用自定义回调注册器）。
    ///
    /// # 参数
    /// - `target_cache`：被装饰的目标缓存
    /// - `registrar`：事务回调注册器
    pub fn with_registrar(
        target_cache: Arc<dyn Cache>,
        registrar: Arc<dyn TransactionCallbackRegistrar>,
    ) -> Self {
        Self {
            target_cache,
            registrar,
            deferred_callbacks: Mutex::new(Vec::new()),
        }
    }

    /// 返回被装饰的目标缓存。
    ///
    /// 对标 `getTargetCache()`。
    pub fn target_cache(&self) -> &Arc<dyn Cache> {
        &self.target_cache
    }

    /// 注册 after-commit 回调。
    ///
    /// 如果有活动事务，回调将在事务提交后执行。
    /// 如果没有活动事务，回调立即执行。
    fn after_commit(&self, callback: Box<dyn FnOnce() + Send + 'static>) {
        match self.registrar.status() {
            TransactionStatus::NoTransaction => {
                // 无事务，立即执行
                callback();
            }
            TransactionStatus::Active => {
                // 有活动事务，延迟执行
                if let Ok(mut callbacks) = self.deferred_callbacks.lock() {
                    callbacks.push(callback);
                }
            }
            TransactionStatus::Committed | TransactionStatus::RolledBack => {
                // 事务已完成，立即执行
                callback();
            }
        }
    }

    /// 执行所有延迟的 after-commit 回调。
    ///
    /// 在事务成功提交后调用此方法。
    pub fn execute_deferred(&self) {
        if let Ok(mut callbacks) = self.deferred_callbacks.lock() {
            for callback in callbacks.drain(..) {
                callback();
            }
        }
    }

    /// 清空所有延迟的回调（用于事务回滚）。
    pub fn discard_deferred(&self) {
        if let Ok(mut callbacks) = self.deferred_callbacks.lock() {
            callbacks.clear();
        }
    }
}

impl std::fmt::Debug for TransactionAwareCacheDecorator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionAwareCacheDecorator")
            .field("target_cache_name", &self.target_cache.name())
            .finish()
    }
}

impl Cache for TransactionAwareCacheDecorator {
    fn name(&self) -> &str {
        self.target_cache.name()
    }

    fn native_cache(&self) -> &dyn Any
    where
        Self: Sized,
    {
        self
    }

    fn get(&self, key: &dyn Any) -> Option<Arc<dyn ValueWrapper>> {
        // 读操作立即执行，不延迟
        self.target_cache.get(key)
    }

    fn put(&self, key: &dyn Any, value: Arc<dyn Any + Send + Sync>) {
        // 写操作延迟到 after-commit
        let target = self.target_cache.clone();
        // key 需要 clone 以支持 FnOnce
        let key_owned: Arc<dyn Any + Send + Sync> = if let Some(s) = key.downcast_ref::<String>() {
            Arc::new(s.clone())
        } else {
            // 对于非 String key，尝试 clone Any
            Arc::new(format!("{:?}", key))
        };
        self.after_commit(Box::new(move || {
            target.put(&*key_owned, value);
        }));
    }

    fn evict(&self, key: &dyn Any) {
        // 失效操作延迟到 after-commit
        let target = self.target_cache.clone();
        let key_owned: Arc<dyn Any + Send + Sync> = if let Some(s) = key.downcast_ref::<String>() {
            Arc::new(s.clone())
        } else {
            Arc::new(format!("{:?}", key))
        };
        self.after_commit(Box::new(move || {
            target.evict(&*key_owned);
        }));
    }

    fn clear(&self) {
        // 清空操作延迟到 after-commit
        let target = self.target_cache.clone();
        self.after_commit(Box::new(move || {
            target.clear();
        }));
    }

    fn put_if_absent(
        &self,
        key: &dyn Any,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Option<Arc<dyn ValueWrapper>> {
        // 即时操作，不延迟（Spring 语义：putIfAbsent 无法延迟到 after-commit）
        self.target_cache.put_if_absent(key, value)
    }

    fn evict_if_present(&self, key: &dyn Any) -> bool {
        // 即时操作，不延迟
        self.target_cache.evict_if_present(key)
    }

    fn invalidate(&self) -> bool {
        // 即时操作，不延迟
        self.target_cache.invalidate()
    }
}

impl CacheExt for TransactionAwareCacheDecorator {
    fn get_typed<T: Any + Send + Sync>(&self, key: &dyn Any) -> Option<Arc<T>> {
        // 直接委托给目标缓存的 get 方法，然后尝试 downcast
        let wrapper = self.target_cache.get(key)?;
        let value = wrapper.get()?;
        value.downcast::<T>().ok()
    }

    fn get_with_loader<T, F>(&self, key: &dyn Any, value_loader: F) -> Result<Arc<T>, CacheError>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> Result<T, CacheError> + Send,
    {
        // 先尝试直接读取
        if let Some(typed) = self.get_typed::<T>(key) {
            return Ok(typed);
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
        self.target_cache.put(key, arc_value);
        Ok(result)
    }

    fn retrieve(
        &self,
        key: &dyn Any,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Option<Arc<dyn Any + Send + Sync>>> + Send>,
    > {
        let result = self.target_cache.get(key);
        Box::pin(async { result.and_then(|vw| vw.get()) })
    }

    fn retrieve_with_loader<T, F, Fut>(
        &self,
        key: &dyn Any,
        value_loader: F,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Arc<T>, CacheError>> + Send>>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<T, CacheError>> + Send + 'static,
    {
        let key_display = key
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_else(|| format!("{:?}", key));
        let target = self.target_cache.clone();
        let key_for_loader = key_display.clone();

        // 先同步读取
        if let Some(existing) = self.get_typed::<T>(key) {
            return Box::pin(async { Ok(existing) });
        }

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
            let key_arc: Arc<dyn Any + Send + Sync> = Arc::new(key_display);
            target.put(&*key_arc, arc_value);
            Ok(result)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immediate_registrar() {
        let cache = Arc::new(vernal_cache::SimpleCache::new("test"));
        let decorator = TransactionAwareCacheDecorator::new(cache.clone());

        assert_eq!(decorator.name(), "test");

        // 立即执行模式：put 立即生效
        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        decorator.put(&key, value.clone());

        // 读取应该立即可见（因为无事务）
        let wrapper = decorator.get(&key).unwrap();
        assert_eq!(wrapper.get().unwrap().downcast_ref::<i32>().unwrap(), &42);
    }

    #[test]
    fn test_immediate_registrar_evict() {
        let cache = Arc::new(vernal_cache::SimpleCache::new("test"));
        let decorator = TransactionAwareCacheDecorator::new(cache.clone());

        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        decorator.put(&key, value);

        // 立即 evict
        decorator.evict(&key);
        assert!(decorator.get(&key).is_none());
    }

    #[test]
    fn test_immediate_registrar_clear() {
        let cache = Arc::new(vernal_cache::SimpleCache::new("test"));
        let decorator = TransactionAwareCacheDecorator::new(cache.clone());

        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        decorator.put(&key, value);

        decorator.clear();
        assert!(decorator.get(&key).is_none());
    }

    #[test]
    fn test_put_if_absent_not_deferred() {
        let cache = Arc::new(vernal_cache::SimpleCache::new("test"));
        let decorator = TransactionAwareCacheDecorator::new(cache);

        let key = String::from("k1");

        // 第一次 put_if_absent：key 不存在，写入并返回 None
        let val1: Arc<dyn Any + Send + Sync> = Arc::new(1i32);
        assert!(decorator.put_if_absent(&key, val1).is_none());

        // 第二次 put_if_absent：key 已存在，返回已有值
        let val2: Arc<dyn Any + Send + Sync> = Arc::new(2i32);
        let existing = decorator.put_if_absent(&key, val2).unwrap();
        assert_eq!(existing.get().unwrap().downcast_ref::<i32>().unwrap(), &1);
    }

    #[test]
    fn test_evict_if_present_not_deferred() {
        let cache = Arc::new(vernal_cache::SimpleCache::new("test"));
        let decorator = TransactionAwareCacheDecorator::new(cache);

        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        decorator.put(&key, value);

        assert!(decorator.evict_if_present(&key));
        assert!(!decorator.evict_if_present(&key)); // 不存在时返回 false
    }

    #[test]
    fn test_target_cache() {
        let cache = Arc::new(vernal_cache::SimpleCache::new("test"));
        let decorator = TransactionAwareCacheDecorator::new(cache.clone());

        assert_eq!(decorator.target_cache().name(), "test");
    }

    #[test]
    fn test_debug_impl() {
        let cache = Arc::new(vernal_cache::SimpleCache::new("test"));
        let decorator = TransactionAwareCacheDecorator::new(cache);
        let debug_str = format!("{:?}", decorator);
        assert!(debug_str.contains("TransactionAwareCacheDecorator"));
        assert!(debug_str.contains("test"));
    }

    #[tokio::test]
    async fn test_get_typed() {
        let cache = Arc::new(vernal_cache::SimpleCache::new("test"));
        let decorator = TransactionAwareCacheDecorator::new(cache);

        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        decorator.put(&key, value);

        let typed = CacheExt::get_typed::<i32>(&decorator, &key).unwrap();
        assert_eq!(*typed, 42);
    }

    #[tokio::test]
    async fn test_retrieve() {
        let cache = Arc::new(vernal_cache::SimpleCache::new("test"));
        let decorator = TransactionAwareCacheDecorator::new(cache);

        let key = String::from("k1");
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        decorator.put(&key, value);

        let result = decorator.retrieve(&key).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().downcast_ref::<i32>().unwrap(), &42);
    }
}
