//! DefaultSingletonBeanRegistry — Spring 风格默认单例注册表。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.DefaultSingletonBeanRegistry`。
//!
//! 在 Spring 中，这是单例 Bean 管理的核心实现，提供三级缓存机制：
//! 1. **一级缓存** (`singletonObjects`) — 完整的单例对象
//! 2. **二级缓存** (`earlySingletonObjects`) — 提前暴露的半成品对象
//! 3. **三级缓存** (`singletonFactories`) — 单例工厂
//!
//! 这三级缓存是 Spring 解决循环依赖的关键机制。

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 默认单例注册表。
///
/// 对应 Spring 的 `DefaultSingletonBeanRegistry`。
///
/// 实现三级 Singleton 缓存，支持循环依赖检测和解决。
pub struct DefaultSingletonBeanRegistry {
    /// 一级缓存：完整的单例对象
    singleton_objects: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// 二级缓存：提前暴露的半成品对象
    early_singleton_objects: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// 三级缓存：单例工厂
    singleton_factories: Mutex<HashMap<String, Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>>>,
    /// 正在创建中的 Bean（循环依赖检测）
    singletons_currently_in_creation: Mutex<HashMap<String, bool>>,
    /// 已注册的单例名称（按注册顺序）
    registered_singletons: Mutex<Vec<String>>,
}

impl DefaultSingletonBeanRegistry {
    /// 创建新的单例注册表。
    pub fn new() -> Self {
        Self {
            singleton_objects: Mutex::new(HashMap::new()),
            early_singleton_objects: Mutex::new(HashMap::new()),
            singleton_factories: Mutex::new(HashMap::new()),
            singletons_currently_in_creation: Mutex::new(HashMap::new()),
            registered_singletons: Mutex::new(Vec::new()),
        }
    }

    /// 获取单例对象（一级缓存）。
    ///
    /// 对应 Spring 的 `getSingleton(String)`。
    pub fn get_singleton(&self, name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.singleton_objects.lock().unwrap().get(name).map(Arc::clone)
    }

    /// 检查是否包含指定单例。
    pub fn contains_singleton(&self, name: &str) -> bool {
        self.singleton_objects.lock().unwrap().contains_key(name)
    }

    /// 获取单例总数。
    pub fn singleton_count(&self) -> usize {
        self.singleton_objects.lock().unwrap().len()
    }

    /// 获取所有单例名称。
    pub fn singleton_names(&self) -> Vec<String> {
        self.singleton_objects.lock().unwrap().keys().cloned().collect()
    }

    /// 注册单例对象。
    ///
    /// 对应 Spring 的 `registerSingleton(String, Object)`。
    pub fn register_singleton(&self, name: String, obj: Arc<dyn Any + Send + Sync>) {
        self.singleton_objects.lock().unwrap().insert(name.clone(), obj);
        self.registered_singletons.lock().unwrap().push(name);
    }

    /// 添加单例工厂（三级缓存）。
    ///
    /// 对应 Spring 的 `addSingletonFactory(String, ObjectFactory)`。
    pub fn add_singleton_factory(&self, name: String, factory: Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>) {
        self.singleton_factories.lock().unwrap().insert(name, factory);
    }

    /// 获取早期 Bean 引用（从三级缓存提升到二级缓存）。
    ///
    /// 对应 Spring 的 `getEarlyBeanReference`。
    ///
    /// 这是循环依赖解决的关键方法：
    /// 1. 从三级缓存获取工厂
    /// 2. 调用工厂创建早期引用
    /// 3. 将结果放入二级缓存
    /// 4. 移除三级缓存
    pub fn get_early_bean_reference(&self, name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        let mut factories = self.singleton_factories.lock().unwrap();
        if let Some(f) = factories.remove(name) {
            let bean = f();
            self.early_singleton_objects.lock().unwrap().insert(name.to_string(), Arc::clone(&bean));
            return Some(bean);
        }
        None
    }

    /// 标记 Bean 正在创建中。
    ///
    /// 对应 Spring 的 `markSingletonAsCurrentlyInCreation`。
    pub fn mark_as_in_creation(&self, name: &str) {
        self.singletons_currently_in_creation.lock().unwrap().insert(name.to_string(), true);
    }

    /// 检查 Bean 是否正在创建中。
    pub fn is_currently_in_creation(&self, name: &str) -> bool {
        self.singletons_currently_in_creation.lock().unwrap().get(name).copied().unwrap_or(false)
    }

    /// 获取正在创建中的 Bean 数量。
    pub fn in_creation_count(&self) -> usize {
        self.singletons_currently_in_creation.lock().unwrap().len()
    }

    /// 销毁所有单例。
    ///
    /// 对应 Spring 的 `destroySingletons()`。
    pub fn destroy_singletons(&self) {
        self.singleton_objects.lock().unwrap().clear();
        self.early_singleton_objects.lock().unwrap().clear();
        self.singleton_factories.lock().unwrap().clear();
        self.singletons_currently_in_creation.lock().unwrap().clear();
    }

    /// 销毁指定单例。
    pub fn destroy_singleton(&self, name: &str) {
        self.singleton_objects.lock().unwrap().remove(name);
        self.early_singleton_objects.lock().unwrap().remove(name);
        self.singleton_factories.lock().unwrap().remove(name);
    }

    /// 获取已注册的单例名称列表（按注册顺序）。
    pub fn registered_singleton_names(&self) -> Vec<String> {
        self.registered_singletons.lock().unwrap().clone()
    }
}

impl Default for DefaultSingletonBeanRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_registry_is_empty() {
        let registry = DefaultSingletonBeanRegistry::new();
        assert_eq!(registry.singleton_count(), 0);
        assert!(registry.singleton_names().is_empty());
    }

    #[test]
    fn register_and_get_singleton() {
        let registry = DefaultSingletonBeanRegistry::new();
        registry.register_singleton("myBean".to_string(), Arc::new(42_i32));

        assert!(registry.contains_singleton("myBean"));
        assert_eq!(registry.singleton_count(), 1);

        let bean = registry.get_singleton("myBean").unwrap();
        assert_eq!(bean.downcast_ref::<i32>(), Some(&42));
    }

    #[test]
    fn get_nonexistent_singleton_returns_none() {
        let registry = DefaultSingletonBeanRegistry::new();
        assert!(registry.get_singleton("missing").is_none());
    }

    #[test]
    fn singleton_factory_and_early_reference() {
        let registry = DefaultSingletonBeanRegistry::new();
        registry.add_singleton_factory(
            "myBean".to_string(),
            Arc::new(|| Arc::new("early_bean".to_string()) as Arc<dyn Any + Send + Sync>),
        );

        let early = registry.get_early_bean_reference("myBean");
        assert!(early.is_some());
        assert_eq!(early.unwrap().downcast_ref::<String>(), Some(&"early_bean".to_string()));
    }

    #[test]
    fn early_reference_removes_factory() {
        let registry = DefaultSingletonBeanRegistry::new();
        registry.add_singleton_factory("test".to_string(), Arc::new(|| Arc::new(1)));

        registry.get_early_bean_reference("test");
        assert!(registry.get_early_bean_reference("test").is_none());
    }

    #[test]
    fn mark_and_check_creation() {
        let registry = DefaultSingletonBeanRegistry::new();
        assert!(!registry.is_currently_in_creation("bean"));

        registry.mark_as_in_creation("bean");
        assert!(registry.is_currently_in_creation("bean"));
        assert_eq!(registry.in_creation_count(), 1);
    }

    #[test]
    fn destroy_singletons_clears_all() {
        let registry = DefaultSingletonBeanRegistry::new();
        registry.register_singleton("a".to_string(), Arc::new(1));
        registry.register_singleton("b".to_string(), Arc::new(2));
        registry.mark_as_in_creation("c");

        registry.destroy_singletons();
        assert_eq!(registry.singleton_count(), 0);
        assert_eq!(registry.in_creation_count(), 0);
    }

    #[test]
    fn destroy_specific_singleton() {
        let registry = DefaultSingletonBeanRegistry::new();
        registry.register_singleton("target".to_string(), Arc::new(1));
        registry.register_singleton("other".to_string(), Arc::new(2));

        registry.destroy_singleton("target");
        assert!(!registry.contains_singleton("target"));
        assert!(registry.contains_singleton("other"));
    }

    #[test]
    fn registered_singleton_names_preserves_order() {
        let registry = DefaultSingletonBeanRegistry::new();
        registry.register_singleton("first".to_string(), Arc::new(1));
        registry.register_singleton("second".to_string(), Arc::new(2));
        registry.register_singleton("third".to_string(), Arc::new(3));

        let names = registry.registered_singleton_names();
        assert_eq!(names, vec!["first", "second", "third"]);
    }
}
