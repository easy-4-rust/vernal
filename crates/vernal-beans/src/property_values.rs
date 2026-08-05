//! PropertyValues — 对应 Spring `org.springframework.beans.PropertyValues`。
//!
//! 属性值集合接口。

use crate::property_value::PropertyValue;

/// 属性值集合接口。
///
/// 对应 Java 接口：`org.springframework.beans.PropertyValues`。
///
/// 包含一个或多个 PropertyValue 对象的容器。
pub trait PropertyValues: Send + Sync {
    /// 获取所有属性值。
    fn property_values(&self) -> &[PropertyValue];

    /// 按名称获取属性值。
    fn get_property_value(&self, name: &str) -> Option<&PropertyValue>;

    /// 检查是否包含指定属性。
    fn contains(&self, name: &str) -> bool;

    /// 获取属性值数量。
    fn len(&self) -> usize;

    /// 检查是否为空。
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// PropertyValues 的简单实现。
#[derive(Debug, Clone, Default)]
pub struct MutablePropertyValuesImpl {
    /// 属性值列表。
    values: Vec<PropertyValue>,
}

impl MutablePropertyValuesImpl {
    /// 创建一个新的 MutablePropertyValuesImpl。
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// 添加属性值。
    pub fn add(&mut self, pv: PropertyValue) {
        self.values.push(pv);
    }

    /// 移除属性值。
    pub fn remove(&mut self, name: &str) -> Option<PropertyValue> {
        if let Some(pos) = self.values.iter().position(|pv| pv.name() == name) {
            Some(self.values.remove(pos))
        } else {
            None
        }
    }

    /// 清除所有属性值。
    pub fn clear(&mut self) {
        self.values.clear();
    }
}

impl PropertyValues for MutablePropertyValuesImpl {
    fn property_values(&self) -> &[PropertyValue] {
        &self.values
    }

    fn get_property_value(&self, name: &str) -> Option<&PropertyValue> {
        self.values.iter().find(|pv| pv.name() == name)
    }

    fn contains(&self, name: &str) -> bool {
        self.values.iter().any(|pv| pv.name() == name)
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

/// 空的 PropertyValues 实现。
#[derive(Debug, Clone, Copy)]
pub struct EmptyPropertyValues;

impl PropertyValues for EmptyPropertyValues {
    fn property_values(&self) -> &[PropertyValue] {
        &[]
    }

    fn get_property_value(&self, _name: &str) -> Option<&PropertyValue> {
        None
    }

    fn contains(&self, _name: &str) -> bool {
        false
    }

    fn len(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_property_values() {
        let pvs = EmptyPropertyValues;
        assert!(pvs.is_empty());
        assert_eq!(pvs.len(), 0);
        assert!(!pvs.contains("any"));
        assert!(pvs.get_property_value("any").is_none());
    }

    #[test]
    fn test_mutable_property_values() {
        let mut pvs = MutablePropertyValuesImpl::new();
        assert!(pvs.is_empty());

        pvs.add(PropertyValue::new(
            "name",
            std::sync::Arc::new(String::from("Alice"))
                as std::sync::Arc<dyn std::any::Any + Send + Sync>,
        ));
        pvs.add(PropertyValue::new(
            "age",
            std::sync::Arc::new(30i32) as std::sync::Arc<dyn std::any::Any + Send + Sync>,
        ));
        assert_eq!(pvs.len(), 2);
        assert!(!pvs.is_empty());
        assert!(pvs.contains("name"));
        assert!(pvs.contains("age"));
        assert!(!pvs.contains("missing"));
    }

    #[test]
    fn test_get_property_value() {
        let mut pvs = MutablePropertyValuesImpl::new();
        pvs.add(PropertyValue::new(
            "name",
            std::sync::Arc::new(String::from("Alice"))
                as std::sync::Arc<dyn std::any::Any + Send + Sync>,
        ));
        let pv = pvs.get_property_value("name").unwrap();
        assert_eq!(pv.name(), "name");
    }

    #[test]
    fn test_remove() {
        let mut pvs = MutablePropertyValuesImpl::new();
        pvs.add(PropertyValue::new(
            "name",
            std::sync::Arc::new(String::from("Alice"))
                as std::sync::Arc<dyn std::any::Any + Send + Sync>,
        ));
        assert_eq!(pvs.len(), 1);
        pvs.remove("name");
        assert_eq!(pvs.len(), 0);
    }

    #[test]
    fn test_clear() {
        let mut pvs = MutablePropertyValuesImpl::new();
        pvs.add(PropertyValue::new(
            "a",
            std::sync::Arc::new(1i32) as std::sync::Arc<dyn std::any::Any + Send + Sync>,
        ));
        pvs.add(PropertyValue::new(
            "b",
            std::sync::Arc::new(2i32) as std::sync::Arc<dyn std::any::Any + Send + Sync>,
        ));
        pvs.clear();
        assert!(pvs.is_empty());
    }
}
