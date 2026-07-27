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
    pub fn for_method_parameter(
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
