//! RegisteredBean — Spring 风格的已注册 Bean 句柄。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.RegisteredBean`。
//!
//! 在 Spring 6.x 中，`RegisteredBean` 提供了对已注册 Bean 定义的只读视图。
//! 它封装了 Bean 名称和 Bean 工厂的引用，提供了便捷的查询方法。
//! 主要用于 AOT 处理和 Bean 生命周期回调中。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`RegisteredBean` 作为轻量级句柄，
//! 存储 Bean 的元数据信息，不持有对 Bean 工厂的引用。

use std::fmt;

/// 已注册 Bean 句柄。
///
/// 对应 Spring 的 `RegisteredBean`。
///
/// 提供已注册 Bean 定义的只读视图。
/// 用于 Bean 生命周期回调和 AOT 处理场景。
#[derive(Clone, Debug)]
pub struct RegisteredBean {
    /// Bean 名称。
    bean_name: String,
    /// Bean 类型名。
    bean_type_name: Option<String>,
    /// Bean 作用域。
    scope: String,
    /// 是否为单例。
    is_singleton: bool,
    /// 是否为延迟初始化。
    is_lazy_init: bool,
}

impl RegisteredBean {
    /// 创建已注册 Bean 句柄。
    pub fn new(bean_name: impl Into<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
            bean_type_name: None,
            scope: "singleton".to_string(),
            is_singleton: true,
            is_lazy_init: false,
        }
    }

    /// 设置 Bean 类型名。
    pub fn with_type_name(mut self, type_name: impl Into<String>) -> Self {
        self.bean_type_name = Some(type_name.into());
        self
    }

    /// 设置作用域。
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        let scope = scope.into();
        self.is_singleton = scope == "singleton";
        self.scope = scope;
        self
    }

    /// 设置延迟初始化。
    pub fn with_lazy_init(mut self, lazy: bool) -> Self {
        self.is_lazy_init = lazy;
        self
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 获取 Bean 类型名。
    pub fn bean_type_name(&self) -> Option<&str> {
        self.bean_type_name.as_deref()
    }

    /// 获取作用域。
    pub fn scope(&self) -> &str {
        &self.scope
    }

    /// 是否为单例。
    pub fn is_singleton(&self) -> bool {
        self.is_singleton
    }

    /// 是否为延迟初始化。
    pub fn is_lazy_init(&self) -> bool {
        self.is_lazy_init
    }

    /// 获取 Bean 的限定描述（用于日志）。
    pub fn qualified_description(&self) -> String {
        match &self.bean_type_name {
            Some(ty) => format!("{} [{}]", self.bean_name, ty),
            None => self.bean_name.clone(),
        }
    }
}

impl fmt::Display for RegisteredBean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RegisteredBean[{}]", self.bean_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_bean_default_is_singleton() {
        let bean = RegisteredBean::new("myService");
        assert_eq!(bean.bean_name(), "myService");
        assert!(bean.is_singleton());
        assert!(!bean.is_lazy_init());
        assert_eq!(bean.scope(), "singleton");
    }

    #[test]
    fn registered_bean_with_prototype_scope() {
        let bean = RegisteredBean::new("proto").with_scope("prototype");
        assert!(!bean.is_singleton());
        assert_eq!(bean.scope(), "prototype");
    }

    #[test]
    fn qualified_description_with_type() {
        let bean = RegisteredBean::new("svc")
            .with_type_name("com.example.MyService");
        assert_eq!(bean.qualified_description(), "svc [com.example.MyService]");
    }

    #[test]
    fn qualified_description_without_type() {
        let bean = RegisteredBean::new("svc");
        assert_eq!(bean.qualified_description(), "svc");
    }

    #[test]
    fn display_format() {
        let bean = RegisteredBean::new("test");
        assert_eq!(format!("{}", bean), "RegisteredBean[test]");
    }
}
