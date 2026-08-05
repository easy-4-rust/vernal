//! DependencyDescriptor — Spring 风格的依赖描述符。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.DependencyDescriptor`。
//!
//! 描述一个依赖请求的完整上下文，包括位置、类型、限定符等。

use std::any::TypeId;

/// Spring 风格的依赖描述符。
///
/// 对应 Spring 的 `DependencyDescriptor`。
///
/// 描述一个依赖请求的完整上下文：
/// - 注入位置（字段、构造器参数、方法参数）
/// - 依赖类型
/// - 是否可选
/// - 是否为批量（List/Array）
/// - 限定符
///
/// ## 与 Dependency 的区别
///
/// - `Dependency` 是 vernal-beans 自有的简化版本（在 `ComponentDefinition` 中使用）
/// - `DependencyDescriptor` 是 Spring 风格的完整描述符（在 `resolveDependency` 中使用）
#[derive(Clone, Debug)]
pub struct DependencyDescriptor {
    /// 注入位置索引。
    field_index: Option<usize>,
    /// 依赖类型。
    type_id: TypeId,
    /// 类型名。
    type_name: &'static str,
    /// 是否可选。
    optional: bool,
    /// 是否为批量（List/Array）。
    multiple: bool,
    /// 限定符名称。
    qualifier: Option<String>,
    /// 所属 Bean 名称。
    containing_bean_name: Option<String>,
}

impl DependencyDescriptor {
    /// 创建字段依赖描述符。
    pub fn for_field(type_id: TypeId, type_name: &'static str) -> Self {
        Self {
            field_index: None,
            type_id,
            type_name,
            optional: false,
            multiple: false,
            qualifier: None,
            containing_bean_name: None,
        }
    }

    /// 创建构造器参数依赖描述符。
    pub fn for_constructor_parameter(
        index: usize,
        type_id: TypeId,
        type_name: &'static str,
    ) -> Self {
        Self {
            field_index: Some(index),
            type_id,
            type_name,
            optional: false,
            multiple: false,
            qualifier: None,
            containing_bean_name: None,
        }
    }

    /// 创建方法参数依赖描述符。
    pub fn for_method_parameter(index: usize, type_id: TypeId, type_name: &'static str) -> Self {
        Self {
            field_index: Some(index),
            type_id,
            type_name,
            optional: false,
            multiple: false,
            qualifier: None,
            containing_bean_name: None,
        }
    }

    /// 设置可选。
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

    /// 设置批量。
    pub fn with_multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// 设置限定符。
    pub fn with_qualifier(mut self, qualifier: impl Into<String>) -> Self {
        self.qualifier = Some(qualifier.into());
        self
    }

    /// 设置所属 Bean 名称。
    pub fn with_containing_bean_name(mut self, name: impl Into<String>) -> Self {
        self.containing_bean_name = Some(name.into());
        self
    }

    /// 获取注入位置索引。
    pub fn field_index(&self) -> Option<usize> {
        self.field_index
    }

    /// 获取依赖类型。
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// 获取类型名。
    pub fn type_name(&self) -> &str {
        self.type_name
    }

    /// 是否可选。
    pub fn is_optional(&self) -> bool {
        self.optional
    }

    /// 是否为批量。
    pub fn is_multiple(&self) -> bool {
        self.multiple
    }

    /// 获取限定符。
    pub fn qualifier(&self) -> Option<&str> {
        self.qualifier.as_deref()
    }

    /// 获取所属 Bean 名称。
    pub fn containing_bean_name(&self) -> Option<&str> {
        self.containing_bean_name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    // ── for_field ────────────────────────────────────────────────────────

    #[test]
    fn for_field_basic() {
        let desc = DependencyDescriptor::for_field(TypeId::of::<String>(), "String");
        assert_eq!(desc.type_id(), TypeId::of::<String>());
        assert_eq!(desc.type_name(), "String");
        assert!(!desc.is_optional());
        assert!(!desc.is_multiple());
        assert!(desc.field_index().is_none());
        assert!(desc.qualifier().is_none());
        assert!(desc.containing_bean_name().is_none());
    }

    // ── for_constructor_parameter ────────────────────────────────────────

    #[test]
    fn for_constructor_parameter_basic() {
        let desc = DependencyDescriptor::for_constructor_parameter(0, TypeId::of::<i32>(), "i32");
        assert_eq!(desc.type_id(), TypeId::of::<i32>());
        assert_eq!(desc.type_name(), "i32");
        assert_eq!(desc.field_index(), Some(0));
        assert!(!desc.is_optional());
        assert!(!desc.is_multiple());
        assert!(desc.qualifier().is_none());
        assert!(desc.containing_bean_name().is_none());
    }

    #[test]
    fn for_constructor_parameter_different_index() {
        let desc = DependencyDescriptor::for_constructor_parameter(2, TypeId::of::<f64>(), "f64");
        assert_eq!(desc.field_index(), Some(2));
        assert_eq!(desc.type_name(), "f64");
    }

    // ── for_method_parameter ─────────────────────────────────────────────

    #[test]
    fn for_method_parameter_basic() {
        let desc = DependencyDescriptor::for_method_parameter(1, TypeId::of::<bool>(), "bool");
        assert_eq!(desc.type_id(), TypeId::of::<bool>());
        assert_eq!(desc.type_name(), "bool");
        assert_eq!(desc.field_index(), Some(1));
        assert!(!desc.is_optional());
        assert!(!desc.is_multiple());
        assert!(desc.qualifier().is_none());
        assert!(desc.containing_bean_name().is_none());
    }

    // ── with_optional ────────────────────────────────────────────────────

    #[test]
    fn with_optional_true() {
        let desc =
            DependencyDescriptor::for_field(TypeId::of::<String>(), "String").with_optional(true);
        assert!(desc.is_optional());
    }

    #[test]
    fn with_optional_false() {
        let desc = DependencyDescriptor::for_field(TypeId::of::<String>(), "String")
            .with_optional(true)
            .with_optional(false);
        assert!(!desc.is_optional());
    }

    // ── with_multiple ────────────────────────────────────────────────────

    #[test]
    fn with_multiple_true() {
        let desc =
            DependencyDescriptor::for_field(TypeId::of::<String>(), "String").with_multiple(true);
        assert!(desc.is_multiple());
    }

    #[test]
    fn with_multiple_false() {
        let desc = DependencyDescriptor::for_field(TypeId::of::<String>(), "String")
            .with_multiple(true)
            .with_multiple(false);
        assert!(!desc.is_multiple());
    }

    // ── with_qualifier ───────────────────────────────────────────────────

    #[test]
    fn with_qualifier() {
        let desc = DependencyDescriptor::for_field(TypeId::of::<String>(), "String")
            .with_qualifier("primary");
        assert_eq!(desc.qualifier(), Some("primary"));
    }

    #[test]
    fn with_qualifier_string() {
        let desc = DependencyDescriptor::for_field(TypeId::of::<String>(), "String")
            .with_qualifier("myQualifier".to_string());
        assert_eq!(desc.qualifier(), Some("myQualifier"));
    }

    #[test]
    fn no_qualifier_by_default() {
        let desc = DependencyDescriptor::for_field(TypeId::of::<String>(), "String");
        assert!(desc.qualifier().is_none());
    }

    // ── with_containing_bean_name ────────────────────────────────────────

    #[test]
    fn with_containing_bean_name() {
        let desc = DependencyDescriptor::for_field(TypeId::of::<String>(), "String")
            .with_containing_bean_name("myBean");
        assert_eq!(desc.containing_bean_name(), Some("myBean"));
    }

    #[test]
    fn with_containing_bean_name_string() {
        let desc = DependencyDescriptor::for_field(TypeId::of::<String>(), "String")
            .with_containing_bean_name("myBean".to_string());
        assert_eq!(desc.containing_bean_name(), Some("myBean"));
    }

    #[test]
    fn no_containing_bean_name_by_default() {
        let desc = DependencyDescriptor::for_field(TypeId::of::<String>(), "String");
        assert!(desc.containing_bean_name().is_none());
    }

    // ── Builder chaining ─────────────────────────────────────────────────

    #[test]
    fn full_builder_chain() {
        let desc =
            DependencyDescriptor::for_constructor_parameter(0, TypeId::of::<String>(), "String")
                .with_optional(true)
                .with_multiple(true)
                .with_qualifier("primary")
                .with_containing_bean_name("myBean");
        assert_eq!(desc.type_id(), TypeId::of::<String>());
        assert_eq!(desc.type_name(), "String");
        assert_eq!(desc.field_index(), Some(0));
        assert!(desc.is_optional());
        assert!(desc.is_multiple());
        assert_eq!(desc.qualifier(), Some("primary"));
        assert_eq!(desc.containing_bean_name(), Some("myBean"));
    }

    // ── Clone ────────────────────────────────────────────────────────────

    #[test]
    fn clone_descriptor() {
        let desc = DependencyDescriptor::for_field(TypeId::of::<String>(), "String")
            .with_optional(true)
            .with_qualifier("q");
        let cloned = desc.clone();
        assert_eq!(cloned.type_id(), desc.type_id());
        assert_eq!(cloned.type_name(), desc.type_name());
        assert_eq!(cloned.is_optional(), desc.is_optional());
        assert_eq!(cloned.qualifier(), desc.qualifier());
    }

    // ── Debug ────────────────────────────────────────────────────────────

    #[test]
    fn debug_format() {
        let desc = DependencyDescriptor::for_field(TypeId::of::<String>(), "String");
        let debug = format!("{:?}", desc);
        assert!(debug.contains("DependencyDescriptor"));
    }
}
