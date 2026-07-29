//! FactoryBeanRegistrySupport — Spring 风格的 FactoryBean 缓存支持。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.FactoryBeanRegistrySupport`。
//!
//! 提供 FactoryBean 创建的单例对象缓存管理。`AbstractBeanFactory` 可以组合
//! 此结构体来获得 FactoryBean 产品的缓存能力。

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Spring 风格的 FactoryBean 单例对象缓存支持。
///
/// 对应 Spring 的 `FactoryBeanRegistrySupport`。
///
/// 管理 FactoryBean 创建的 singleton 对象的缓存。当 FactoryBean 的 `is_singleton()`
/// 返回 `true` 时，容器将 FactoryBean 的产品缓存起来，后续请求直接从缓存返回。
///
/// ## 线程安全
///
/// 内部使用 `Arc<Mutex<HashMap>>` 保证线程安全。
#[derive(Debug)]
pub struct FactoryBeanRegistrySupport {
    /// FactoryBean 对象缓存 key: FactoryBean 名称, value: 缓存的产品实例
    factory_bean_object_cache: Arc<Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
}

impl FactoryBeanRegistrySupport {
    /// 创建新的 FactoryBeanRegistrySupport。
    pub fn new() -> Self {
        Self {
            factory_bean_object_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 从缓存中获取 FactoryBean 的产品对象。
    ///
    /// 对应 Spring 的 `getCachedObjectForFactoryBean(String beanName)`。
    ///
    /// # 参数
    ///
    /// * `bean_name` — FactoryBean 的名称
    ///
    /// # 返回
    ///
    /// - `Some(Arc)` — 缓存的产品实例
    /// - `None` — 尚未缓存
    pub fn get_cached_object(&self, bean_name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.factory_bean_object_cache
            .lock()
            .ok()
            .and_then(|cache| cache.get(bean_name).cloned())
    }

    /// 将 FactoryBean 的产品对象放入缓存。
    ///
    /// 对应 Spring 的 `cacheObjectForFactoryBean(String beanName, Object beanObject)`。
    ///
    /// # 参数
    ///
    /// * `bean_name` — FactoryBean 的名称
    /// * `bean_object` — 要缓存的产品实例
    pub fn cache_object(
        &self,
        bean_name: impl Into<String>,
        bean_object: Arc<dyn Any + Send + Sync>,
    ) {
        if let Ok(mut cache) = self.factory_bean_object_cache.lock() {
            cache.insert(bean_name.into(), bean_object);
        }
    }

    /// 从缓存中移除 FactoryBean 的产品对象。
    ///
    /// 对应 Spring 的 `removeCachedObjectForFactoryBean(String beanName)`。
    ///
    /// # 参数
    ///
    /// * `bean_name` — FactoryBean 的名称
    pub fn remove_cached_object(&self, bean_name: &str) {
        if let Ok(mut cache) = self.factory_bean_object_cache.lock() {
            cache.remove(bean_name);
        }
    }

    /// 清空所有缓存。
    ///
    /// 对应 Spring 的 `clearCachedObjectForFactoryBean` 的批量操作。
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.factory_bean_object_cache.lock() {
            cache.clear();
        }
    }

    /// 检查是否包含指定 FactoryBean 的缓存。
    ///
    /// # 参数
    ///
    /// * `bean_name` — FactoryBean 的名称
    pub fn contains_cached_object(&self, bean_name: &str) -> bool {
        self.factory_bean_object_cache
            .lock()
            .ok()
            .map(|cache| cache.contains_key(bean_name))
            .unwrap_or(false)
    }
}

impl Default for FactoryBeanRegistrySupport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_and_get() {
        let support = FactoryBeanRegistrySupport::new();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        support.cache_object("myFactory", obj.clone());

        let cached = support.get_cached_object("myFactory");
        assert!(cached.is_some());
        assert!(support.contains_cached_object("myFactory"));
    }

    #[test]
    fn test_get_nonexistent() {
        let support = FactoryBeanRegistrySupport::new();
        assert!(support.get_cached_object("nonexistent").is_none());
    }

    #[test]
    fn test_remove() {
        let support = FactoryBeanRegistrySupport::new();
        let obj: Arc<dyn Any + Send + Sync> = Arc::new("test");
        support.cache_object("bean", obj);
        assert!(support.contains_cached_object("bean"));

        support.remove_cached_object("bean");
        assert!(!support.contains_cached_object("bean"));
    }

    #[test]
    fn test_clear() {
        let support = FactoryBeanRegistrySupport::new();
        support.cache_object("a", Arc::new(1i32) as Arc<dyn Any + Send + Sync>);
        support.cache_object("b", Arc::new(2i32) as Arc<dyn Any + Send + Sync>);
        support.clear_cache();
        assert!(!support.contains_cached_object("a"));
        assert!(!support.contains_cached_object("b"));
    }
}
