//! InstantiationStrategy — Spring 风格的 Bean 实例化策略 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.InstantiationStrategy`。
//!
//! 定义如何根据 Bean 定义创建 Bean 实例的策略接口。

use std::any::Any;
use std::sync::Arc;

use crate::bean_definition::BeanDefinition;

/// Spring 风格的 Bean 实例化策略 trait。
///
/// 对应 Spring 的 `InstantiationStrategy`。
///
/// 定义如何根据 `BeanDefinition` 创建 Bean 实例。不同的实现可以
/// 使用不同的实例化方式：
///
/// - `SimpleInstantiationStrategy` — 使用默认构造器或工厂方法
/// - `CglibInstantiationStrategy`（仅 Spring）— 使用 CGLIB 动态创建子类
///
/// 在 vernal 中，此 trait 用于将 Bean 的创建逻辑与容器解耦。
pub trait InstantiationStrategy: Send + Sync + std::fmt::Debug {
    /// 实例化 Bean。
    ///
    /// 对应 Spring 的 `Object instantiate(RootBeanDefinition bd, String beanName, BeanFactory owner)`。
    ///
    /// # 参数
    ///
    /// * `bd` — Bean 定义，包含类名、构造参数等信息
    /// * `bean_name` — Bean 的名称
    /// * `factory_bean_name` — 工厂 Bean 的名称（如果使用工厂 Bean 创建，否则为 `None`）
    /// * `factory_method_name` — 工厂方法名（如果使用工厂方法创建，否则为 `None`）
    /// * `args` — 构造参数或工厂方法参数（可选）
    ///
    /// # 返回
    ///
    /// - `Ok(Arc)` — 创建成功的 Bean 实例
    /// - `Err` — 实例化失败
    fn instantiate(
        &self,
        bd: &dyn BeanDefinition,
        bean_name: &str,
        factory_bean_name: Option<&str>,
        factory_method_name: Option<&str>,
        args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
}
