//! SimpleInstantiationStrategy — Spring 风格的简单实例化策略。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.SimpleInstantiationStrategy`。
//!
//! 在 Spring 中，`SimpleInstantiationStrategy` 使用反射直接构造 Bean 实例。
//! 当 Bean 有方法覆盖时，Spring 会切换到 CGLIB 策略。
//!
//! ## 设计说明
//!
//! 在 vernal 中，工厂闭包承担了反射构造的职责。
//! `SimpleInstantiationStrategy` 通过闭包创建实例，不涉及 CGLIB。

use crate::factory::support::instantiation_strategy::InstantiationStrategy;
use std::any::Any;
use std::sync::Arc;

/// 简单实例化策略。
///
/// 对应 Spring 的 `SimpleInstantiationStrategy`。
///
/// 直接使用工厂闭包创建 Bean 实例，不涉及 CGLIB 子类代理。
#[derive(Clone, Debug, Default)]
pub struct SimpleInstantiationStrategy;

impl SimpleInstantiationStrategy {
    /// 创建简单实例化策略。
    pub fn new() -> Self { Self }

    /// 使用工厂闭包实例化 Bean。
    ///
    /// 这是 `SimpleInstantiationStrategy` 的便捷方法。
    ///
    /// # 参数
    /// - `factory` — 工厂闭包，返回 Bean 实例
    pub fn instantiate_with_factory(
        &self,
        factory: &dyn Fn() -> Arc<dyn Any + Send + Sync>,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(factory())
    }

    /// 使用带参数的工厂闭包实例化 Bean。
    pub fn instantiate_with_args(
        &self,
        factory: &dyn Fn(&[Arc<dyn Any + Send + Sync>]) -> Arc<dyn Any + Send + Sync>,
        args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(factory(args))
    }
}

impl InstantiationStrategy for SimpleInstantiationStrategy {
    fn instantiate(
        &self,
        bean_class: &str,
        _args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Err(format!(
            "SimpleInstantiationStrategy: Bean '{}' requires a factory closure",
            bean_class
        ).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instantiate_with_factory() {
        let strategy = SimpleInstantiationStrategy::new();
        let result = strategy.instantiate_with_factory(&|| Arc::new(42_i32)).unwrap();
        assert_eq!(result.downcast_ref::<i32>(), Some(&42));
    }

    #[test]
    fn instantiate_with_args() {
        let strategy = SimpleInstantiationStrategy::new();
        let args: Vec<Arc<dyn Any + Send + Sync>> = vec![Arc::new(10_i32), Arc::new(20_i32)];
        let result = strategy.instantiate_with_args(
            &|args| {
                let sum: i32 = args.iter()
                    .filter_map(|a| a.downcast_ref::<i32>())
                    .sum();
                Arc::new(sum)
            },
            &args,
        ).unwrap();
        assert_eq!(result.downcast_ref::<i32>(), Some(&30));
    }

    #[test]
    fn trait_instantiate_returns_error() {
        let strategy = SimpleInstantiationStrategy::new();
        let result = strategy.instantiate("TestBean", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("factory closure"));
    }
}
