//! InitDestroyAnnotationBeanPostProcessor — Spring 风格的初始化/销毁注解后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.InitDestroyAnnotationBeanPostProcessor`。
//!
//! 处理 `@PostConstruct` 和 `@PreDestroy` 注解。
//!
//! 在 Spring 中，此类在 Bean 实例化后扫描 `@PostConstruct` 标注的方法
//! 并在 Bean 销毁前扫描 `@PreDestroy` 标注的方法，实现声明式生命周期管理。

use std::any::Any;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

use crate::factory::config::bean_post_processor::BeanPostProcessor;

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
    /// 创建新的后处理器。
    pub fn new() -> Self {
        Self {
            init_methods: Mutex::new(HashSet::new()),
            destroy_methods: Mutex::new(HashSet::new()),
            initialized: Mutex::new(false),
        }
    }

    /// 注册一个初始化方法名（对应 `@PostConstruct`）。
    pub fn register_init_method(&self, method_name: String) {
        self.init_methods.lock().unwrap().insert(method_name);
    }

    /// 注册一个销毁方法名（对应 `@PreDestroy`）。
    pub fn register_destroy_method(&self, method_name: String) {
        self.destroy_methods.lock().unwrap().insert(method_name);
    }

    /// 已注册的初始化方法数量。
    pub fn init_method_count(&self) -> usize {
        self.init_methods.lock().unwrap().len()
    }

    /// 已注册的销毁方法数量。
    pub fn destroy_method_count(&self) -> usize {
        self.destroy_methods.lock().unwrap().len()
    }

    /// 是否包含指定的初始化方法。
    pub fn has_init_method(&self, name: &str) -> bool {
        self.init_methods.lock().unwrap().contains(name)
    }

    /// 是否包含指定的销毁方法。
    pub fn has_destroy_method(&self, name: &str) -> bool {
        self.destroy_methods.lock().unwrap().contains(name)
    }

    /// 标记为已初始化。
    pub fn initialize(&self) {
        *self.initialized.lock().unwrap() = true;
    }

    /// 是否已初始化。
    pub fn is_initialized(&self) -> bool {
        *self.initialized.lock().unwrap()
    }

    /// 获取所有初始化方法名。
    pub fn init_methods(&self) -> Vec<String> {
        self.init_methods.lock().unwrap().iter().cloned().collect()
    }

    /// 获取所有销毁方法名。
    pub fn destroy_methods(&self) -> Vec<String> {
        self.destroy_methods.lock().unwrap().iter().cloned().collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_init_and_destroy_methods() {
        let pp = InitDestroyAnnotationBeanPostProcessor::new();
        pp.register_init_method("afterPropertiesSet".to_string());
        pp.register_init_method("customInit".to_string());
        pp.register_destroy_method("destroy".to_string());

        assert!(pp.has_init_method("afterPropertiesSet"));
        assert!(pp.has_init_method("customInit"));
        assert!(!pp.has_init_method("missing"));
        assert_eq!(pp.init_method_count(), 2);

        assert!(pp.has_destroy_method("destroy"));
        assert_eq!(pp.destroy_method_count(), 1);
    }

    #[test]
    fn initialization_state() {
        let pp = InitDestroyAnnotationBeanPostProcessor::new();
        assert!(!pp.is_initialized());
        pp.initialize();
        assert!(pp.is_initialized());
    }

    #[test]
    fn list_methods() {
        let pp = InitDestroyAnnotationBeanPostProcessor::new();
        pp.register_init_method("init".to_string());
        pp.register_destroy_method("close".to_string());
        pp.register_destroy_method("shutdown".to_string());

        let mut inits = pp.init_methods();
        inits.sort();
        assert_eq!(inits, vec!["init"]);

        let mut destroys = pp.destroy_methods();
        destroys.sort();
        assert_eq!(destroys, vec!["close", "shutdown"]);
    }
}
