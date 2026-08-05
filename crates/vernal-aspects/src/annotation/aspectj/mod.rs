//! 对标 `org.springframework.context.annotation.aspectj` 包（按审计脚本"保留末两层"规则）。
//!
//! 提供 Spring Configured 启用机制的 Rust 等价实现，覆盖：
//! - `SpringConfiguredConfiguration`：`@Configuration` 等价
//! - `EnableSpringConfigured`：启用注解等价

mod enable_spring_configured;
mod spring_configured_configuration;

pub use enable_spring_configured::{
    EnableSpringConfigured, enable_and_register, enable_spring_configured,
};
pub use spring_configured_configuration::SpringConfiguredConfiguration;

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
