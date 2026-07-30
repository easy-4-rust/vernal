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
    pub fn new(type_id: TypeId, type_name: String, required: bool) -> Self {
        Self {
            type_id,
            type_name,
            required,
            qualifier: None,
            injection_point_name: String::new(),
        }
    }

    pub fn with_qualifier(mut self, qualifier: String) -> Self {
        self.qualifier = Some(qualifier);
        self
    }

    pub fn with_injection_point_name(mut self, name: String) -> Self {
        self.injection_point_name = name;
        self
    }

    pub fn is_required(&self) -> bool {
        self.required
    }

    pub fn has_qualifier(&self) -> bool {
        self.qualifier.is_some()
    }
}
