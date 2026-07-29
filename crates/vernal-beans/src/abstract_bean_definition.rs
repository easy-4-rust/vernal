//! AbstractBeanDefinition — Spring 风格的 Bean 定义抽象基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AbstractBeanDefinition`。
//!
//! 提供 BeanDefinition trait 的共享实现：作用域、惰性初始化、primary、autowire 模式、
//! init/destroy 方法等。RootBeanDefinition 和 ChildBeanDefinition 继承此类。

use crate::autowire::Autowire;
use crate::bean_definition::BeanDefinition;
use crate::component_key::ComponentKey;
use crate::component_scope::Scope;
use crate::constructor_argument_values::ConstructorArgumentValues;
use crate::mutable_property_values::MutablePropertyValues;
use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// Spring 风格的 Bean 定义抽象基类。
///
/// 对应 Spring 的 `AbstractBeanDefinition`。
///
/// 提供 BeanDefinition 中共用字段和默认行为的实现。
/// RootBeanDefinition 和 ChildBeanDefinition 可以通过组合此结构体
/// 获得共享功能，而非强制继承。
#[derive(Clone, Debug)]
pub struct AbstractBeanDefinition {
    /// Bean 类名。
    bean_class_name: Option<String>,
    /// 父 Bean 定义名称。
    parent_name: Option<String>,
    /// 作用域。
    scope: Scope,
    /// 惰性初始化。
    lazy_init: bool,
    /// abstract 标志。
    abstract_flag: bool,
    /// autowire candidate。
    autowire_candidate: bool,
    /// primary 标志。
    primary: bool,
    /// fallback 标志。
    fallback: bool,
    /// synthetic 标志。
    synthetic: bool,
    /// 角色。
    role: i32,
    /// 描述。
    description: Option<String>,
    /// 依赖列表。
    depends_on: Vec<String>,
    /// autowire 模式。
    autowire_mode: Autowire,
    /// init 方法名。
    init_method_name: Option<String>,
    /// destroy 方法名。
    destroy_method_name: Option<String>,
    /// 工厂 Bean 名称。
    factory_bean_name: Option<String>,
    /// 工厂方法名。
    factory_method_name: Option<String>,
    /// 构造参数值。
    constructor_argument_values: ConstructorArgumentValues,
    /// 属性值。
    property_values: MutablePropertyValues,
}

impl AbstractBeanDefinition {
    /// 创建新的 AbstractBeanDefinition。
    pub fn new() -> Self {
        Self {
            bean_class_name: None,
            parent_name: None,
            scope: Scope::Singleton,
            lazy_init: false,
            abstract_flag: false,
            autowire_candidate: true,
            primary: false,
            fallback: false,
            synthetic: false,
            role: 0,
            description: None,
            depends_on: Vec::new(),
            autowire_mode: Autowire::No,
            init_method_name: None,
            destroy_method_name: None,
            factory_bean_name: None,
            factory_method_name: None,
            constructor_argument_values: ConstructorArgumentValues::new(),
            property_values: MutablePropertyValues::new(),
        }
    }

    /// 设置 Bean 类名。
    pub fn set_bean_class_name(&mut self, name: impl Into<String>) {
        self.bean_class_name = Some(name.into());
    }

    /// 获取 Bean 类名。
    pub fn get_bean_class_name(&self) -> Option<&str> {
        self.bean_class_name.as_deref()
    }

    /// 设置父名称。
    pub fn set_parent_name(&mut self, name: impl Into<String>) {
        self.parent_name = Some(name.into());
    }

    /// 获取父名称。
    pub fn get_parent_name(&self) -> Option<&str> {
        self.parent_name.as_deref()
    }

    /// 设置作用域。
    pub fn set_scope(&mut self, scope: Scope) {
        self.scope = scope;
    }

    /// 获取作用域。
    pub fn get_scope(&self) -> Scope {
        self.scope
    }

    /// 设置惰性初始化。
    pub fn set_lazy_init(&mut self, lazy: bool) {
        self.lazy_init = lazy;
    }

    /// 是否惰性初始化。
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

    /// 设置 autowire 模式。
    pub fn set_autowire_mode(&mut self, mode: Autowire) {
        self.autowire_mode = mode;
    }

    /// 获取 autowire 模式。
    pub fn autowire_mode(&self) -> Autowire {
        self.autowire_mode
    }

    /// 设置 init 方法名。
    pub fn set_init_method_name(&mut self, name: impl Into<String>) {
        self.init_method_name = Some(name.into());
    }

    /// 获取 init 方法名。
    pub fn get_init_method_name(&self) -> Option<&str> {
        self.init_method_name.as_deref()
    }

    /// 设置 destroy 方法名。
    pub fn set_destroy_method_name(&mut self, name: impl Into<String>) {
        self.destroy_method_name = Some(name.into());
    }

    /// 获取 destroy 方法名。
    pub fn get_destroy_method_name(&self) -> Option<&str> {
        self.destroy_method_name.as_deref()
    }

    /// 添加依赖。
    pub fn add_depends_on(&mut self, name: impl Into<String>) {
        self.depends_on.push(name.into());
    }

    /// 获取依赖列表。
    pub fn get_depends_on(&self) -> &[String] {
        &self.depends_on
    }

    /// 设置 factory bean 名称。
    pub fn set_factory_bean_name(&mut self, name: impl Into<String>) {
        self.factory_bean_name = Some(name.into());
    }

    /// 获取 factory bean 名称。
    pub fn get_factory_bean_name(&self) -> Option<&str> {
        self.factory_bean_name.as_deref()
    }

    /// 设置 factory 方法名。
    pub fn set_factory_method_name(&mut self, name: impl Into<String>) {
        self.factory_method_name = Some(name.into());
    }

    /// 获取 factory 方法名。
    pub fn get_factory_method_name(&self) -> Option<&str> {
        self.factory_method_name.as_deref()
    }

    /// 获取构造参数值（可变引用）。
    pub fn get_constructor_argument_values_mut(&mut self) -> &mut ConstructorArgumentValues {
        &mut self.constructor_argument_values
    }

    /// 获取构造参数值。
    pub fn constructor_argument_values(&self) -> &ConstructorArgumentValues {
        &self.constructor_argument_values
    }

    /// 获取属性值（可变引用）。
    pub fn get_property_values_mut(&mut self) -> &mut MutablePropertyValues {
        &mut self.property_values
    }

    /// 获取属性值。
    pub fn property_values(&self) -> &MutablePropertyValues {
        &self.property_values
    }

    /// 设置 abstract 标志。
    pub fn set_abstract_flag(&mut self, flag: bool) {
        self.abstract_flag = flag;
    }

    /// 是否 abstract。
    pub fn is_abstract(&self) -> bool {
        self.abstract_flag
    }

    /// 设置 description。
    pub fn set_description(&mut self, desc: impl Into<String>) {
        self.description = Some(desc.into());
    }

    /// 获取 description。
    pub fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// 设置角色。
    pub fn set_role(&mut self, role: i32) {
        self.role = role;
    }

    /// 获取角色。
    pub fn get_role(&self) -> i32 {
        self.role
    }

    /// 设置 autowire candidate。
    pub fn set_autowire_candidate(&mut self, candidate: bool) {
        self.autowire_candidate = candidate;
    }

    /// 是否 autowire candidate。
    pub fn is_autowire_candidate(&self) -> bool {
        self.autowire_candidate
    }

    /// 设置 fallback。
    pub fn set_fallback(&mut self, fallback: bool) {
        self.fallback = fallback;
    }

    /// 是否 fallback。
    pub fn is_fallback(&self) -> bool {
        self.fallback
    }

    /// 设置 synthetic。
    pub fn set_synthetic(&mut self, synthetic: bool) {
        self.synthetic = synthetic;
    }

    /// 是否 synthetic。
    pub fn is_synthetic(&self) -> bool {
        self.synthetic
    }
}

impl Default for AbstractBeanDefinition {
    fn default() -> Self {
        Self::new()
    }
}

// BeanDefinition trait 选择性实现（不持有 ComponentKey）
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

    fn is_fallback(&self) -> bool {
        self.fallback
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
        self.abstract_flag
    }
}
