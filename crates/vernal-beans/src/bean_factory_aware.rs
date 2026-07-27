//! BeanFactoryAware — Spring 风格的 BeanFactory 感知接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.BeanFactoryAware`。
//!
//! Bean 实现此接口后，容器在创建 Bean 时回调 `set_bean_factory`，
//! 将当前 `BeanFactory` 注入 Bean。

use std::sync::Arc;

use crate::aware::Aware;

/// Spring 风格的 BeanFactory 感知接口。
///
/// 对应 Spring 的 `BeanFactoryAware.setBeanFactory(BeanFactory beanFactory)`。
///
/// 容器在实例化 Bean 后、`BeanPostProcessor.postProcessBeforeInitialization` 之前，
/// 调用此方法将当前 BeanFactory 注入。
///
/// 注意：由于 `dyn BeanFactory` 不是 dyn-compatible 的（泛型方法），
/// 此接口使用 `Arc<dyn Any>` 传递容器引用。实现方需要 downcast 到具体类型。
pub trait BeanFactoryAware: Aware {
    /// 将当前 BeanFactory 注入 Bean。
    ///
    /// 对应 Spring 的 `BeanFactoryAware.setBeanFactory(BeanFactory beanFactory)`。
    ///
    /// `bean_factory` 是一个类型擦除的容器引用。实现方可以通过
    /// `Arc::downcast_ref::<ConcreteContainer>()` 获取具体类型。
    fn set_bean_factory(&mut self, bean_factory: Arc<dyn std::any::Any + Send + Sync>);
}
