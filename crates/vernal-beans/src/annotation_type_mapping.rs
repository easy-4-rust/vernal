//! AnnotationTypeMapping — Spring 风格的注解类型映射。
//!
//! 对应 Java 类：`org.springframework.core.annotation.AnnotationTypeMapping`。
//!
//! 表示注解类型与其属性之间的映射关系，用于在注解处理过程中
//! 解析元注解和属性覆盖。

use std::collections::HashMap;

/// Spring 风格的注解类型映射。
///
/// 对应 Spring 的 `AnnotationTypeMapping`。
///
/// 描述单个注解类型的元数据：注解类型名、属性集合以及与源注解的关系。
/// 该类型作为注解处理流水线中的中间表示。
#[derive(Debug, Clone)]
pub struct AnnotationTypeMapping {
    /// 注解类型名（如 `Component`、`Service`）。
    annotation_type: String,
    /// 注解属性键值对。
    attributes: HashMap<String, String>,
    /// 源注解类型名（如果当前注解是通过元注解链发现的）。
    source: Option<String>,
    /// 在注解链中的距离（0 表示直接注解）。
    distance: usize,
}

impl AnnotationTypeMapping {
    /// 创建新的注解类型映射。
    pub fn new(annotation_type: impl Into<String>) -> Self {
        Self {
            annotation_type: annotation_type.into(),
            attributes: HashMap::new(),
            source: None,
            distance: 0,
        }
    }

    /// 创建带源与距离的注解类型映射。
    pub fn with_source(
        annotation_type: impl Into<String>,
        source: impl Into<String>,
        distance: usize,
    ) -> Self {
        Self {
            annotation_type: annotation_type.into(),
            attributes: HashMap::new(),
            source: Some(source.into()),
            distance,
        }
    }

    /// 获取注解类型名。
    pub fn annotation_type(&self) -> &str {
        &self.annotation_type
    }

    /// 获取注解属性。
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    /// 获取可变注解属性。
    pub fn attributes_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.attributes
    }

    /// 获取源注解类型名。
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    /// 获取在注解链中的距离。
    pub fn distance(&self) -> usize {
        self.distance
    }

    /// 是否为直接注解（距离为 0）。
    pub fn is_direct(&self) -> bool {
        self.distance == 0
    }

    /// 是否为元注解（距离大于 0）。
    pub fn is_meta(&self) -> bool {
        self.distance > 0
    }

    /// 插入一个属性。
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// 获取指定属性。
    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(String::as_str)
    }

    /// 是否包含指定属性。
    pub fn has_attribute(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    /// 属性数量。
    pub fn attribute_count(&self) -> usize {
        self.attributes.len()
    }
}

impl PartialEq for AnnotationTypeMapping {
    fn eq(&self, other: &Self) -> bool {
        self.annotation_type == other.annotation_type
            && self.attributes == other.attributes
            && self.source == other.source
            && self.distance == other.distance
    }
}

impl Eq for AnnotationTypeMapping {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_mapping() {
        let mut mapping = AnnotationTypeMapping::new("Service");
        mapping.set_attribute("value", "myService");
        assert_eq!(mapping.annotation_type(), "Service");
        assert_eq!(mapping.get_attribute("value"), Some("myService"));
        assert!(mapping.is_direct());
        assert!(!mapping.is_meta());
    }

    #[test]
    fn test_meta_mapping() {
        // `with_source(annotation_type, source, distance)`:
        // 当前映射描述的注解类型是 `Component`，它通过 `Service`（元注解链）发现。
        let mut mapping = AnnotationTypeMapping::with_source("Component", "Service", 1);
        mapping.set_attribute("value", "");
        assert_eq!(mapping.annotation_type(), "Component");
        assert_eq!(mapping.source(), Some("Service"));
        assert_eq!(mapping.distance(), 1);
        assert!(mapping.is_meta());
    }
}
