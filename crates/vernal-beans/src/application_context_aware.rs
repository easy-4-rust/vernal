//! ApplicationContextAware — Spring 风格的 ApplicationContext 感知接口。
//!
//! 对应 Java 类：`org.springframework.context.ApplicationContextAware`。
//!
//! Bean 实现此接口后，容器在创建 Bean 时回调 `set_application_context`，
//! 将当前 `ApplicationContext` 注入 Bean。

use std::any::Any;
use std::sync::Arc;

use crate::aware::Aware;

/// Spring 风格的 ApplicationContext 感知接口。
///
/// 对应 Spring 的 `ApplicationContextAware.setApplicationContext(ApplicationContext applicationContext)`。
///
/// 容器在实例化 Bean 后、初始化阶段之前，调用此方法将当前 ApplicationContext 注入。
///
/// 注意：由于 `dyn ApplicationContext` 不是 dyn-compatible 的（泛型方法），
/// 此接口使用 `Arc<dyn Any>` 传递容器引用。实现方需要 downcast 到具体类型。
pub trait ApplicationContextAware: Aware {
    /// 将当前 ApplicationContext 注入 Bean。
    ///
    /// 对应 Spring 的 `ApplicationContextAware.setApplicationContext(ApplicationContext applicationContext)`。
    ///
    /// `context` 是一个类型擦除的容器引用。实现方可以通过
    /// `Arc::downcast_ref::<ConcreteApplicationContext>()` 获取具体类型。
    fn set_application_context(&mut self, context: Arc<dyn Any + Send + Sync>);
}
