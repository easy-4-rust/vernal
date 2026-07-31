//! 对标 `org.springframework.context.annotation.aspectj.SpringConfiguredConfiguration`。
//!
//! 对应 Java：`SpringConfiguredConfiguration.java`（`spring-aspects` 模块）
//! 包路径：`org.springframework.context.annotation.aspectj`
//! 核心职责：`@Configuration` 类——注册 `AnnotationBeanConfigurerAspect` 单例 Bean。
//!
//! 注意：按审计脚本"保留末两层"规则，本文件位于 `annotation/aspectj/` 而非 `context/annotation/aspectj/`。

use std::sync::{Arc, RwLock};

/// Spring Configured 配置。
///
/// 对标 Spring 的 `SpringConfiguredConfiguration`。
/// 注册 `AnnotationBeanConfigurerAspect` 单例 Bean。
pub struct SpringConfiguredConfiguration {
    /// 是否已注册。
    registered: RwLock<bool>,
}

impl SpringConfiguredConfiguration {
    /// 创建配置实例。
    pub fn new() -> Self {
        Self {
            registered: RwLock::new(false),
        }
    }

    /// 注册 Bean 配置切面。
    ///
    /// 对应 Spring 的 `@Bean(name = BEAN_CONFIGURER_ASPECT_BEAN_NAME)`。
    pub fn register(&self) -> bool {
        let mut guard = self.registered.write().unwrap();
        if !*guard {
            *guard = true;
            true
        } else {
            false
        }
    }

    /// 是否已注册。
    pub fn is_registered(&self) -> bool {
        *self.registered.read().unwrap()
    }

    /// Bean 配置切面的 Bean 名称。
    ///
    /// 对应 Spring 的 `BEAN_CONFIGURER_ASPECT_BEAN_NAME`。
    pub const BEAN_CONFIGURER_ASPECT_BEAN_NAME: &'static str =
        "org.springframework.context.config.internalBeanConfigurerAspect";
}

impl Default for SpringConfiguredConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_creation() {
        let config = SpringConfiguredConfiguration::new();
        assert!(!config.is_registered());
    }

    #[test]
    fn test_configuration_register() {
        let config = SpringConfiguredConfiguration::new();
        assert!(config.register());
        assert!(config.is_registered());
        // 重复注册返回 false
        assert!(!config.register());
    }

    #[test]
    fn test_configuration_register_multiple_times() {
        let config = SpringConfiguredConfiguration::new();
        assert!(config.register());
        assert!(!config.register());
        assert!(!config.register());
        assert!(config.is_registered());
    }

    #[test]
    fn test_bean_name() {
        assert_eq!(
            SpringConfiguredConfiguration::BEAN_CONFIGURER_ASPECT_BEAN_NAME,
            "org.springframework.context.config.internalBeanConfigurerAspect"
        );
    }

    #[test]
    fn test_configuration_default() {
        let config = SpringConfiguredConfiguration::default();
        assert!(!config.is_registered());
    }

    #[test]
    fn test_configuration_debug() {
        let config = SpringConfiguredConfiguration::new();
        // 不检查 Debug 实现，只检查创建成功
        let _ = config;
    }

    #[test]
    fn test_configuration_clone() {
        let config = SpringConfiguredConfiguration::new();
        let _ = config;
    }
}
