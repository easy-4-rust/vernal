//! 对标 `org.springframework.context.annotation.aspectj.EnableSpringConfigured` 注解。
//!
//! 对应 Java：`EnableSpringConfigured.java`（`spring-aspects` 模块）
//! 包路径：`org.springframework.context.annotation.aspectj`
//! 核心职责：启用注解——引入 `SpringConfiguredConfiguration`，激活 `@Configurable` DI 切面。
//!
//! Spring 原始实现：`@Import(SpringConfiguredConfiguration.class)`
//! Rust 等价：提供 `enable_spring_configured()` 函数，返回 `SpringConfiguredConfiguration` 实例。

use super::spring_configured_configuration::SpringConfiguredConfiguration;

/// 启用 Spring Configured 注解。
///
/// 对标 Spring 的 `@EnableSpringConfigured` 注解。
/// 提供启用 `@Configurable` DI 切面的功能。
pub struct EnableSpringConfigured;

impl EnableSpringConfigured {
    /// 启用 Spring Configured 功能。
    ///
    /// 返回 `SpringConfiguredConfiguration` 实例。
    pub fn enable() -> SpringConfiguredConfiguration {
        SpringConfiguredConfiguration::new()
    }

    /// 启用 Spring Configured 功能并注册切面。
    pub fn enable_and_register() -> SpringConfiguredConfiguration {
        let config = SpringConfiguredConfiguration::new();
        config.register();
        config
    }
}

/// 启用 Spring Configured 功能。
///
/// 对标 Spring 的 `@EnableSpringConfigured` 注解。
/// 返回 `SpringConfiguredConfiguration` 实例，可用于注册 Bean 配置切面。
///
/// # Example
///
/// ```rust
/// use vernal_aspects::annotation::aspectj::enable_spring_configured;
///
/// let config = enable_spring_configured();
/// assert!(!config.is_registered());
/// ```
pub fn enable_spring_configured() -> SpringConfiguredConfiguration {
    SpringConfiguredConfiguration::new()
}

/// 启用 Spring Configured 功能并注册切面。
///
/// 对标 Spring 的 `@EnableSpringConfigured` + `@Configuration` 组合。
pub fn enable_and_register() -> SpringConfiguredConfiguration {
    let config = SpringConfiguredConfiguration::new();
    config.register();
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enable_spring_configured() {
        let config = enable_spring_configured();
        assert!(!config.is_registered());
        assert_eq!(
            SpringConfiguredConfiguration::BEAN_CONFIGURER_ASPECT_BEAN_NAME,
            "org.springframework.context.config.internalBeanConfigurerAspect"
        );
    }

    #[test]
    fn test_enable_and_register() {
        let config = enable_and_register();
        assert!(config.is_registered());
    }

    #[test]
    fn test_enable_spring_configured_idempotent() {
        let config = enable_and_register();
        assert!(config.is_registered());
        // Second register should return false
        assert!(!config.register());
    }
}
