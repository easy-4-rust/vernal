//! 对标 `org.springframework.context.annotation.aspectj` 包。
//!
//! 提供 Spring Configured 启用注解的 Rust 等价实现。

mod spring_configured_configuration;

pub use spring_configured_configuration::SpringConfiguredConfiguration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_configured_configuration() {
        let config = SpringConfiguredConfiguration::new();
        let _ = config;
    }
}
