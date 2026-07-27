//! InjectionPoint — Spring 风格的注入点信息。
//!
//! 对应 Java 类：`org.springframework.beans.factory.InjectionPoint`。
//!
//! 描述依赖注入的位置（字段、构造器参数、方法参数），用于限定符匹配和错误报告。

use std::any::TypeId;

/// Spring 风格的注入点信息。
///
/// 对应 Spring 的 `InjectionPoint`。
///
/// 描述依赖注入的位置，用于：
/// - `@Qualifier` 匹配
/// - 错误报告（"无法在字段 X 上注入"）
/// - AOP 切入点匹配
///
/// ## 与 Dependency 的区别
///
/// - `Dependency` 是类型级别的（描述需要什么类型）
/// - `InjectionPoint` 是位置级别的（描述在哪里注入）
#[derive(Clone, Debug)]
pub struct InjectionPoint {
    /// 注入的类型。
    type_id: TypeId,
    /// 类型名。
    type_name: &'static str,
    /// 所属 Bean 名称。
    containing_bean_name: Option<String>,
    /// 字段/参数名称。
    member_name: Option<String>,
    /// 限定符。
    qualifier: Option<String>,
}

impl InjectionPoint {
    /// 创建注入点。
    pub fn new(type_id: TypeId, type_name: &'static str) -> Self {
        Self {
            type_id,
            type_name,
            containing_bean_name: None,
            member_name: None,
            qualifier: None,
        }
    }

    /// 设置所属 Bean 名称。
    pub fn with_containing_bean_name(mut self, name: impl Into<String>) -> Self {
        self.containing_bean_name = Some(name.into());
        self
    }

    /// 设置成员名称。
    pub fn with_member_name(mut self, name: impl Into<String>) -> Self {
        self.member_name = Some(name.into());
        self
    }

    /// 设置限定符。
    pub fn with_qualifier(mut self, qualifier: impl Into<String>) -> Self {
        self.qualifier = Some(qualifier.into());
        self
    }

    /// 获取注入类型。
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// 获取类型名。
    pub fn type_name(&self) -> &str {
        self.type_name
    }

    /// 获取所属 Bean 名称。
    pub fn containing_bean_name(&self) -> Option<&str> {
        self.containing_bean_name.as_deref()
    }

    /// 获取成员名称。
    pub fn member_name(&self) -> Option<&str> {
        self.member_name.as_deref()
    }

    /// 获取限定符。
    pub fn qualifier(&self) -> Option<&str> {
        self.qualifier.as_deref()
    }
}
