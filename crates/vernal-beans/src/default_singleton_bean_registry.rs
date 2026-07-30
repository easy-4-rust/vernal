//! DefaultSingletonBeanRegistry — Spring 风格的默认单例注册表。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.DefaultSingletonBeanRegistry`。
//!
//! 实现三级缓存机制，用于管理单例 Bean 的完整生命周期：
//! - 一级缓存（`singleton_objects`）：完全初始化的单例
//! - 二级缓存（`early_singleton_objects`）：早期暴露的单例（用于循环依赖）
//! - 三级缓存（`singleton_factories`）：单例工厂（用于创建早期引用）

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Spring 风格的默认单例注册表。
///
/// 实现三级缓存机制，与 Spring 的 `DefaultSingletonBeanRegistry` 对应。
///
/// ## 三级缓存说明
///
/// | 级别 | 字段 | 内容 |
/// |------|------|------|
/// | L1 | `singleton_objects` | 完全初始化的单例 Bean |
/// | L2 | `early_singleton_objects` | 早期暴露的 Bean（可能未完全注入） |
/// | L3 | `singleton_factories` | 单例工厂闭包，可创建早期引用 |
///
/// ## 循环依赖解析流程
///
/// 1. 获取 Bean 时，先查 L1 缓存
/// 2. L1 未命中且允许早期引用时，查 L2 缓存
/// 3. L2 未命中时，从 L3 工厂创建早期引用，提升到 L2
/// 4. Bean 完全初始化后，从 L2 移入 L1
pub struct DefaultSingletonBeanRegistry {
    /// 一级缓存：完全初始化的单例。
    singleton_objects: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// 二级缓存：早期暴露的单例（用于解决循环依赖）。
    early_singleton_objects: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// 三级缓存：单例工厂（用于创建早期引用）。
    singleton_factories:
        Mutex<HashMap<String, Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>>>,
    /// 正在创建中的单例名称集合（用于循环依赖检测）。
    singletons_currently_in_creation: Mutex<HashMap<String, bool>>,
    /// 已注册的单例名称（按注册顺序记录）。
    registered_singletons: Mutex<Vec<String>>,
}

impl DefaultSingletonBeanRegistry {
    /// 创建空的单例注册表。
    pub fn new() -> Self {
        Self {
            singleton_objects: Mutex::new(HashMap::new()),
            early_singleton_objects: Mutex::new(HashMap::new()),
            singleton_factories: Mutex::new(HashMap::new()),
            singletons_currently_in_creation: Mutex::new(HashMap::new()),
            registered_singletons: Mutex::new(Vec::new()),
        }
    }

    /// 获取单例（支持三级缓存）。
    ///
    /// 对应 Spring 的 `DefaultSingletonBeanRegistry.getSingleton(String, boolean)`。
    ///
    /// # 参数
    ///
    /// - `name` — Bean 名称
    /// - `allow_early_ref` — 是否允许从二级/三级缓存获取早期引用
    ///
    /// # 返回
    ///
    /// 如果找到单例（包括早期引用），返回 `Some(Arc<...>)`；否则返回 `None`。
    pub fn get_singleton(
        &self,
        name: &str,
        allow_early_ref: bool,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        // 1. 查一级缓存
        {
            let objects = self.singleton_objects.lock().unwrap();
            if let Some(obj) = objects.get(name) {
                return Some(Arc::clone(obj));
            }
        }

        if !allow_early_ref {
            return None;
        }

        // 2. 检查是否正在创建中（未在创建中的 Bean 不应该从早期缓存获取）
        {
            let creating = self.singletons_currently_in_creation.lock().unwrap();
            if !creating.get(name).copied().unwrap_or(false) {
                return None;
            }
        }

        // 3. 查二级缓存
        {
            let early = self.early_singleton_objects.lock().unwrap();
            if let Some(obj) = early.get(name) {
                return Some(Arc::clone(obj));
            }
        }

        // 4. 从三级缓存工厂创建早期引用，提升到二级缓存
        {
            let mut factories = self.singleton_factories.lock().unwrap();
            if let Some(factory) = factories.remove(name) {
                let early_ref = factory();
                let mut early = self.early_singleton_objects.lock().unwrap();
                early.insert(name.to_string(), Arc::clone(&early_ref));
                return Some(early_ref);
            }
        }

        None
    }

    /// 注册单例到一级缓存。
    ///
    /// 对应 Spring 的 `DefaultSingletonBeanRegistry.registerSingleton(String, Object)`。
    ///
    /// 同时清除该名称在二级和三级缓存中的条目。
    pub fn register_singleton(
        &self,
        name: &str,
        singleton_object: Arc<dyn Any + Send + Sync>,
    ) {
        {
            let mut objects = self.singleton_objects.lock().unwrap();
            objects.insert(name.to_string(), singleton_object);
        }
        // 清除二级和三级缓存
        {
            let mut factories = self.singleton_factories.lock().unwrap();
            factories.remove(name);
        }
        {
            let mut early = self.early_singleton_objects.lock().unwrap();
            early.remove(name);
        }
        {
            let mut registered = self.registered_singletons.lock().unwrap();
            if !registered.iter().any(|n| n == name) {
                registered.push(name.to_string());
            }
        }
    }

    /// 添加单例工厂到三级缓存。
    ///
    /// 用于在 Bean 创建早期阶段注册工厂，以便在循环依赖发生时
    /// 可以创建早期引用（可能包含 AOP 代理）。
    ///
    /// 对应 Spring 的 `DefaultSingletonBeanRegistry.addSingletonFactory(String, ObjectFactory)`。
    pub fn add_singleton_factory(
        &self,
        name: &str,
        factory: Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>,
    ) {
        // 只有一级缓存中没有时才添加
        {
            let objects = self.singleton_objects.lock().unwrap();
            if objects.contains_key(name) {
                return;
            }
        }
        let mut factories = self.singleton_factories.lock().unwrap();
        factories.insert(name.to_string(), factory);
    }

    /// 获取早期单例引用。
    ///
    /// 从三级缓存的工厂创建早期引用。如果三级缓存中没有工厂，
    /// 则尝试从二级缓存获取。
    pub fn get_early_bean_reference(
        &self,
        name: &str,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        // 先检查二级缓存
        {
            let early = self.early_singleton_objects.lock().unwrap();
            if let Some(obj) = early.get(name) {
                return Some(Arc::clone(obj));
            }
        }
        // 从三级缓存工厂创建
        {
            let factories = self.singleton_factories.lock().unwrap();
            if let Some(factory) = factories.get(name) {
                let early_ref = factory();
                let mut early = self.early_singleton_objects.lock().unwrap();
                early.insert(name.to_string(), Arc::clone(&early_ref));
                return Some(early_ref);
            }
        }
        None
    }

    /// 检查是否包含指定单例（一级缓存）。
    ///
    /// 对应 Spring 的 `DefaultSingletonBeanRegistry.containsSingleton(String)`。
    pub fn contains_singleton(&self, name: &str) -> bool {
        self.singleton_objects
            .lock()
            .unwrap()
            .contains_key(name)
    }

    /// 获取所有已注册的单例名称。
    ///
    /// 对应 Spring 的 `DefaultSingletonBeanRegistry.getSingletonNames()`。
    pub fn singleton_names(&self) -> Vec<String> {
        self.registered_singletons.lock().unwrap().clone()
    }

    /// 获取一级缓存中的单例数量。
    ///
    /// 对应 Spring 的 `DefaultSingletonBeanRegistry.getSingletonCount()`。
    pub fn singleton_count(&self) -> usize {
        self.singleton_objects.lock().unwrap().len()
    }

    /// 标记单例正在创建中。
    ///
    /// 用于循环依赖检测。Bean 开始实例化时调用。
    pub fn mark_singleton_as_in_creation(&self, name: &str) {
        let mut creating = self.singletons_currently_in_creation.lock().unwrap();
        creating.insert(name.to_string(), true);
    }

    /// 检查单例是否正在创建中。
    ///
    /// 用于循环依赖检测。
    pub fn is_singleton_currently_in_creation(&self, name: &str) -> bool {
        self.singletons_currently_in_creation
            .lock()
            .unwrap()
            .get(name)
            .copied()
            .unwrap_or(false)
    }

    /// 标记单例创建完成。
    ///
    /// Bean 完全初始化后调用，清除"正在创建"标记。
    pub fn mark_singleton_as_created(&self, name: &str) {
        let mut creating = self.singletons_currently_in_creation.lock().unwrap();
        creating.remove(name);
    }

    /// 销毁所有单例。
    ///
    /// 清空所有三级缓存。实际的 Bean 销毁回调
    /// 由上层（如 `DefaultListableBeanFactory`）负责调用。
    pub fn destroy_singletons(&self) {
        let names: Vec<String> = {
            let registered = self.registered_singletons.lock().unwrap();
            registered.clone()
        };

        // 按逆序销毁（LIFO，与 Spring 保持一致）
        for name in names.iter().rev() {
            self.destroy_singleton(name);
        }

        // 清空所有缓存
        self.singleton_objects.lock().unwrap().clear();
        self.early_singleton_objects.lock().unwrap().clear();
        self.singleton_factories.lock().unwrap().clear();
        self.singletons_currently_in_creation
            .lock()
            .unwrap()
            .clear();
        self.registered_singletons.lock().unwrap().clear();
    }

    /// 销毁指定单例。
    ///
    /// 从所有缓存级别移除该 Bean。
    pub fn destroy_singleton(&self, name: &str) {
        self.singleton_objects.lock().unwrap().remove(name);
        self.early_singleton_objects.lock().unwrap().remove(name);
        self.singleton_factories.lock().unwrap().remove(name);
        self.singletons_currently_in_creation
            .lock()
            .unwrap()
            .remove(name);
    }

    /// 清除所有缓存但保留注册记录。
    ///
    /// 用于测试或重置场景。
    pub fn clear(&self) {
        self.singleton_objects.lock().unwrap().clear();
        self.early_singleton_objects.lock().unwrap().clear();
        self.singleton_factories.lock().unwrap().clear();
        self.singletons_currently_in_creation
            .lock()
            .unwrap()
            .clear();
    }
}

impl Default for DefaultSingletonBeanRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn register_and_get_singleton() {
        let registry = DefaultSingletonBeanRegistry::new();
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42_i32);
        registry.register_singleton("answer", Arc::clone(&value));

        assert!(registry.contains_singleton("answer"));
        assert_eq!(registry.singleton_count(), 1);

        let got = registry.get_singleton("answer", false).unwrap();
        assert!(got.downcast_ref::<i32>().is_some());
        assert_eq!(*got.downcast_ref::<i32>().unwrap(), 42);
    }

    #[test]
    fn three_level_cache_early_reference() {
        let registry = DefaultSingletonBeanRegistry::new();

        // 标记正在创建
        registry.mark_singleton_as_in_creation("my_bean");

        // 添加三级缓存工厂
        let factory: Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync> =
            Arc::new(|| Arc::new("early_ref".to_string()) as Arc<dyn Any + Send + Sync>);
        registry.add_singleton_factory("my_bean", factory);

        // 获取早期引用（应该从三级缓存提升到二级）
        let early = registry.get_singleton("my_bean", true);
        assert!(early.is_some());
        let val = early.unwrap();
        assert_eq!(
            *val.downcast_ref::<String>().unwrap(),
            "early_ref"
        );

        // 二级缓存应该有值
        let early2 = registry.get_early_bean_reference("my_bean");
        assert!(early2.is_some());

        // 完全初始化后注册到一级缓存
        let final_bean: Arc<dyn Any + Send + Sync> = Arc::new("final".to_string());
        registry.register_singleton("my_bean", final_bean);
        registry.mark_singleton_as_created("my_bean");

        assert!(registry.contains_singleton("my_bean"));
    }

    #[test]
    fn destroy_singletons_clears_all() {
        let registry = DefaultSingletonBeanRegistry::new();
        registry.register_singleton("a", Arc::new(1_i32));
        registry.register_singleton("b", Arc::new(2_i32));

        assert_eq!(registry.singleton_count(), 2);

        registry.destroy_singletons();

        assert_eq!(registry.singleton_count(), 0);
        assert!(registry.singleton_names().is_empty());
    }

    #[test]
    fn creation_tracking() {
        let registry = DefaultSingletonBeanRegistry::new();
        assert!(!registry.is_singleton_currently_in_creation("bean"));

        registry.mark_singleton_as_in_creation("bean");
        assert!(registry.is_singleton_currently_in_creation("bean"));

        registry.mark_singleton_as_created("bean");
        assert!(!registry.is_singleton_currently_in_creation("bean"));
    }
}
