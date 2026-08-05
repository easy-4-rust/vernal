//! DependencyDescriptor — Spring 风格的依赖描述符。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.DependencyDescriptor`。
//!
//! 描述单个依赖注入点的信息。

use std::any::TypeId;

/// 依赖描述符。
///
/// 对应 Spring 的 `DependencyDescriptor`。
///
/// 描述单个依赖注入点的信息：
/// - 依赖类型
/// - 是否必需
/// - 限定符
/// - 注入点名称
#[derive(Debug, Clone)]
pub struct DependencyDescriptor {
    /// 依赖的类型 ID
    pub type_id: TypeId,
    /// 依赖的类型名称
    pub type_name: String,
    /// 是否必需
    pub required: bool,
    /// 限定符（可选）
    pub qualifier: Option<String>,
    /// 注入点名称（字段名或方法名）
    pub injection_point_name: String,
}

impl DependencyDescriptor {
    /// 创建一个新的实例。
    pub fn new(type_id: TypeId, type_name: String, required: bool) -> Self {
        Self {
            type_id,
            type_name,
            required,
            qualifier: None,
            injection_point_name: String::new(),
        }
    }

    /// 执行with_qualifier操作。
    pub fn with_qualifier(mut self, qualifier: String) -> Self {
        self.qualifier = Some(qualifier);
        self
    }

    /// 执行with_injection_point_name操作。
    pub fn with_injection_point_name(mut self, name: String) -> Self {
        self.injection_point_name = name;
        self
    }

    /// 判断是否必需的。
    pub fn is_required(&self) -> bool {
        self.required
    }

    /// 判断是否限定符。
    pub fn has_qualifier(&self) -> bool {
        self.qualifier.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    #[test]
    fn new_basic() {
        let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true);
        assert_eq!(desc.type_id, TypeId::of::<String>());
        assert_eq!(desc.type_name, "String");
        assert!(desc.required);
        assert!(desc.qualifier.is_none());
        assert!(desc.injection_point_name.is_empty());
    }

    #[test]
    fn new_optional() {
        let desc = DependencyDescriptor::new(TypeId::of::<i32>(), "i32".to_string(), false);
        assert!(!desc.required);
    }

    #[test]
    fn with_qualifier() {
        let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true)
            .with_qualifier("primary".to_string());
        assert!(desc.has_qualifier());
        assert_eq!(desc.qualifier, Some("primary".to_string()));
    }

    #[test]
    fn with_injection_point_name() {
        let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true)
            .with_injection_point_name("myField".to_string());
        assert_eq!(desc.injection_point_name, "myField");
    }

    #[test]
    fn is_required_true() {
        let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true);
        assert!(desc.is_required());
    }

    #[test]
    fn is_required_false() {
        let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), false);
        assert!(!desc.is_required());
    }

    #[test]
    fn has_qualifier_false_by_default() {
        let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true);
        assert!(!desc.has_qualifier());
    }

    #[test]
    fn has_qualifier_true() {
        let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true)
            .with_qualifier("q".to_string());
        assert!(desc.has_qualifier());
    }

    #[test]
    fn builder_chaining() {
        let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true)
            .with_qualifier("primary".to_string())
            .with_injection_point_name("myField".to_string());
        assert!(desc.is_required());
        assert!(desc.has_qualifier());
        assert_eq!(desc.qualifier, Some("primary".to_string()));
        assert_eq!(desc.injection_point_name, "myField");
    }

    #[test]
    fn debug_format() {
        let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true);
        let debug = format!("{:?}", desc);
        assert!(debug.contains("DependencyDescriptor"));
    }

    #[test]
    fn clone() {
        let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true)
            .with_qualifier("q".to_string());
        let cloned = desc.clone();
        assert_eq!(cloned.type_id, desc.type_id);
        assert_eq!(cloned.type_name, desc.type_name);
        assert_eq!(cloned.required, desc.required);
        assert_eq!(cloned.qualifier, desc.qualifier);
    }
}
