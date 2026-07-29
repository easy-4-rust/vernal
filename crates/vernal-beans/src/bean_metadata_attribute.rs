//! BeanMetadataAttribute — Spring 风格的 Bean 元数据属性。
//!
//! 对应 Java 类：`org.springframework.beans.BeanMetadataAttribute`。
//!
//! 表示一个具名的元数据属性值，可关联配置来源。

use std::any::Any;

use crate::bean_metadata_element::BeanMetadataElement;

/// Spring 风格的 Bean 元数据属性。
///
/// 对应 Spring 的 `BeanMetadataAttribute`。
///
/// 存储一个具名的元数据属性，具有名称和可选的值，
/// 并实现 `BeanMetadataElement` 以支持关联配置来源。
#[derive(Debug)]
pub struct BeanMetadataAttribute {
    /// 属性名称。
    name: String,
    /// 属性值（类型擦除）。
    value: Option<Box<dyn Any + Send>>,
}

impl BeanMetadataAttribute {
    /// 创建新的 BeanMetadataAttribute。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
        }
    }

    /// 创建带值的 BeanMetadataAttribute。
    pub fn with_value(name: impl Into<String>, value: Box<dyn Any + Send>) -> Self {
        Self {
            name: name.into(),
            value: Some(value),
        }
    }

    /// 获取属性名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取属性值引用。
    pub fn value(&self) -> Option<&dyn Any> {
        self.value.as_deref().map(|v| v as &dyn Any)
    }

    /// 设置属性值。
    pub fn set_value(&mut self, value: Box<dyn Any + Send>) {
        self.value = Some(value);
    }
}

impl BeanMetadataElement for BeanMetadataAttribute {
    fn get_source(&self) -> Option<&dyn Any> {
        None
    }
}
