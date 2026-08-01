//! 应用启动契约。
//!
//! 对标 Spring `org.springframework.core.metrics.ApplicationStartup`。

use super::StartupStep;

/// 应用启动契约。
///
/// 对应 Java: org.springframework.core.metrics.ApplicationStartup
///
/// Spring 语义：应用启动观测的入口——`start(name)` 创建并开始一个步骤。
pub trait ApplicationStartup: Send + Sync {
    /// 开始一个启动步骤。
    ///
    /// 对应 Java: `ApplicationStartup#start(String)`
    fn start(&self, name: &str) -> Box<dyn StartupStep>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::DefaultApplicationStartup;

    #[test]
    fn default_startup_satisfies_contract() {
        // D 类（重构安全）：`DefaultApplicationStartup` 实现该契约
        fn assert_startup<T: ApplicationStartup>() {}
        assert_startup::<DefaultApplicationStartup>();
    }
}
