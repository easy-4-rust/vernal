//! ApplicationEventPublisherAware — Spring 风格的 ApplicationEventPublisher 感知接口。
//!
//! 对应 Java 类：`org.springframework.context.ApplicationEventPublisherAware`。
//!
//! Bean 实现此接口后，容器在创建 Bean 时回调 `set_application_event_publisher`，
//! 将当前 `ApplicationEventPublisher` 注入 Bean。

use std::any::Any;
use std::sync::Arc;

use crate::aware::Aware;

/// Spring 风格的 ApplicationEventPublisher 感知接口。
///
/// 对应 Spring 的 `ApplicationEventPublisherAware.setApplicationEventPublisher(ApplicationEventPublisher applicationEventPublisher)`。
///
/// 容器在实例化 Bean 后，调用此方法将当前事件发布器注入。
///
/// 注意：由于 `dyn ApplicationEventPublisher` 不是 dyn-compatible 的（泛型方法），
/// 此接口使用 `Arc<dyn Any>` 传递引用。实现方需要 downcast 到具体类型。
pub trait ApplicationEventPublisherAware: Aware {
    /// 将当前 ApplicationEventPublisher 注入 Bean。
    ///
    /// 对应 Spring 的 `ApplicationEventPublisherAware.setApplicationEventPublisher(ApplicationEventPublisher applicationEventPublisher)`。
    ///
    /// `publisher` 是一个类型擦除的事件发布器引用。实现方可以通过
    /// `Arc::downcast_ref::<ConcretePublisher>()` 获取具体类型。
    fn set_application_event_publisher(&mut self, publisher: Arc<dyn Any + Send + Sync>);
}
