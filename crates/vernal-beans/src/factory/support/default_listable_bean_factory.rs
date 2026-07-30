//! DefaultListableBeanFactory — Spring 风格的默认可列举 Bean 工厂。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.DefaultListableBeanFactory`。
//!
//! 完整功能的 Bean 工厂实现，是 Spring IoC 容器的核心。

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::factory::support::abstract_bean_factory::AbstractBeanFactory;
use crate::factory::support::autowire_candidate_resolver::AutowireCandidateResolver;
use crate::factory::support::dependency_descriptor::DependencyDescriptor;

/// 默认可列举 Bean 工厂。
///
/// 对应 Spring 的 `DefaultListableBeanFactory`。
///
/// 完整功能的 Bean 工厂实现，支持：
/// - Bean 定义注册与查找
/// - 按类型列举 Bean
/// - 自动装配候选解析
/// - 依赖注入描述符
pub struct DefaultListableBeanFactory {
    /// 继承自 AbstractBeanFactory
    parent: AbstractBeanFactory,
    /// 类型到 Bean 名称的映射（type -> [bean_name]）
    beans_of_type: Mutex<HashMap<std::any::TypeId, Vec<String>>>,
    /// 自动装配候选解析器
    autowire_candidate_resolver: Arc<dyn AutowireCandidateResolver>,
    /// 依赖描述符缓存
    dependency_descriptors: Mutex<HashMap<String, DependencyDescriptor>>,
}

impl DefaultListableBeanFactory {
    pub fn new() -> Self {
        Self {
            parent: AbstractBeanFactory::new(),
            beans_of_type: Mutex::new(HashMap::new()),
            autowire_candidate_resolver: Arc::new(SimpleAutowireCandidateResolver::new()),
            dependency_descriptors: Mutex::new(HashMap::new()),
        }
    }

    /// 注册 Bean 类型映射
    pub fn register_type_mapping(&self, type_id: std::any::TypeId, bean_name: String) {
        self.beans_of_type.lock().unwrap()
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(bean_name);
    }

    /// 按类型查找 Bean 名称列表
    pub fn get_bean_names_for_type(&self, type_id: std::any::TypeId) -> Vec<String> {
        self.beans_of_type.lock().unwrap()
            .get(&type_id)
            .cloned()
            .unwrap_or_default()
    }

    /// 获取类型映射数量
    pub fn type_mapping_count(&self) -> usize {
        self.beans_of_type.lock().unwrap().len()
    }

    /// 注册依赖描述符
    pub fn register_dependency_descriptor(&self, bean_name: String, descriptor: DependencyDescriptor) {
        self.dependency_descriptors.lock().unwrap()
            .insert(bean_name, descriptor);
    }

    /// 获取依赖描述符
    pub fn get_dependency_descriptor(&self, bean_name: &str) -> Option<DependencyDescriptor> {
        self.dependency_descriptors.lock().unwrap()
            .get(bean_name)
            .cloned()
    }

    /// 委托给 AbstractBeanFactory
    pub fn bean_definition_count(&self) -> usize {
        self.parent.bean_definition_count()
    }

    /// 委托给 AbstractBeanFactory
    pub fn singleton_count(&self) -> usize {
        self.parent.singleton_count()
    }
}

impl Default for DefaultListableBeanFactory {
    fn default() -> Self { Self::new() }
}

/// 简单的自动装配候选解析器。
pub struct SimpleAutowireCandidateResolver;

impl SimpleAutowireCandidateResolver {
    pub fn new() -> Self { Self }
}

impl AutowireCandidateResolver for SimpleAutowireCandidateResolver {
    fn is_autowire_candidate(&self, _type_id: std::any::TypeId, _bean_name: &str) -> bool {
        true
    }
}
