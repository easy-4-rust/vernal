//! 环境能力契约。
//!
//! 对标 Spring `org.springframework.core.env.EnvironmentCapable`。

use super::Environment;

/// 环境能力契约。
///
/// 对应 Java: org.springframework.core.env.EnvironmentCapable
///
/// Spring 语义：组件暴露其所属环境（对标 `ApplicationContext` 实现此接口）。
pub trait EnvironmentCapable: Send + Sync {
    /// 返回组件所属环境。
    fn environment(&self) -> &dyn Environment;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::StandardEnvironment;

    struct CapableComponent {
        env: StandardEnvironment,
    }

    impl EnvironmentCapable for CapableComponent {
        fn environment(&self) -> &dyn Environment {
            &self.env
        }
    }

    #[test]
    fn exposes_environment() {
        // A 类（合同对齐）：对标 Spring `getEnvironment()`
        let component = CapableComponent {
            env: StandardEnvironment::new(),
        };
        assert_eq!(
            component.environment().get_default_profiles(),
            vec!["default".to_string()]
        );
    }
}
