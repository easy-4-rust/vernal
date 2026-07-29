//! EnvironmentAware — Spring 风格的 Environment 感知接口。
//!
//! 对应 Java 类：`org.springframework.context.EnvironmentAware`。
//!
//! Bean 实现此接口后，容器在创建 Bean 时回调 `set_environment`，
//! 将当前 `Environment` 注入 Bean。

use std::any::Any;
use std::sync::Arc;

use crate::aware::Aware;

/// Spring 风格的 Environment 感知接口。
///
/// 对应 Spring 的 `EnvironmentAware.setEnvironment(Environment environment)`。
///
/// 容器在实例化 Bean 后，调用此方法将当前 Environment 注入。
///
/// 注意：由于 `dyn Environment` 不是 dyn-compatible 的（泛型方法），
/// 此接口使用 `Arc<dyn Any>` 传递引用。实现方需要 downcast 到具体类型。
pub trait EnvironmentAware: Aware {
    /// 将当前 Environment 注入 Bean。
    ///
    /// 对应 Spring 的 `EnvironmentAware.setEnvironment(Environment environment)`。
    ///
    /// `env` 是一个类型擦除的环境引用。实现方可以通过
    /// `Arc::downcast_ref::<ConcreteEnvironment>()` 获取具体类型。
    fn set_environment(&mut self, env: Arc<dyn Any + Send + Sync>);
}
