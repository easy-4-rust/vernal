//! AbstractBeanDefinition — Spring 风格的 Bean 定义抽象基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AbstractBeanDefinition`。
//!
//! 在 Spring 中，`AbstractBeanDefinition` 是所有 Bean 定义的公共基类，
//! `RootBeanDefinition` 和 `ChildBeanDefinition` 都继承自它。
//! 提供了 BeanDefinition trait 的默认实现和通用字段访问器。
//!
//! ## 主要字段
//!
//! - `bean_class_name` — Bean 类名
//! - `scope` — 作用域（singleton/prototype）
//! - `lazy_init` — 是否延迟初始化
//! - `autowire_mode` — 自动装配模式
//! - `init_method_name` / `destroy_method_name` — 生命周期方法

use crate::component_scope::Scope;
use crate::factory::config::bean_definition::BeanDefinition;

use crate::component_key::ComponentKey;

/// Bean 定义抽象基类。
///
/// 对应 Spring 的 `AbstractBeanDefinition`。
///
/// 提供 BeanDefinition trait 的默认实现和通用字段访问器。
#[derive(Clone, Debug, Default)]
pub struct AbstractBeanDefinition {
    /// Bean 类名
    pub bean_class_name: Option<String>,
    /// 父 Bean 定义名称
    pub parent_name: Option<String>,
    /// 作用域
    pub scope: Scope,
    /// 是否延迟初始化
    pub lazy_init: bool,
    /// 是否 primary
    pub primary: bool,
    /// 是否为自动装配候选
    pub autowire_candidate: bool,
    /// Bean 角色（0=APPLICATION, 1=SUPPORT, 2=INFRASTRUCTURE）
    pub role: i32,
    /// 描述信息
    pub description: Option<String>,
    /// 初始化方法名
    pub init_method_name: Option<String>,
    /// 销毁方法名
    pub destroy_method_name: Option<String>,
    /// 工厂 Bean 名称
    pub factory_bean_name: Option<String>,
    /// 工厂方法名称
    pub factory_method_name: Option<String>,
    /// 是否 abstract
    pub is_abstract: bool,
    /// 构造器参数是否已解析
    pub constructor_args_resolved: bool,
}

impl AbstractBeanDefinition {
    /// 创建新的抽象 Bean 定义。
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取 Bean 类名。
    pub fn bean_class_name(&self) -> Option<&str> {
        self.bean_class_name.as_deref()
    }

    /// 设置 Bean 类名。
    pub fn set_bean_class_name(&mut self, name: impl Into<String>) {
        self.bean_class_name = Some(name.into());
    }

    /// 获取父 Bean 定义名称。
    pub fn parent_name(&self) -> Option<&str> {
        self.parent_name.as_deref()
    }

    /// 设置父 Bean 定义名称。
    pub fn set_parent_name(&mut self, name: impl Into<String>) {
        self.parent_name = Some(name.into());
    }

    /// 获取作用域。
    pub fn scope(&self) -> Scope {
        self.scope
    }

    /// 设置作用域。
    pub fn set_scope(&mut self, scope: Scope) {
        self.scope = scope;
    }

    /// 设置延迟初始化。
    pub fn set_lazy_init(&mut self, lazy: bool) {
        self.lazy_init = lazy;
    }

    /// 是否延迟初始化。
    pub fn is_lazy_init(&self) -> bool {
        self.lazy_init
    }

    /// 设置 primary。
    pub fn set_primary(&mut self, primary: bool) {
        self.primary = primary;
    }

    /// 是否 primary。
    pub fn is_primary(&self) -> bool {
        self.primary
    }

    /// 设置自动装配候选。
    pub fn set_autowire_candidate(&mut self, candidate: bool) {
        self.autowire_candidate = candidate;
    }

    /// 是否为自动装配候选。
    pub fn is_autowire_candidate(&self) -> bool {
        self.autowire_candidate
    }

    /// 设置角色。
    pub fn set_role(&mut self, role: i32) {
        self.role = role;
    }

    /// 设置描述。
    pub fn set_description(&mut self, desc: impl Into<String>) {
        self.description = Some(desc.into());
    }

    /// 设置初始化方法名。
    pub fn set_init_method_name(&mut self, name: impl Into<String>) {
        self.init_method_name = Some(name.into());
    }

    /// 设置销毁方法名。
    pub fn set_destroy_method_name(&mut self, name: impl Into<String>) {
        self.destroy_method_name = Some(name.into());
    }

    /// 设置工厂 Bean 名称。
    pub fn set_factory_bean_name(&mut self, name: impl Into<String>) {
        self.factory_bean_name = Some(name.into());
    }

    /// 设置工厂方法名称。
    pub fn set_factory_method_name(&mut self, name: impl Into<String>) {
        self.factory_method_name = Some(name.into());
    }

    /// 设置是否 abstract。
    pub fn set_abstract(&mut self, is_abstract: bool) {
        self.is_abstract = is_abstract;
    }

    /// 是否 abstract。
    pub fn is_abstract(&self) -> bool {
        self.is_abstract
    }

    /// 是否为单例。
    pub fn is_singleton(&self) -> bool {
        matches!(self.scope, Scope::Singleton)
    }

    /// 是否为原型。
    pub fn is_prototype(&self) -> bool {
        matches!(self.scope, Scope::Transient)
    }
}

impl BeanDefinition for AbstractBeanDefinition {
    fn bean_name(&self) -> &ComponentKey {
        unimplemented!("AbstractBeanDefinition does not hold ComponentKey")
    }

    fn bean_class_name(&self) -> &str {
        self.bean_class_name.as_deref().unwrap_or("unknown")
    }

    fn scope(&self) -> Scope {
        self.scope
    }

    fn is_lazy_init(&self) -> bool {
        self.lazy_init
    }

    fn is_primary(&self) -> bool {
        self.primary
    }

    fn is_autowire_candidate(&self) -> bool {
        self.autowire_candidate
    }

    fn role(&self) -> i32 {
        self.role
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn parent_name(&self) -> Option<&str> {
        self.parent_name.as_deref()
    }

    fn factory_bean_name(&self) -> Option<&str> {
        self.factory_bean_name.as_deref()
    }

    fn factory_method_name(&self) -> Option<&str> {
        self.factory_method_name.as_deref()
    }

    fn init_method_name(&self) -> Option<&str> {
        self.init_method_name.as_deref()
    }

    fn destroy_method_name(&self) -> Option<&str> {
        self.destroy_method_name.as_deref()
    }

    fn is_abstract(&self) -> bool {
        self.is_abstract
    }

    fn is_singleton(&self) -> bool {
        self.is_singleton()
    }

    fn is_prototype(&self) -> bool {
        self.is_prototype()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_definition_has_defaults() {
        let def = AbstractBeanDefinition::new();
        assert_eq!(def.bean_class_name(), None);
        assert_eq!(def.scope(), Scope::Singleton);
        assert!(!def.is_lazy_init());
        assert!(!def.is_primary());
        assert!(!def.is_autowire_candidate());
        assert!(!def.is_abstract());
    }

    #[test]
    fn set_and_get_properties() {
        let mut def = AbstractBeanDefinition::new();
        def.set_bean_class_name("com.example.MyService");
        def.set_scope(Scope::Transient);
        def.set_lazy_init(true);
        def.set_primary(true);
        def.set_description("A test service");

        assert_eq!(def.bean_class_name(), Some("com.example.MyService"));
        assert_eq!(def.scope(), Scope::Transient);
        assert!(def.is_lazy_init());
        assert!(def.is_primary());
        assert_eq!(def.description(), Some("A test service"));
    }

    #[test]
    fn lifecycle_methods() {
        let mut def = AbstractBeanDefinition::new();
        def.set_init_method_name("init");
        def.set_destroy_method_name("cleanup");

        assert_eq!(def.init_method_name(), Some("init"));
        assert_eq!(def.destroy_method_name(), Some("cleanup"));
    }

    #[test]
    fn factory_methods() {
        let mut def = AbstractBeanDefinition::new();
        def.set_factory_bean_name("factoryBean");
        def.set_factory_method_name("createInstance");

        assert_eq!(def.factory_bean_name(), Some("factoryBean"));
        assert_eq!(def.factory_method_name(), Some("createInstance"));
    }

    #[test]
    fn scope_checks() {
        let mut def = AbstractBeanDefinition::new();

        def.set_scope(Scope::Singleton);
        assert!(def.is_singleton());
        assert!(!def.is_prototype());

        def.set_scope(Scope::Transient);
        assert!(!def.is_singleton());
        assert!(def.is_prototype());
    }

    #[test]
    fn parent_name() {
        let mut def = AbstractBeanDefinition::new();
        assert!(def.parent_name().is_none());

        def.set_parent_name("parentBean");
        assert_eq!(def.parent_name(), Some("parentBean"));
    }
}
