//! DefaultListableBeanFactory — Spring 风格的默认可列举 Bean 工厂。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.DefaultListableBeanFactory`。
//!
//! 在 Spring 中，这是最完整的 Bean 工厂实现，是 IoC 容器的核心。
//! 它集成了 `AbstractBeanFactory`、`AbstractAutowireCapableBeanFactory`
//! 和 `ListableBeanFactory` 的所有功能。
//!
//! ## 主要功能
//!
//! - Bean 定义注册与查找
//! - 按类型列举 Bean
//! - 自动装配候选解析
//! - 依赖注入描述符管理

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::factory::support::abstract_bean_factory::AbstractBeanFactory;
use crate::factory::support::autowire_candidate_resolver::AutowireCandidateResolver;
use crate::factory::support::dependency_descriptor::DependencyDescriptor;

/// 默认可列举 Bean 工厂。
///
/// 对应 Spring 的 `DefaultListableBeanFactory`。
///
/// 完整功能的 Bean 工厂实现，支持 Bean 定义注册、
/// 按类型查找、自动装配等。
#[allow(dead_code)]
pub struct DefaultListableBeanFactory {
    /// 继承自 AbstractBeanFactory
    parent: AbstractBeanFactory,
    /// 类型到 Bean 名称的映射（type -> [bean_name]）
    beans_of_type: Mutex<HashMap<std::any::TypeId, Vec<String>>>,
    /// 自动装配候选解析器
    autowire_candidate_resolver: Arc<dyn AutowireCandidateResolver>,
    /// 依赖描述符缓存
    dependency_descriptors: Mutex<HashMap<String, DependencyDescriptor>>,
    /// 是否允许 Bean 定义覆盖
    allow_bean_definition_overriding: Mutex<bool>,
}

impl DefaultListableBeanFactory {
    /// 创建新的默认可列举 Bean 工厂。
    pub fn new() -> Self {
        Self {
            parent: AbstractBeanFactory::new(),
            beans_of_type: Mutex::new(HashMap::new()),
            autowire_candidate_resolver: Arc::new(SimpleAutowireCandidateResolver::new()),
            dependency_descriptors: Mutex::new(HashMap::new()),
            allow_bean_definition_overriding: Mutex::new(true),
        }
    }

    /// 注册 Bean 类型映射。
    ///
    /// 对应 Spring 的类型索引维护。
    pub fn register_type_mapping(&self, type_id: std::any::TypeId, bean_name: String) {
        self.beans_of_type.lock().unwrap()
            .entry(type_id)
            .or_default()
            .push(bean_name);
    }

    /// 按类型查找 Bean 名称列表。
    pub fn get_bean_names_for_type(&self, type_id: std::any::TypeId) -> Vec<String> {
        self.beans_of_type.lock().unwrap()
            .get(&type_id)
            .cloned()
            .unwrap_or_default()
    }

    /// 获取类型映射数量。
    pub fn type_mapping_count(&self) -> usize {
        self.beans_of_type.lock().unwrap().len()
    }

    /// 注册依赖描述符。
    pub fn register_dependency_descriptor(&self, bean_name: String, descriptor: DependencyDescriptor) {
        self.dependency_descriptors.lock().unwrap()
            .insert(bean_name, descriptor);
    }

    /// 获取依赖描述符。
    pub fn get_dependency_descriptor(&self, bean_name: &str) -> Option<DependencyDescriptor> {
        self.dependency_descriptors.lock().unwrap()
            .get(bean_name)
            .cloned()
    }

    /// 设置是否允许 Bean 定义覆盖。
    pub fn set_allow_bean_definition_overriding(&self, allow: bool) {
        *self.allow_bean_definition_overriding.lock().unwrap() = allow;
    }

    /// 是否允许 Bean 定义覆盖。
    pub fn is_allow_bean_definition_overriding(&self) -> bool {
        *self.allow_bean_definition_overriding.lock().unwrap()
    }

    /// 委托给 AbstractBeanFactory
    pub fn bean_definition_count(&self) -> usize {
        self.parent.bean_definition_count()
    }

    /// 委托给 AbstractBeanFactory
    pub fn singleton_count(&self) -> usize {
        self.parent.singleton_count()
    }

    /// 获取父工厂的引用。
    pub fn parent_factory(&self) -> &AbstractBeanFactory {
        &self.parent
    }
}

impl Default for DefaultListableBeanFactory {
    fn default() -> Self { Self::new() }
}

/// 简单的自动装配候选解析器。
pub struct SimpleAutowireCandidateResolver;

impl SimpleAutowireCandidateResolver {
    /// 创建一个新的实例。
    pub fn new() -> Self { Self }
}

impl AutowireCandidateResolver for SimpleAutowireCandidateResolver {
    fn is_autowire_candidate(&self, _type_id: std::any::TypeId, _bean_name: &str) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    #[test]
    fn new_factory_is_empty() {
        let factory = DefaultListableBeanFactory::new();
        assert_eq!(factory.bean_definition_count(), 0);
        assert_eq!(factory.singleton_count(), 0);
        assert_eq!(factory.type_mapping_count(), 0);
    }

    #[test]
    fn register_type_mapping() {
        let factory = DefaultListableBeanFactory::new();
        factory.register_type_mapping(TypeId::of::<String>(), "myString".to_string());
        factory.register_type_mapping(TypeId::of::<String>(), "anotherString".to_string());

        let names = factory.get_bean_names_for_type(TypeId::of::<String>());
        assert_eq!(names.len(), 2);
        assert_eq!(factory.type_mapping_count(), 1);
    }

    #[test]
    fn get_bean_names_for_unknown_type() {
        let factory = DefaultListableBeanFactory::new();
        let names = factory.get_bean_names_for_type(TypeId::of::<i32>());
        assert!(names.is_empty());
    }

    #[test]
    fn dependency_descriptor_management() {
        let factory = DefaultListableBeanFactory::new();
        let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true);

        factory.register_dependency_descriptor("myBean".to_string(), desc);
        let found = factory.get_dependency_descriptor("myBean").unwrap();
        assert!(found.is_required());
    }

    #[test]
    fn bean_definition_overriding_default() {
        let factory = DefaultListableBeanFactory::new();
        assert!(factory.is_allow_bean_definition_overriding());
    }

    #[test]
    fn disable_bean_definition_overriding() {
        let factory = DefaultListableBeanFactory::new();
        factory.set_allow_bean_definition_overriding(false);
        assert!(!factory.is_allow_bean_definition_overriding());
    }

    #[test]
    fn simple_resolver_always_allows() {
        let resolver = SimpleAutowireCandidateResolver::new();
        assert!(resolver.is_autowire_candidate(TypeId::of::<String>(), "any"));
    }

    #[test]
    fn parent_factory_returns_reference() {
        let factory = DefaultListableBeanFactory::new();
        let parent = factory.parent_factory();
        assert_eq!(parent.bean_definition_count(), 0);
    }

    #[test]
    fn parent_factory_delegates_operations() {
        let factory = DefaultListableBeanFactory::new();
        let parent = factory.parent_factory();
        parent.register_bean_definition("bean".to_string(), Arc::new(42i32));
        assert_eq!(parent.bean_definition_count(), 1);
        // Verify it's the same underlying factory
        assert_eq!(factory.bean_definition_count(), 1);
    }

    #[test]
    fn get_dependency_descriptor_missing() {
        let factory = DefaultListableBeanFactory::new();
        assert!(factory.get_dependency_descriptor("nonexistent").is_none());
    }

    #[test]
    fn register_multiple_dependency_descriptors() {
        let factory = DefaultListableBeanFactory::new();
        let desc1 = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true);
        let desc2 = DependencyDescriptor::new(TypeId::of::<i32>(), "i32".to_string(), false);

        factory.register_dependency_descriptor("bean1".to_string(), desc1);
        factory.register_dependency_descriptor("bean2".to_string(), desc2);

        let found1 = factory.get_dependency_descriptor("bean1").unwrap();
        assert!(found1.is_required());
        let found2 = factory.get_dependency_descriptor("bean2").unwrap();
        assert!(!found2.is_required());
    }

    #[test]
    fn register_multiple_type_mappings_different_types() {
        let factory = DefaultListableBeanFactory::new();
        factory.register_type_mapping(TypeId::of::<String>(), "s1".to_string());
        factory.register_type_mapping(TypeId::of::<i32>(), "i1".to_string());
        factory.register_type_mapping(TypeId::of::<String>(), "s2".to_string());

        assert_eq!(factory.type_mapping_count(), 2);
        assert_eq!(factory.get_bean_names_for_type(TypeId::of::<String>()).len(), 2);
        assert_eq!(factory.get_bean_names_for_type(TypeId::of::<i32>()).len(), 1);
        assert!(factory.get_bean_names_for_type(TypeId::of::<f64>()).is_empty());
    }

    #[test]
    fn toggle_bean_definition_overriding() {
        let factory = DefaultListableBeanFactory::new();
        assert!(factory.is_allow_bean_definition_overriding());

        factory.set_allow_bean_definition_overriding(false);
        assert!(!factory.is_allow_bean_definition_overriding());

        factory.set_allow_bean_definition_overriding(true);
        assert!(factory.is_allow_bean_definition_overriding());
    }
}
