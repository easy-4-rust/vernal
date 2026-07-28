//! ApplicationScope — Spring 风格的应用作用域。
//!
//! 对应 Java 类：`org.springframework.web.context.ServletContextScope`。
//!
//! 在整个应用生命周期内缓存 Bean 实例（类似 Singleton 但通过 Scope 机制管理）。

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::bean_scope::BeanScope;

/// Spring 风格的应用作用域。
///
/// 对应 Spring 的 `ServletContextScope`。
///
/// 在整个应用生命周期内缓存 Bean 实例：
/// - 与 Singleton 类似，但通过 Scope 机制管理
/// - 应用关闭时调用所有 destruction callback
/// - 支持 `remove` 主动移除 Bean
pub struct ApplicationScope {
    /// 已缓存的 Bean 实例。
    instances: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// 销毁回调。
    destruction_callbacks: Mutex<Vec<Box<dyn FnOnce() + Send + Sync>>>,
}

impl ApplicationScope {
    /// 创建新的 ApplicationScope。
    pub fn new() -> Self {
        Self {
            instances: Mutex::new(HashMap::new()),
            destruction_callbacks: Mutex::new(Vec::new()),
        }
    }

    /// 获取已缓存的 Bean 数量。
    pub fn cached_count(&self) -> usize {
        self.instances
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// 执行所有销毁回调。
    pub fn destroy(&self) {
        let callbacks: Vec<_> = {
            let mut cbs = self
                .destruction_callbacks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *cbs)
        };
        for callback in callbacks {
            callback();
        }
        self.instances
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }
}

impl Default for ApplicationScope {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanScope for ApplicationScope {
    fn get(
        &self,
        name: &str,
        object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let mut instances = self
            .instances
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if let Some(existing) = instances.get(name) {
            return Ok(Box::new(Arc::clone(existing)));
        }

        let instance = object_factory();
        let arc_instance: Arc<dyn Any + Send + Sync> = Arc::from(instance);
        instances.insert(name.to_string(), Arc::clone(&arc_instance));
        Ok(Box::new(arc_instance))
    }

    fn remove(
        &self,
        name: &str,
    ) -> Result<Option<Box<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        let removed = self
            .instances
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(name);
        Ok(removed.map(|arc| Box::new(arc) as Box<dyn Any + Send + Sync>))
    }

    fn register_destruction_callback(
        &self,
        _name: &str,
        callback: Box<dyn FnOnce() + Send + Sync>,
    ) {
        self.destruction_callbacks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(callback);
    }

    fn resolve_contextual_object(&self, key: &str) -> Option<Box<dyn Any>> {
        if key == "application" {
            Some(Box::new("application".to_string()))
        } else {
            None
        }
    }

    fn conversation_id(&self) -> Option<&str> {
        Some("application")
    }
}
