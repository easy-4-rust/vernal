//! AnnotationMetadata — Spring 风格的注解元数据 trait。
//!
//! 对应 Java 类：`org.springframework.core.type.AnnotationMetadata`。
//!
//! 描述一个类上的注解信息，包括注解类型、是否作用于类/方法/字段级别等。

use std::collections::HashMap;

/// 注解级别。
///
/// 标识一个注解作用于程序的哪个位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnnotationLevel {
    /// 类级别注解。
    Class,
    /// 字段级别注解。
    Field,
    /// 方法级别注解。
    Method,
}

/// 一个被采集到的注解描述。
///
/// 包含注解类型名、可选的目标方法名以及注解属性。
#[derive(Debug, Clone)]
pub struct AnnotationDescriptor {
    /// 注解类型名（如 `Component`、`Bean`）。
    annotation_type: String,
    /// 如果是方法/字段级别注解，关联的方法或字段名。
    member_name: Option<String>,
    /// 注解级别。
    level: AnnotationLevel,
    /// 注解属性键值对。
    attributes: HashMap<String, String>,
}

impl AnnotationDescriptor {
    /// 创建新的类级别注解描述。
    pub fn new_class(annotation_type: impl Into<String>) -> Self {
        Self {
            annotation_type: annotation_type.into(),
            member_name: None,
            level: AnnotationLevel::Class,
            attributes: HashMap::new(),
        }
    }

    /// 创建新的方法级别注解描述。
    pub fn new_method(annotation_type: impl Into<String>, method_name: impl Into<String>) -> Self {
        Self {
            annotation_type: annotation_type.into(),
            member_name: Some(method_name.into()),
            level: AnnotationLevel::Method,
            attributes: HashMap::new(),
        }
    }

    /// 创建新的字段级别注解描述。
    pub fn new_field(annotation_type: impl Into<String>, field_name: impl Into<String>) -> Self {
        Self {
            annotation_type: annotation_type.into(),
            member_name: Some(field_name.into()),
            level: AnnotationLevel::Field,
            attributes: HashMap::new(),
        }
    }

    /// 获取注解类型名。
    pub fn annotation_type(&self) -> &str {
        &self.annotation_type
    }

    /// 获取关联的方法或字段名。
    pub fn member_name(&self) -> Option<&str> {
        self.member_name.as_deref()
    }

    /// 获取注解级别。
    pub fn level(&self) -> AnnotationLevel {
        self.level
    }

    /// 获取注解方法名（与 `member_name` 等价的语义化别名）。
    pub fn method_name(&self) -> Option<&str> {
        if self.level == AnnotationLevel::Method {
            self.member_name.as_deref()
        } else {
            None
        }
    }

    /// 获取注解属性。
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    /// 获取可变注解属性。
    pub fn attributes_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.attributes
    }

    /// 是否为类级别注解。
    pub fn is_class_level(&self) -> bool {
        self.level == AnnotationLevel::Class
    }

    /// 是否为字段级别注解。
    pub fn is_field_level(&self) -> bool {
        self.level == AnnotationLevel::Field
    }

    /// 是否为方法级别注解。
    pub fn is_method_level(&self) -> bool {
        self.level == AnnotationLevel::Method
    }

    /// 插入一个属性。
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }
}

/// Spring 风格的注解元数据 trait。
///
/// 对应 Spring 的 `AnnotationMetadata`。
///
/// 描述一个被注解的类型（通常是类）所携带的注解信息。
pub trait AnnotationMetadata: Send + Sync {
    /// 获取该类型上所有注解描述。
    fn annotations(&self) -> &[AnnotationDescriptor];

    /// 获取指定类型的注解（首个匹配）。
    fn get_annotation(&self, annotation_type: &str) -> Option<&AnnotationDescriptor> {
        self.annotations()
            .iter()
            .find(|a| a.annotation_type() == annotation_type)
    }

    /// 获取注解类型名列表。
    ///
    /// 对应 Spring 的 `getAnnotationTypes()`。
    fn get_annotation_types(&self) -> Vec<String> {
        self.annotations()
            .iter()
            .map(|a| a.annotation_type().to_owned())
            .collect()
    }

    /// 是否存在指定类型的注解。
    ///
    /// 对应 Spring 的 `hasAnnotation(String annotationName)`。
    fn has_annotation(&self, annotation_type: &str) -> bool {
        self.annotations()
            .iter()
            .any(|a| a.annotation_type() == annotation_type)
    }

    /// 是否存在元注解（此处简化为与 `has_annotation` 一致）。
    ///
    /// 对应 Spring 的 `hasMetaAnnotation(String annotationName)`。
    fn has_meta_annotation(&self, annotation_type: &str) -> bool {
        self.has_annotation(annotation_type)
    }

    /// 是否存在指定类型且作用于类级别的注解。
    fn has_class_annotation(&self, annotation_type: &str) -> bool {
        self.annotations()
            .iter()
            .any(|a| a.is_class_level() && a.annotation_type() == annotation_type)
    }

    /// 是否存在指定类型且作用于方法级别的注解。
    fn has_method_annotation(&self, annotation_type: &str) -> bool {
        self.annotations()
            .iter()
            .any(|a| a.is_method_level() && a.annotation_type() == annotation_type)
    }

    /// 获取所有方法级别的注解。
    fn get_method_annotations(&self) -> Vec<&AnnotationDescriptor> {
        self.annotations()
            .iter()
            .filter(|a| a.is_method_level())
            .collect()
    }

    /// 获取第一个类级别注解的类型名。
    ///
    /// 对应 Spring 的 `getClassName()` 周边语义，作为便捷访问器。
    fn get_annotation_type(&self) -> Option<&str> {
        self.annotations()
            .iter()
            .find(|a| a.is_class_level())
            .map(|a| a.annotation_type())
    }

    /// 获取首个方法级别注解的方法名。
    fn get_method_name(&self) -> Option<&str> {
        self.annotations()
            .iter()
            .find(|a| a.is_method_level())
            .and_then(|a| a.method_name())
    }

    /// 是否包含任意类级别注解。
    fn is_class_level(&self) -> bool {
        self.annotations().iter().any(|a| a.is_class_level())
    }

    /// 是否包含任意字段级别注解。
    fn is_field_level(&self) -> bool {
        self.annotations().iter().any(|a| a.is_field_level())
    }

    /// 是否包含任意方法级别注解。
    fn is_method_level(&self) -> bool {
        self.annotations().iter().any(|a| a.is_method_level())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SampleMetadata {
        annotations: Vec<AnnotationDescriptor>,
    }

    impl AnnotationMetadata for SampleMetadata {
        fn annotations(&self) -> &[AnnotationDescriptor] {
            &self.annotations
        }
    }

    #[test]
    fn test_descriptor_levels() {
        let class = AnnotationDescriptor::new_class("Component");
        assert!(class.is_class_level());
        assert!(!class.is_method_level());

        let method = AnnotationDescriptor::new_method("Bean", "createThing");
        assert!(method.is_method_level());
        assert_eq!(method.method_name(), Some("createThing"));
    }

    #[test]
    fn test_metadata_queries() {
        let meta = SampleMetadata {
            annotations: vec![
                AnnotationDescriptor::new_class("Configuration"),
                AnnotationDescriptor::new_method("Bean", "createThing"),
            ],
        };
        assert!(meta.has_annotation("Configuration"));
        assert!(meta.has_method_annotation("Bean"));
        assert!(meta.is_method_level());
        assert_eq!(meta.get_method_name(), Some("createThing"));
    }
}
