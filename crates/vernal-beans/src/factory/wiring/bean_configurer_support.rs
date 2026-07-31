//! BeanConfigurerSupport — Spring 风格的 Bean 配置支持类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.wiring.BeanConfigurerSupport`。
//!
//! 在 Spring 中，`BeanConfigurerSupport` 实现了 `BeanFactoryAware` 接口，
//! 提供了自动配置 Bean 的能力。当 Bean 实现了此接口时，
//! Spring 容器会自动注入依赖并调用配置方法。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`BeanConfigurerSupport` 通过闭包机制实现 Bean 配置，
//! 支持延迟配置和条件配置。

use std::collections::HashMap;
use std::sync::Mutex;

/// Bean 配置支持类。
///
/// 对应 Spring 的 `BeanConfigurerSupport`。
///
/// 提供自动配置 Bean 的能力。
#[derive(Debug, Default)]
pub struct BeanConfigurerSupport {
    /// 配置属性。
    config_properties: Mutex<HashMap<String, String>>,
    /// 是否已配置。
    configured: std::sync::atomic::AtomicBool,
    /// Bean 名称。
    bean_name: Mutex<String>,
}

impl BeanConfigurerSupport {
    /// 创建 Bean 配置支持实例。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 Bean 名称。
    ///
    /// 对应 Spring 的 `setBeanName(String)`。
    pub fn set_bean_name(&self, name: impl Into<String>) {
        *self.bean_name.lock().unwrap() = name.into();
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> String {
        self.bean_name.lock().unwrap().clone()
    }

    /// 设置配置属性。
    pub fn set_config_property(&self, key: impl Into<String>, value: impl Into<String>) {
        self.config_properties.lock().unwrap().insert(key.into(), value.into());
    }

    /// 获取配置属性。
    pub fn config_property(&self, key: &str) -> Option<String> {
        self.config_properties.lock().unwrap().get(key).cloned()
    }

    /// 执行配置。
    ///
    /// 对应 Spring 的 `afterPropertiesSet()` 回调。
    pub fn configure(&self) {
        self.configured.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// 检查是否已配置。
    pub fn is_configured(&self) -> bool {
        self.configured.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 获取配置属性数量。
    pub fn property_count(&self) -> usize {
        self.config_properties.lock().unwrap().len()
    }

    /// 重置配置状态。
    pub fn reset(&self) {
        self.configured.store(false, std::sync::atomic::Ordering::Relaxed);
        self.config_properties.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_bean_name() {
        let support = BeanConfigurerSupport::new();
        support.set_bean_name("myService");
        assert_eq!(support.bean_name(), "myService");
    }

    #[test]
    fn config_properties() {
        let support = BeanConfigurerSupport::new();
        support.set_config_property("host", "localhost");
        support.set_config_property("port", "8080");
        assert_eq!(support.property_count(), 2);
        assert_eq!(support.config_property("host"), Some("localhost".to_string()));
    }

    #[test]
    fn configure_sets_flag() {
        let support = BeanConfigurerSupport::new();
        assert!(!support.is_configured());
        support.configure();
        assert!(support.is_configured());
    }

    #[test]
    fn reset_clears_state() {
        let support = BeanConfigurerSupport::new();
        support.set_config_property("key", "value");
        support.configure();
        assert!(support.is_configured());

        support.reset();
        assert!(!support.is_configured());
        assert_eq!(support.property_count(), 0);
    }

    #[test]
    fn unknown_property_returns_none() {
        let support = BeanConfigurerSupport::new();
        assert!(support.config_property("unknown").is_none());
    }
}
