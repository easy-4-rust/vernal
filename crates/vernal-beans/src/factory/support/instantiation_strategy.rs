//! InstantiationStrategy — Spring 风格的 Bean 实例化策略接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.InstantiationStrategy`。
//!
//! 在 Spring 中，`InstantiationStrategy` 定义了 Bean 实例化的抽象策略，
//! 允许容器使用不同的方式创建 Bean 实例。Spring 有两个主要实现：
//! - `SimpleInstantiationStrategy` — 直接反射构造（默认）
//! - `CglibSubclassingInstantiationStrategy` — CGLIB 子类代理（用于方法覆盖）
//!
//! ## 设计说明
//!
//! Spring 的 `InstantiationStrategy` 通过反射创建实例。
//! 在 vernal 中，由于 Rust 没有反射，实例化通过工厂闭包完成。

use std::any::Any;
use std::sync::Arc;

/// Bean 实例化策略接口。
///
/// 对应 Spring 的 `InstantiationStrategy`。
///
/// 定义 Bean 实例化的抽象策略。
pub trait InstantiationStrategy: Send + Sync + 'static {
    /// 使用工厂闭包实例化 Bean。
    ///
    /// 对应 Spring 的 `InstantiationStrategy.instantiate(RootBeanDefinition, String, BeanFactory)`。
    ///
    /// # 参数
    ///
    /// - `bean_class` — Bean 类型名（用于错误信息）
    /// - `args` — 构造参数
    ///
    /// # 错误
    ///
    /// 实例化失败时返回错误。
    fn instantiate(
        &self,
        bean_class: &str,
        args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 工厂闭包实例化策略。
///
/// 使用闭包作为工厂方法创建 Bean 实例。
/// 这是 vernal 中最常用的实例化策略。
pub struct FactoryClosureStrategy;

impl FactoryClosureStrategy {
    /// 创建一个新的实例。
    pub fn new() -> Self { Self }
}

impl Default for FactoryClosureStrategy {
    fn default() -> Self { Self::new() }
}

impl InstantiationStrategy for FactoryClosureStrategy {
    fn instantiate(
        &self,
        bean_class: &str,
        _args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Err(format!(
            "FactoryClosureStrategy: Bean '{}' requires a factory closure, use SimpleInstantiationStrategy::instantiate_with_factory",
            bean_class
        ).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_closure_strategy_returns_error() {
        let strategy = FactoryClosureStrategy::new();
        let result = strategy.instantiate("MyBean", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("factory closure"));
    }
}
