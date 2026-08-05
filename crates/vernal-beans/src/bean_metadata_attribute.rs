//! BeanMetadataAttribute — 对应 Spring `org.springframework.beans.BeanMetadataAttribute`。
//!
//! 保存单个 bean 元数据属性的类。

use std::any::Any;
use std::fmt;

use crate::bean_metadata_element::BeanMetadataElement;

/// 保存单个 bean 元数据属性的类。
///
/// 对应 Java 类：`org.springframework.beans.BeanMetadataAttribute`。
///
/// 用于存储 bean 定义中的元数据属性，如注解值等。
#[derive(Debug)]
pub struct BeanMetadataAttribute {
    /// 属性名称。
    name: String,
    /// 属性值。
    value: Box<dyn Any + Send + Sync>,
    /// 配置源对象。
    source: Option<Box<dyn Any + Send + Sync>>,
}

impl BeanMetadataAttribute {
    /// 创建一个新的 BeanMetadataAttribute。
    ///
    /// 对应 Java 构造器：`BeanMetadataAttribute(String name, Object value)`
    pub fn new(name: impl Into<String>, value: impl Any + Send + Sync) -> Self {
        Self {
            name: name.into(),
            value: Box::new(value),
            source: None,
        }
    }

    /// 创建一个带有配置源的 BeanMetadataAttribute。
    ///
    /// 对应 Java 构造器：`BeanMetadataAttribute(String name, Object value, Object source)`
    pub fn with_source(
        name: impl Into<String>,
        value: impl Any + Send + Sync,
        source: impl Any + Send + Sync,
    ) -> Self {
        Self {
            name: name.into(),
            value: Box::new(value),
            source: Some(Box::new(source)),
        }
    }

    /// 获取属性名称。
    ///
    /// 对应 Java 方法：`String getName()`
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取属性值。
    ///
    /// 对应 Java 方法：`Object getValue()`
    pub fn value(&self) -> &dyn Any {
        self.value.as_ref()
    }

    /// 获取可变属性值。
    pub fn value_mut(&mut self) -> &mut (dyn Any + Send + Sync) {
        self.value.as_mut()
    }

    /// 设置属性值。
    ///
    /// 对应 Java 方法：`void setValue(Object value)`
    pub fn set_value(&mut self, value: impl Any + Send + Sync) {
        self.value = Box::new(value);
    }

    /// 设置配置源对象。
    pub fn set_source(&mut self, source: impl Any + Send + Sync) {
        self.source = Some(Box::new(source));
    }
}

impl BeanMetadataElement for BeanMetadataAttribute {
    fn source(&self) -> Option<&(dyn Any + 'static)> {
        self.source
            .as_ref()
            .map(|s| s.as_ref() as &(dyn Any + 'static))
    }
}

impl fmt::Display for BeanMetadataAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BeanMetadataAttribute(name='{}')", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let attr = BeanMetadataAttribute::new("test", String::from("value"));
        assert_eq!(attr.name(), "test");
        assert_eq!(attr.value().downcast_ref::<String>().unwrap(), "value");
    }

    #[test]
    fn test_with_source() {
        let attr = BeanMetadataAttribute::with_source("test", "value", "source");
        assert_eq!(attr.name(), "test");
        assert!(attr.source().is_some());
    }

    #[test]
    fn test_set_value() {
        let mut attr = BeanMetadataAttribute::new("test", "old");
        attr.set_value("new");
        assert_eq!(attr.value().downcast_ref::<&str>().unwrap(), &"new");
    }

    #[test]
    fn test_set_source() {
        let mut attr = BeanMetadataAttribute::new("test", "value");
        assert!(attr.source().is_none());
        attr.set_source("source");
        assert!(attr.source().is_some());
    }

    #[test]
    fn test_display() {
        let attr = BeanMetadataAttribute::new("myAttr", "val");
        assert_eq!(format!("{}", attr), "BeanMetadataAttribute(name='myAttr')");
    }
}
