//! PropertyValue — Spring 风格的单个属性值。
//!
//! 对应 Java 类：`org.springframework.beans.PropertyValue`。
//!
//! 存储一个 Bean 属性的名称和值。

use std::any::Any;
use std::sync::Arc;

/// Spring 风格的单个属性值。
///
/// 对应 Spring 的 `PropertyValue`。
///
/// 存储一个 Bean 属性的名称和值。用于 `MutablePropertyValues`。
#[derive(Clone, Debug)]
pub struct PropertyValue {
    /// 属性名称。
    name: String,
    /// 属性值（类型擦除）。
    value: Arc<dyn Any + Send + Sync>,
}

impl PropertyValue {
    /// 创建新的 PropertyValue。
    pub fn new(name: impl Into<String>, value: Arc<dyn Any + Send + Sync>) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// 获取属性名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取属性值。
    pub fn value(&self) -> &Arc<dyn Any + Send + Sync> {
        &self.value
    }
}
