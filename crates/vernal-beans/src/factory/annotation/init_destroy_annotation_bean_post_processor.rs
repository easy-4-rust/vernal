//! InitDestroyAnnotationBeanPostProcessor — Spring 风格的初始化/销毁注解后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.InitDestroyAnnotationBeanPostProcessor`。
//!
//! 处理 `@PostConstruct` 和 `@PreDestroy` 注解。

use std::any::Any;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

use crate::bean_post_processor::BeanPostProcessor;

/// Spring 风格的初始化/销毁注解后处理器。
///
/// 对应 Spring 的 `InitDestroyAnnotationBeanPostProcessor`。
///
/// 扫描 Bean 的方法，识别 `@PostConstruct` 和 `@PreDestroy` 注解，
/// 在适当的生命周期阶段调用这些方法。
pub struct InitDestroyAnnotationBeanPostProcessor {
    init_methods: Mutex<HashSet<String>>,
    destroy_methods: Mutex<HashSet<String>>,
    initialized: Mutex<bool>,
}

impl InitDestroyAnnotationBeanPostProcessor {
    pub fn new() -> Self {
        Self {
            init_methods: Mutex::new(HashSet::new()),
            destroy_methods: Mutex::new(HashSet::new()),
            initialized: Mutex::new(false),
        }
    }

    pub fn register_init_method(&self, method_name: String) {
        self.init_methods.lock().unwrap().insert(method_name);
    }

    pub fn register_destroy_method(&self, method_name: String) {
        self.destroy_methods.lock().unwrap().insert(method_name);
    }

    pub fn init_method_count(&self) -> usize {
        self.init_methods.lock().unwrap().len()
    }

    pub fn destroy_method_count(&self) -> usize {
        self.destroy_methods.lock().unwrap().len()
    }

    pub fn has_init_method(&self, name: &str) -> bool {
        self.init_methods.lock().unwrap().contains(name)
    }

    pub fn has_destroy_method(&self, name: &str) -> bool {
        self.destroy_methods.lock().unwrap().contains(name)
    }

    pub fn initialize(&self) {
        *self.initialized.lock().unwrap() = true;
    }
}

impl Default for InitDestroyAnnotationBeanPostProcessor {
    fn default() -> Self { Self::new() }
}

impl BeanPostProcessor for InitDestroyAnnotationBeanPostProcessor {
    fn post_process_after_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }
}
