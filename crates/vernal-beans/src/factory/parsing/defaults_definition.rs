//! DefaultsDefinition — 对应 Spring beans.factory.parsing.DefaultsDefinition。
//!
//! 默认值定义接口。表示一个提供默认配置值的定义元素。
//! 在 XML 配置中，`<beans>` 根元素可以定义默认的 lazy-init、
//! autowire、scope 等属性，作为其内部所有 `<bean>` 元素的默认值。
//!
//! 对应 Java 接口：`org.springframework.beans.factory.parsing.DefaultsDefinition`。

use std::fmt;

/// 默认值定义接口。
///
/// 对应 Spring 的 `DefaultsDefinition`。
///
/// 在 Spring 的 `<beans>` 根元素中，可以设置全局默认属性：
///
/// ```xml
/// <beans default-lazy-init="true"
///        default-autowire="byType"
///        default-scope="singleton">
///     <bean id="svc" class="MyService"/>
/// </beans>
/// ```
///
/// 在 Rust 版本中，`DefaultsDefinition` trait 和
/// [`BeanDefaults`] 结构体共同提供此功能。
pub trait DefaultsDefinition: Send + Sync + fmt::Debug {
    /// 返回默认的 lazy-init 值。
    fn default_lazy_init(&self) -> Option<bool> {
        None
    }

    /// 返回默认的 autowire 模式。
    fn default_autowire_mode(&self) -> Option<i32> {
        None
    }

    /// 返回默认的作用域。
    fn default_scope(&self) -> Option<&str> {
        None
    }

    /// 返回默认的 init 方法名称。
    fn default_init_method(&self) -> Option<&str> {
        None
    }

    /// 返回默认的 destroy 方法名称。
    fn default_destroy_method(&self) -> Option<&str> {
        None
    }

    /// 返回默认的 merge 模式。
    fn default_merge(&self) -> Option<bool> {
        None
    }
}

/// Bean 默认值的具体实现。
///
/// 对应 Spring 的 `<beans>` 元素中的默认属性。
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::defaults_definition::{BeanDefaults, DefaultsDefinition};
///
/// let defaults = BeanDefaults::new()
///     .with_lazy_init(true)
///     .with_default_scope("singleton");
///
/// assert_eq!(defaults.default_lazy_init(), Some(true));
/// assert_eq!(defaults.default_scope(), Some("singleton"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct BeanDefaults {
    /// 默认的 lazy-init 值。
    lazy_init: Option<bool>,
    /// 默认的 autowire 模式（0=NO, 1=BY_NAME, 2=BY_TYPE, 3=CONSTRUCTOR）。
    autowire_mode: Option<i32>,
    /// 默认作用域。
    scope: Option<String>,
    /// 默认的 init 方法。
    init_method: Option<String>,
    /// 默认的 destroy 方法。
    destroy_method: Option<String>,
    /// 默认的 merge 模式。
    default_merge: Option<bool>,
}

impl BeanDefaults {
    /// 创建一个新的空默认值定义。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置默认 lazy-init 值（链式构建）。
    #[must_use]
    pub fn with_lazy_init(mut self, lazy: bool) -> Self {
        self.lazy_init = Some(lazy);
        self
    }

    /// 设置默认 autowire 模式（链式构建）。
    #[must_use]
    pub fn with_autowire_mode(mut self, mode: i32) -> Self {
        self.autowire_mode = Some(mode);
        self
    }

    /// 设置默认作用域（链式构建）。
    #[must_use]
    pub fn with_default_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// 设置默认 init 方法（链式构建）。
    #[must_use]
    pub fn with_init_method(mut self, method: impl Into<String>) -> Self {
        self.init_method = Some(method.into());
        self
    }

    /// 设置默认 destroy 方法（链式构建）。
    #[must_use]
    pub fn with_destroy_method(mut self, method: impl Into<String>) -> Self {
        self.destroy_method = Some(method.into());
        self
    }

    /// 设置默认 merge 模式（链式构建）。
    #[must_use]
    pub fn with_default_merge(mut self, merge: bool) -> Self {
        self.default_merge = Some(merge);
        self
    }
}

impl DefaultsDefinition for BeanDefaults {
    fn default_lazy_init(&self) -> Option<bool> {
        self.lazy_init
    }

    fn default_autowire_mode(&self) -> Option<i32> {
        self.autowire_mode
    }

    fn default_scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }

    fn default_init_method(&self) -> Option<&str> {
        self.init_method.as_deref()
    }

    fn default_destroy_method(&self) -> Option<&str> {
        self.destroy_method.as_deref()
    }

    fn default_merge(&self) -> Option<bool> {
        self.default_merge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_defaults() {
        let defaults = BeanDefaults::new();
        assert_eq!(defaults.default_lazy_init(), None);
        assert_eq!(defaults.default_scope(), None);
        assert_eq!(defaults.default_init_method(), None);
    }

    #[test]
    fn test_chain_build() {
        let defaults = BeanDefaults::new()
            .with_lazy_init(true)
            .with_autowire_mode(2)
            .with_default_scope("singleton")
            .with_init_method("init")
            .with_destroy_method("cleanup")
            .with_default_merge(false);

        assert_eq!(defaults.default_lazy_init(), Some(true));
        assert_eq!(defaults.default_autowire_mode(), Some(2));
        assert_eq!(defaults.default_scope(), Some("singleton"));
        assert_eq!(defaults.default_init_method(), Some("init"));
        assert_eq!(defaults.default_destroy_method(), Some("cleanup"));
        assert_eq!(defaults.default_merge(), Some(false));
    }

    #[test]
    fn test_defaults_as_trait() {
        let defaults = BeanDefaults::new().with_lazy_init(false);
        // 通过 trait 方法调用
        let lazy: &dyn DefaultsDefinition = &defaults;
        assert_eq!(lazy.default_lazy_init(), Some(false));
        assert_eq!(lazy.default_autowire_mode(), None);
    }

    #[test]
    fn test_individual_setters() {
        let defaults = BeanDefaults::new().with_lazy_init(true);
        assert_eq!(defaults.default_lazy_init(), Some(true));
        assert_eq!(defaults.default_autowire_mode(), None);
        assert_eq!(defaults.default_scope(), None);
        assert_eq!(defaults.default_init_method(), None);
        assert_eq!(defaults.default_destroy_method(), None);
        assert_eq!(defaults.default_merge(), None);
    }

    #[test]
    fn test_autowire_mode_only() {
        let defaults = BeanDefaults::new().with_autowire_mode(1);
        assert_eq!(defaults.default_autowire_mode(), Some(1));
        assert_eq!(defaults.default_lazy_init(), None);
    }

    #[test]
    fn test_scope_only() {
        let defaults = BeanDefaults::new().with_default_scope("prototype");
        assert_eq!(defaults.default_scope(), Some("prototype"));
    }

    #[test]
    fn test_init_method_only() {
        let defaults = BeanDefaults::new().with_init_method("onInit");
        assert_eq!(defaults.default_init_method(), Some("onInit"));
    }

    #[test]
    fn test_destroy_method_only() {
        let defaults = BeanDefaults::new().with_destroy_method("onDestroy");
        assert_eq!(defaults.default_destroy_method(), Some("onDestroy"));
    }

    #[test]
    fn test_merge_only() {
        let defaults = BeanDefaults::new().with_default_merge(true);
        assert_eq!(defaults.default_merge(), Some(true));
    }

    #[test]
    fn test_clone_trait() {
        let defaults = BeanDefaults::new()
            .with_lazy_init(true)
            .with_default_scope("singleton");
        let cloned = defaults.clone();
        assert_eq!(cloned.default_lazy_init(), Some(true));
        assert_eq!(cloned.default_scope(), Some("singleton"));
    }

    #[test]
    fn test_debug_trait() {
        let defaults = BeanDefaults::new().with_lazy_init(true);
        let debug_str = format!("{:?}", defaults);
        assert!(debug_str.contains("BeanDefaults"));
    }

    #[test]
    fn test_chain_all_setters() {
        let defaults = BeanDefaults::new()
            .with_lazy_init(false)
            .with_autowire_mode(3)
            .with_default_scope("request")
            .with_init_method("init")
            .with_destroy_method("destroy")
            .with_default_merge(true);

        assert_eq!(defaults.default_lazy_init(), Some(false));
        assert_eq!(defaults.default_autowire_mode(), Some(3));
        assert_eq!(defaults.default_scope(), Some("request"));
        assert_eq!(defaults.default_init_method(), Some("init"));
        assert_eq!(defaults.default_destroy_method(), Some("destroy"));
        assert_eq!(defaults.default_merge(), Some(true));
    }

    #[test]
    fn test_trait_default_methods() {
        #[derive(Debug)]
        struct NoDefaults;
        impl DefaultsDefinition for NoDefaults {}

        let d = NoDefaults;
        assert_eq!(d.default_lazy_init(), None);
        assert_eq!(d.default_autowire_mode(), None);
        assert_eq!(d.default_scope(), None);
        assert_eq!(d.default_init_method(), None);
        assert_eq!(d.default_destroy_method(), None);
        assert_eq!(d.default_merge(), None);
    }

    #[test]
    fn test_scope_with_string() {
        let defaults = BeanDefaults::new().with_default_scope(String::from("session"));
        assert_eq!(defaults.default_scope(), Some("session"));
    }

    #[test]
    fn test_init_method_with_string() {
        let defaults = BeanDefaults::new().with_init_method(String::from("setup"));
        assert_eq!(defaults.default_init_method(), Some("setup"));
    }

    #[test]
    fn test_destroy_method_with_string() {
        let defaults = BeanDefaults::new().with_destroy_method(String::from("teardown"));
        assert_eq!(defaults.default_destroy_method(), Some("teardown"));
    }
}
