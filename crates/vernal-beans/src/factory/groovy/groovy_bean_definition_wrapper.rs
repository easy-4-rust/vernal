//! GroovyBeanDefinitionWrapper — 对应 Java 类：org.springframework.beans.factory.groovy.GroovyBeanDefinitionWrapper。
//!
//! 对应 Spring beans.factory.groovy 包。
//!
//! 在 Spring 中，`GroovyBeanDefinitionWrapper` 是一个 Bean 定义的包装器，
//! 用于在 Groovy DSL 中提供属性设置的便捷方法。它包装了底层的
//! `GenericBeanDefinition`，并提供了 Groovy 友好的 API。
//!
//! ## 使用场景
//!
//! - Groovy DSL 中的 Bean 定义包装
//! - 提供动态属性设置能力
//! - 简化 Bean 定义的创建过程

use std::collections::HashMap;

/// GroovyBeanDefinitionWrapper — Spring 风格的 Bean 定义包装器。
///
/// 对应 Java 类：`org.springframework.beans.factory.groovy.GroovyBeanDefinitionWrapper`。
///
/// 包装 Bean 定义，提供 Groovy 友好的属性设置 API。
/// 支持动态属性、构造器参数和生命周期回调。
///
/// ## Java 对比
///
/// | Java | Rust |
/// |------|------|
/// | `GroovyBeanDefinitionWrapper` | `GroovyBeanDefinitionWrapper` |
/// | `setProperty(name, value)` | `set_property(name, value)` |
/// | `setBeanClass(Class)` | `set_bean_class(name)` |
/// | `setScope(String)` | `set_scope(name)` |
#[derive(Debug, Clone)]
pub struct GroovyBeanDefinitionWrapper {
    /// Bean 名称
    bean_name: String,
    /// Bean 类名
    bean_class: String,
    /// 作用域
    scope: String,
    /// 属性列表
    properties: HashMap<String, String>,
    /// 构造器参数
    constructor_args: Vec<String>,
    /// 初始化方法
    init_method: Option<String>,
    /// 销毁方法
    destroy_method: Option<String>,
    /// 是否为抽象 Bean
    abstract_flag: bool,
    /// 父 Bean 名称
    parent: Option<String>,
    /// 是否延迟初始化
    lazy_init: bool,
    /// 自动装配模式
    autowire_mode: String,
    /// 依赖列表
    depends_on: Vec<String>,
}

impl GroovyBeanDefinitionWrapper {
    /// 创建新的 GroovyBeanDefinitionWrapper。
    ///
    /// # 参数
    /// - `bean_name` — Bean 名称
    /// - `bean_class` — Bean 类名
    pub fn new(bean_name: impl Into<String>, bean_class: impl Into<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
            bean_class: bean_class.into(),
            scope: "singleton".to_string(),
            properties: HashMap::new(),
            constructor_args: Vec::new(),
            init_method: None,
            destroy_method: None,
            abstract_flag: false,
            parent: None,
            lazy_init: false,
            autowire_mode: "no".to_string(),
            depends_on: Vec::new(),
        }
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 获取 Bean 类名。
    pub fn bean_class(&self) -> &str {
        &self.bean_class
    }

    /// 设置 Bean 类名。
    pub fn set_bean_class(&mut self, class_name: impl Into<String>) {
        self.bean_class = class_name.into();
    }

    /// 获取作用域。
    pub fn scope(&self) -> &str {
        &self.scope
    }

    /// 设置作用域。
    ///
    /// 对应 Groovy DSL 的 `scope = 'prototype'`。
    pub fn set_scope(&mut self, scope: impl Into<String>) {
        self.scope = scope.into();
    }

    /// 设置属性值。
    ///
    /// 对应 Groovy DSL 的 `property = value`。
    pub fn set_property(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(name.into(), value.into());
    }

    /// 获取属性值。
    pub fn get_property(&self, name: &str) -> Option<&str> {
        self.properties.get(name).map(|s| s.as_str())
    }

    /// 获取所有属性名。
    pub fn property_names(&self) -> Vec<&str> {
        self.properties.keys().map(|s| s.as_str()).collect()
    }

    /// 获取属性数量。
    pub fn property_count(&self) -> usize {
        self.properties.len()
    }

    /// 添加构造器参数。
    pub fn add_constructor_arg(&mut self, arg: impl Into<String>) {
        self.constructor_args.push(arg.into());
    }

    /// 获取构造器参数。
    pub fn constructor_args(&self) -> &[String] {
        &self.constructor_args
    }

    /// 设置初始化方法。
    pub fn set_init_method(&mut self, method: impl Into<String>) {
        self.init_method = Some(method.into());
    }

    /// 获取初始化方法。
    pub fn init_method(&self) -> Option<&str> {
        self.init_method.as_deref()
    }

    /// 设置销毁方法。
    pub fn set_destroy_method(&mut self, method: impl Into<String>) {
        self.destroy_method = Some(method.into());
    }

    /// 获取销毁方法。
    pub fn destroy_method(&self) -> Option<&str> {
        self.destroy_method.as_deref()
    }

    /// 设置是否为抽象 Bean。
    pub fn set_abstract(&mut self, is_abstract: bool) {
        self.abstract_flag = is_abstract;
    }

    /// 是否为抽象 Bean。
    pub fn is_abstract(&self) -> bool {
        self.abstract_flag
    }

    /// 设置父 Bean。
    pub fn set_parent(&mut self, parent: impl Into<String>) {
        self.parent = Some(parent.into());
    }

    /// 获取父 Bean 名称。
    pub fn parent(&self) -> Option<&str> {
        self.parent.as_deref()
    }

    /// 设置延迟初始化。
    pub fn set_lazy_init(&mut self, lazy: bool) {
        self.lazy_init = lazy;
    }

    /// 是否延迟初始化。
    pub fn is_lazy_init(&self) -> bool {
        self.lazy_init
    }

    /// 设置自动装配模式。
    pub fn set_autowire_mode(&mut self, mode: impl Into<String>) {
        self.autowire_mode = mode.into();
    }

    /// 获取自动装配模式。
    pub fn autowire_mode(&self) -> &str {
        &self.autowire_mode
    }

    /// 添加依赖。
    pub fn add_depends_on(&mut self, bean_name: impl Into<String>) {
        self.depends_on.push(bean_name.into());
    }

    /// 获取依赖列表。
    pub fn depends_on(&self) -> &[String] {
        &self.depends_on
    }

    /// 检查是否为单例。
    pub fn is_singleton(&self) -> bool {
        self.scope == "singleton"
    }
}

impl Default for GroovyBeanDefinitionWrapper {
    fn default() -> Self {
        Self::new("unnamed", "java.lang.Object")
    }
}

impl std::fmt::Display for GroovyBeanDefinitionWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GroovyBeanDefinitionWrapper[name={}, class={}, scope={}]",
            self.bean_name, self.bean_class, self.scope
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_wrapper_stores_name_and_class() {
        let w = GroovyBeanDefinitionWrapper::new("myBean", "com.example.MyClass");
        assert_eq!(w.bean_name(), "myBean");
        assert_eq!(w.bean_class(), "com.example.MyClass");
    }

    #[test]
    fn default_scope_is_singleton() {
        let w = GroovyBeanDefinitionWrapper::new("b", "C");
        assert_eq!(w.scope(), "singleton");
        assert!(w.is_singleton());
    }

    #[test]
    fn set_scope() {
        let mut w = GroovyBeanDefinitionWrapper::new("b", "C");
        w.set_scope("prototype");
        assert_eq!(w.scope(), "prototype");
        assert!(!w.is_singleton());
    }

    #[test]
    fn set_and_get_property() {
        let mut w = GroovyBeanDefinitionWrapper::new("b", "C");
        w.set_property("name", "value");
        assert_eq!(w.get_property("name"), Some("value"));
        assert_eq!(w.property_count(), 1);
    }

    #[test]
    fn constructor_args() {
        let mut w = GroovyBeanDefinitionWrapper::new("b", "C");
        w.add_constructor_arg("arg1");
        w.add_constructor_arg("arg2");
        assert_eq!(w.constructor_args().len(), 2);
    }

    #[test]
    fn lifecycle_methods() {
        let mut w = GroovyBeanDefinitionWrapper::new("b", "C");
        w.set_init_method("init");
        w.set_destroy_method("destroy");
        assert_eq!(w.init_method(), Some("init"));
        assert_eq!(w.destroy_method(), Some("destroy"));
    }

    #[test]
    fn abstract_and_parent() {
        let mut w = GroovyBeanDefinitionWrapper::new("b", "C");
        w.set_abstract(true);
        w.set_parent("parentBean");
        assert!(w.is_abstract());
        assert_eq!(w.parent(), Some("parentBean"));
    }

    #[test]
    fn lazy_init() {
        let mut w = GroovyBeanDefinitionWrapper::new("b", "C");
        w.set_lazy_init(true);
        assert!(w.is_lazy_init());
    }

    #[test]
    fn autowire_mode() {
        let mut w = GroovyBeanDefinitionWrapper::new("b", "C");
        w.set_autowire_mode("byType");
        assert_eq!(w.autowire_mode(), "byType");
    }

    #[test]
    fn depends_on() {
        let mut w = GroovyBeanDefinitionWrapper::new("b", "C");
        w.add_depends_on("dataSource");
        w.add_depends_on("config");
        assert_eq!(w.depends_on().len(), 2);
    }

    #[test]
    fn display_format() {
        let w = GroovyBeanDefinitionWrapper::new("myBean", "MyClass");
        let s = format!("{}", w);
        assert!(s.contains("myBean"));
        assert!(s.contains("MyClass"));
    }

    #[test]
    fn set_bean_class() {
        let mut w = GroovyBeanDefinitionWrapper::new("b", "OldClass");
        w.set_bean_class("NewClass");
        assert_eq!(w.bean_class(), "NewClass");
    }

    #[test]
    fn property_names() {
        let mut w = GroovyBeanDefinitionWrapper::new("b", "C");
        w.set_property("a", "1");
        w.set_property("b", "2");
        let mut names = w.property_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }
}
