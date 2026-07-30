//! SimpleInstantiationStrategy — Spring 风格的简单实例化策略。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.SimpleInstantiationStrategy`。
//!
//! 使用工厂闭包创建 Bean 实例。这是 vernal 中默认的实例化策略。

use crate::instantiation_strategy::InstantiationStrategy;
use std::any::Any;
use std::sync::Arc;

/// Spring 风格的简单实例化策略。
///
/// 对应 Spring 的 `SimpleInstantiationStrategy`。
///
/// 直接使用工厂闭包创建 Bean 实例，不涉及 CGLIB 子类代理。
///
/// ## 设计说明
///
/// Spring 的 `SimpleInstantiationStrategy` 在有方法替换时使用 CGLIB，
/// 否则直接反射构造。在 vernal 中，工厂闭包承担了反射构造的职责，
/// 方法替换通过 trait 对象的动态分派实现。
#[derive(Clone, Debug, Default)]
pub struct SimpleInstantiationStrategy;

impl SimpleInstantiationStrategy {
    /// 创建简单实例化策略。
    pub fn new() -> Self {
        Self
    }

    /// 使用工厂闭包实例化 Bean。
    ///
    /// 这是 `SimpleInstantiationStrategy` 的便捷方法，
    /// 不需要 `bean_class` 和 `args` 参数。
    ///
    /// # 参数
    ///
    /// - `factory` — 工厂闭包，返回 Bean 实例
    ///
    /// # 错误
    ///
    /// 工厂闭包返回 `Err` 时，此方法传播该错误。
    pub fn instantiate(
        &self,
        factory: &dyn Fn() -> Arc<dyn Any + Send + Sync>,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(factory())
    }
}

impl InstantiationStrategy for SimpleInstantiationStrategy {
    /// 使用工厂闭包实例化 Bean。
    ///
    /// 实现 `InstantiationStrategy` trait。`args` 参数在此实现中被忽略，
    /// 因为工厂闭包已经捕获了所有构造上下文。
    fn instantiate(
        &self,
        bean_class: &str,
        _args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Err(format!(
            "SimpleInstantiationStrategy: Bean '{}' 需要工厂闭包才能实例化，请使用 `instantiate_with_factory` 方法",
            bean_class
        )
        .into())
    }
}
