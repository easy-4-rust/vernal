//! NamedBeanHolder — Spring 风格的带名称 Bean 持有者。
//!
//! 对应 Java 类：`org.springframework.beans.factory.NamedBeanHolder`。
//!
//! 用于返回 Bean 实例及其在容器中的名称。

use std::any::Any;
use std::sync::Arc;

/// Spring 风格的带名称 Bean 持有者。
///
/// 对应 Spring 的 `NamedBeanHolder<T>`。
///
/// 当容器需要同时返回 Bean 实例和其名称时使用此类型。
/// 例如 `AutowireCapableBeanFactory.resolveNamedBean(Class<T>)` 返回此类型。
#[derive(Debug)]
pub struct NamedBeanHolder<T: Any + Send + Sync> {
    /// Bean 实例。
    instance: Arc<T>,
    /// Bean 在容器中的名称。
    bean_name: String,
}

impl<T: Any + Send + Sync> NamedBeanHolder<T> {
    /// 创建新的 NamedBeanHolder。
    pub fn new(instance: Arc<T>, bean_name: impl Into<String>) -> Self {
        Self {
            instance,
            bean_name: bean_name.into(),
        }
    }

    /// 获取 Bean 实例。
    pub fn instance(&self) -> &Arc<T> {
        &self.instance
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 消费并返回内部值。
    pub fn into_inner(self) -> (Arc<T>, String) {
        (self.instance, self.bean_name)
    }
}
