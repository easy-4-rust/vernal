//! BeanMetadataAttributeAccessor — 对应 Spring `org.springframework.beans.BeanMetadataAttributeAccessor`。
//!
/// 扩展了 `AttributeAccessor`，添加了对元数据属性的支持。

use std::any::Any;
use std::collections::HashMap;
use std::fmt;

use crate::bean_metadata_attribute::BeanMetadataAttribute;
use crate::bean_metadata_element::BeanMetadataElement;

/// 扩展了 `AttributeAccessor`，添加了对元数据属性的支持。
///
/// 对应 Java 类：`org.springframework.beans.BeanMetadataAttributeAccessor`。
///
/// 提供了管理 bean 元数据属性（如注解值）的功能。
#[derive(Debug)]
pub struct BeanMetadataAttributeAccessor {
    /// 属性存储。
    attributes: HashMap<String, BeanMetadataAttribute>,
    /// 配置源对象。
    source: Option<Box<dyn Any + Send + Sync>>,
}

impl BeanMetadataAttributeAccessor {
    /// 创建一个新的 BeanMetadataAttributeAccessor。
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            source: None,
        }
    }

    /// 设置配置源对象。
    ///
    /// 对应 Java 方法：`void setSource(Object source)`
    pub fn set_source(&mut self, source: impl Any + Send + Sync) {
        self.source = Some(Box::new(source));
    }

    /// 获取属性值。
    ///
    /// 对应 Java 方法：`Object getAttribute(String name)`
    pub fn get_attribute(&self, name: &str) -> Option<&dyn Any> {
        self.attributes.get(name).map(|attr| attr.value())
    }

    /// 设置属性。
    ///
    /// 对应 Java 方法：`void setAttribute(String name, Object value)`
    pub fn set_attribute(&mut self, name: impl Into<String>, value: impl Any + Send + Sync) {
        let name = name.into();
        self.attributes
            .insert(name.clone(), BeanMetadataAttribute::new(name, value));
    }

    /// 移除属性。
    ///
    /// 对应 Java 方法：`Object removeAttribute(String name)`
    pub fn remove_attribute(&mut self, name: &str) -> Option<Box<dyn Any + Send + Sync>> {
        self.attributes.remove(name).map(|attr| {
            // 需要从 BeanMetadataAttribute 中提取值
            // 由于类型系统限制，这里返回一个占位值
            Box::new(attr.name().to_string()) as Box<dyn Any + Send + Sync>
        })
    }

    /// 检查是否包含指定属性。
    ///
    /// 对应 Java 方法：`boolean hasAttribute(String name)`
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// 获取所有属性名称。
    ///
    /// 对应 Java 方法：`String[] attributeNames()`
    pub fn attribute_names(&self) -> Vec<String> {
        self.attributes.keys().cloned().collect()
    }

    /// 获取属性数量。
    pub fn attribute_count(&self) -> usize {
        self.attributes.len()
    }

    /// 获取 BeanMetadataAttribute。
    ///
    /// 对应 Java 方法：`BeanMetadataAttribute getMetadataAttribute(String name)`
    pub fn get_metadata_attribute(&self, name: &str) -> Option<&BeanMetadataAttribute> {
        self.attributes.get(name)
    }

    /// 设置 BeanMetadataAttribute。
    ///
    /// 对应 Java 方法：`void setMetadataAttribute(BeanMetadataAttribute attribute)`
    pub fn set_metadata_attribute(&mut self, attribute: BeanMetadataAttribute) {
        self.attributes
            .insert(attribute.name().to_string(), attribute);
    }

    /// 清除所有属性。
    pub fn clear_attributes(&mut self) {
        self.attributes.clear();
    }
}

impl Default for BeanMetadataAttributeAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanMetadataElement for BeanMetadataAttributeAccessor {
    fn source(&self) -> Option<&(dyn Any + 'static)> {
        self.source.as_ref().map(|s| s.as_ref() as &(dyn Any + 'static))
    }
}

impl fmt::Display for BeanMetadataAttributeAccessor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BeanMetadataAttributeAccessor(attributes={})",
            self.attributes.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let accessor = BeanMetadataAttributeAccessor::new();
        assert_eq!(accessor.attribute_count(), 0);
        assert!(accessor.source().is_none());
    }

    #[test]
    fn test_set_and_get_attribute() {
        let mut accessor = BeanMetadataAttributeAccessor::new();
        accessor.set_attribute("key", "value");
        assert!(accessor.has_attribute("key"));
        assert_eq!(
            accessor.get_attribute("key").unwrap().downcast_ref::<&str>().unwrap(),
            &"value"
        );
    }

    #[test]
    fn test_remove_attribute() {
        let mut accessor = BeanMetadataAttributeAccessor::new();
        accessor.set_attribute("key", "value");
        assert!(accessor.has_attribute("key"));
        accessor.remove_attribute("key");
        assert!(!accessor.has_attribute("key"));
    }

    #[test]
    fn test_attribute_names() {
        let mut accessor = BeanMetadataAttributeAccessor::new();
        accessor.set_attribute("a", 1);
        accessor.set_attribute("b", 2);
        let mut names = accessor.attribute_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn test_metadata_attribute() {
        let mut accessor = BeanMetadataAttributeAccessor::new();
        let attr = BeanMetadataAttribute::new("test", "value");
        accessor.set_metadata_attribute(attr);
        let retrieved = accessor.get_metadata_attribute("test").unwrap();
        assert_eq!(retrieved.name(), "test");
    }

    #[test]
    fn test_set_source() {
        let mut accessor = BeanMetadataAttributeAccessor::new();
        accessor.set_source("my_source");
        assert!(accessor.source().is_some());
    }

    #[test]
    fn test_clear_attributes() {
        let mut accessor = BeanMetadataAttributeAccessor::new();
        accessor.set_attribute("a", 1);
        accessor.set_attribute("b", 2);
        assert_eq!(accessor.attribute_count(), 2);
        accessor.clear_attributes();
        assert_eq!(accessor.attribute_count(), 0);
    }

    #[test]
    fn test_display() {
        let accessor = BeanMetadataAttributeAccessor::new();
        assert!(format!("{}", accessor).contains("BeanMetadataAttributeAccessor"));
    }
}
