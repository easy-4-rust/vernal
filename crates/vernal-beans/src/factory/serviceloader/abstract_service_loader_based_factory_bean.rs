//! AbstractServiceLoaderBasedFactoryBean — Spring 风格的抽象 ServiceLoader 工厂 Bean。
//!
//! 对应 Java 类：`org.springframework.beans.factory.serviceloader.AbstractServiceLoaderBasedFactoryBean`。
//!
//! 在 Spring 中，`AbstractServiceLoaderBasedFactoryBean` 是基于 Java `ServiceLoader`
//! 机制的工厂 Bean 基类。它从 `META-INF/services/` 文件中加载服务实现。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`AbstractServiceLoaderBasedFactoryBean` 使用注册表
//! 来模拟 ServiceLoader 的行为，支持动态注册服务实现。

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 抽象 ServiceLoader 工厂 Bean。
///
/// 对应 Spring 的 `AbstractServiceLoaderBasedFactoryBean`。
///
/// 基于服务注册表的工厂 Bean 基类。
#[derive(Debug, Default)]
pub struct AbstractServiceLoaderBasedFactoryBean {
    /// 服务类型名。
    service_type: String,
    /// 已注册的服务实现（service_type -> implementations）。
    registry: Mutex<HashMap<String, Vec<String>>>,
    /// 当前加载的服务索引。
    current_index: Mutex<usize>,
}

impl AbstractServiceLoaderBasedFactoryBean {
    /// 创建抽象 ServiceLoader 工厂 Bean。
    pub fn new(service_type: impl Into<String>) -> Self {
        Self {
            service_type: service_type.into(),
            registry: Mutex::new(HashMap::new()),
            current_index: Mutex::new(0),
        }
    }

    /// 获取服务类型名。
    pub fn service_type(&self) -> &str {
        &self.service_type
    }

    /// 注册服务实现。
    ///
    /// 对应 Java 的 `META-INF/services/` 文件注册。
    pub fn register_service(&self, implementation: impl Into<String>) {
        self.registry
            .lock()
            .unwrap()
            .entry(self.service_type.clone())
            .or_default()
            .push(implementation.into());
    }

    /// 获取所有已注册的服务实现。
    pub fn service_implementations(&self) -> Vec<String> {
        self.registry
            .lock()
            .unwrap()
            .get(&self.service_type)
            .cloned()
            .unwrap_or_default()
    }

    /// 获取服务实现数量。
    pub fn service_count(&self) -> usize {
        self.registry
            .lock()
            .unwrap()
            .get(&self.service_type)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// 获取下一个服务实现。
    ///
    /// 对应 Spring 的迭代式服务加载。
    pub fn next_service(&self) -> Option<String> {
        let implementations = self.service_implementations();
        let mut index = self.current_index.lock().unwrap();
        if *index < implementations.len() {
            let service = implementations[*index].clone();
            *index += 1;
            Some(service)
        } else {
            None
        }
    }

    /// 重置迭代器。
    pub fn reset(&self) {
        *self.current_index.lock().unwrap() = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_list_services() {
        let factory = AbstractServiceLoaderBasedFactoryBean::new("com.example.Logger");
        factory.register_service("com.example.ConsoleLogger");
        factory.register_service("com.example.FileLogger");
        assert_eq!(factory.service_count(), 2);
    }

    #[test]
    fn iterate_through_services() {
        let factory = AbstractServiceLoaderBasedFactoryBean::new("Service");
        factory.register_service("Impl1");
        factory.register_service("Impl2");

        assert_eq!(factory.next_service(), Some("Impl1".to_string()));
        assert_eq!(factory.next_service(), Some("Impl2".to_string()));
        assert_eq!(factory.next_service(), None);
    }

    #[test]
    fn reset_iterator() {
        let factory = AbstractServiceLoaderBasedFactoryBean::new("Service");
        factory.register_service("Impl");

        factory.next_service();
        assert_eq!(factory.next_service(), None);

        factory.reset();
        assert_eq!(factory.next_service(), Some("Impl".to_string()));
    }

    #[test]
    fn empty_factory_returns_none() {
        let factory = AbstractServiceLoaderBasedFactoryBean::new("Service");
        assert_eq!(factory.next_service(), None);
        assert_eq!(factory.service_count(), 0);
    }
}
