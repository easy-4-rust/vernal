//! moka ↔ Cache trait 适配层。
//!
//! 提供 `moka::sync::Cache` 与 `vernal_cache::Cache` 之间的适配。

use std::any::Any;
use std::sync::Arc;

use vernal_cache::{Cache, CacheError, CacheExt, CacheResult, ValueWrapper};

/// moka 缓存适配器。
///
/// 将 `moka::sync::Cache<String, Arc<dyn Any + Send + Sync>>` 包装为 `Cache` trait。
pub struct MokaCacheAdapter {
    name: String,
    inner: moka::sync::Cache<String, Arc<dyn Any + Send + Sync>>,
}

impl MokaCacheAdapter {
    /// 创建 moka 缓存适配器。
    pub fn new(name: String, inner: moka::sync::Cache<String, Arc<dyn Any + Send + Sync>>) -> Self {
        Self { name, inner }
    }

    /// 获取底层 moka 缓存。
    pub fn inner(&self) -> &moka::sync::Cache<String, Arc<dyn Any + Send + Sync>> {
        &self.inner
    }
}

impl Cache for MokaCacheAdapter {
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
        let value = self.inner.get(key_str)?;
        Some(Arc::new(MokaValueWrapper(value)))
    }

    fn put(&self, key: &dyn Any, value: Arc<dyn Any + Send + Sync>) {
        if let Some(key_str) = key.downcast_ref::<String>() {
            self.inner.insert(key_str.clone(), value);
        }
    }

    fn put_if_absent(
        &self,
        key: &dyn Any,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Option<Arc<dyn ValueWrapper>> {
        let key_str = key.downcast_ref::<String>()?.to_string();
        if let Some(existing) = self.inner.get(&key_str) {
            Some(Arc::new(MokaValueWrapper(existing)))
        } else {
            self.inner.insert(key_str, value);
            None
        }
    }

    fn evict(&self, key: &dyn Any) {
        if let Some(key_str) = key.downcast_ref::<String>() {
            self.inner.invalidate(key_str.as_str());
        }
    }

    fn evict_if_present(&self, key: &dyn Any) -> bool {
        if let Some(key_str) = key.downcast_ref::<String>() {
            self.inner.invalidate(key_str.as_str());
            return true;
        }
        false
    }

    fn clear(&self) {
        self.inner.invalidate_all();
    }

    fn invalidate(&self) -> bool {
        let had_entries = self.inner.entry_count() > 0;
        self.inner.invalidate_all();
        had_entries
    }
}

impl CacheExt for MokaCacheAdapter {
    fn get_typed<T: Any + Send + Sync>(&self, key: &dyn Any) -> Option<Arc<T>> {
        let key_str = key.downcast_ref::<String>()?;
        let value = self.inner.get(key_str)?;
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

        if let Some(value) = self.inner.get(&key_str) {
            if let Ok(typed) = value.downcast::<T>() {
                return Ok(typed);
            }
        }

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
        self.inner.insert(key_str, arc_value);
        Ok(result)
    }

    fn retrieve(
        &self,
        key: &dyn Any,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Option<Arc<dyn Any + Send + Sync>>> + Send>,
    > {
        let key_str = key.downcast_ref::<String>().cloned();
        let cache = self.inner.clone();
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
        let cache = self.inner.clone();

        if let Some(value) = cache.get(&key_str) {
            if let Ok(typed) = value.downcast::<T>() {
                return Box::pin(async { Ok(typed) });
            }
        }

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

/// moka 缓存值包装器。
struct MokaValueWrapper(Arc<dyn Any + Send + Sync>);

impl ValueWrapper for MokaValueWrapper {
    fn get(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        Some(self.0.clone())
    }
}
