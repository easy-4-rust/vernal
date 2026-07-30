//! InstantiationStrategy — Spring 风格的 Bean 实例化策略接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.InstantiationStrategy`。
//!
//! 定义 Bean 实例化的抽象策略，允许容器使用不同的方式创建 Bean 实例。

use std::any::Any;
use std::sync::Arc;

/// Spring 风格的 Bean 实例化策略接口。
///
/// 对应 Spring 的 `InstantiationStrategy`。
///
/// ## 实现
///
/// - `SimpleInstantiationStrategy` — 使用工厂闭包直接实例化
///
/// ## 设计说明
///
/// Spring 的 `InstantiationStrategy` 通过反射创建实例。
/// 在 vernal 中，由于 Rust 没有反射，实例化通过工厂闭包完成。
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
