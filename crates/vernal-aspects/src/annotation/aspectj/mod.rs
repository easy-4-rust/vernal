//! 对标 `org.springframework.context.annotation.aspectj` 包（按审计脚本"保留末两层"规则）。
//!
//! 提供 Spring Configured 启用机制的 Rust 等价实现，覆盖：
//! - `SpringConfiguredConfiguration`：`@Configuration` 等价
//! - `EnableSpringConfigured`：启用注解等价

mod spring_configured_configuration;
mod enable_spring_configured;

pub use spring_configured_configuration::SpringConfiguredConfiguration;
pub use enable_spring_configured::{EnableSpringConfigured, enable_spring_configured, enable_and_register};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_configured_configuration() {
        let config = SpringConfiguredConfiguration::new();
        assert!(!config.is_registered());
    }

    #[test]
    fn test_enable_spring_configured() {
        let config = enable_spring_configured();
        assert!(!config.is_registered());
    }
}
