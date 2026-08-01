//! 属性访问器支持实现。
//!
//! 对标 Spring `org.springframework.core.AttributeAccessorSupport`。

use std::any::Any;
use std::collections::HashMap;

use crate::AttributeAccessor;

/// 属性访问器支持实现。
///
/// 对应 Java: org.springframework.core.AttributeAccessorSupport
///
/// Spring 语义：基于 `LinkedHashMap` 的属性存取实现。
#[derive(Debug, Default)]
pub struct AttributeAccessorSupport {
    attributes: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl AttributeAccessorSupport {
    /// 创建空访问器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl AttributeAccessor for AttributeAccessorSupport {
    fn set_attribute(&mut self, name: &str, value: Box<dyn Any + Send + Sync>) {
        self.attributes.insert(name.to_string(), value);
    }

    fn get_attribute(&self, name: &str) -> Option<&dyn Any> {
        self.attributes.get(name).map(|b| b.as_ref() as &dyn Any)
    }

    fn remove_attribute(&mut self, name: &str) -> Option<Box<dyn Any + Send + Sync>> {
        self.attributes.remove(name)
    }

    fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    fn attribute_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.attributes.keys().cloned().collect();
        names.sort();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_gets_and_removes() {
        // A 类（合同对齐）：对标 Spring attribute 生命周期
        let mut accessor = AttributeAccessorSupport::new();
        accessor.set_attribute("key", Box::new(42_i32));
        assert!(accessor.has_attribute("key"));
        let value = accessor.get_attribute("key").unwrap();
        assert_eq!(value.downcast_ref::<i32>(), Some(&42));
        let removed = accessor.remove_attribute("key");
        assert!(removed.is_some());
        assert!(!accessor.has_attribute("key"));
    }

    #[test]
    fn missing_attribute_returns_none() {
        // B 类（边界行为）
        let mut accessor = AttributeAccessorSupport::new();
        assert!(accessor.get_attribute("missing").is_none());
        assert!(accessor.remove_attribute("missing").is_none());
    }

    #[test]
    fn lists_attribute_names() {
        // B 类（边界行为）：对标 Spring `attributeNames`
        let mut accessor = AttributeAccessorSupport::new();
        accessor.set_attribute("b", Box::new(1_i32));
        accessor.set_attribute("a", Box::new(2_i32));
        assert_eq!(accessor.attribute_names(), vec!["a", "b"]);
    }
}
