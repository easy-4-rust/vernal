//! AbstractBeanDefinition — Spring 风格的 Bean 定义抽象基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AbstractBeanDefinition`。
//! RootBeanDefinition 和 ChildBeanDefinition 的父类。

use crate::bean_definition::BeanDefinition;
use crate::component_scope::Scope;
use std::any::Any;
use std::fmt;

/// Spring 风格的 Bean 定义抽象基类。
///
/// 对应 Spring 的 `AbstractBeanDefinition`。
/// 提供 BeanDefinition trait 的默认实现和通用字段访问器。
#[derive(Clone, Debug, Default)]
pub struct AbstractBeanDefinition {
    pub bean_class_name: Option<String>,
    pub parent_name: Option<String>,
    pub scope: Scope,
    pub lazy_init: bool,
    pub primary: bool,
    pub autowire_candidate: bool,
    pub role: i32,
    pub description: Option<String>,
    pub init_method_name: Option<String>,
    pub destroy_method_name: Option<String>,
    pub factory_bean_name: Option<String>,
    pub factory_method_name: Option<String>,
}

impl AbstractBeanDefinition {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bean_class_name(&self) -> Option<&str> {
        self.bean_class_name.as_deref()
    }

    pub fn parent_name(&self) -> Option<&str> {
        self.parent_name.as_deref()
    }

    pub fn scope(&self) -> Scope {
        self.scope
    }

    pub fn set_scope(&mut self, scope: Scope) {
        self.scope = scope;
    }

    pub fn set_lazy_init(&mut self, lazy: bool) {
        self.lazy_init = lazy;
    }

    pub fn set_primary(&mut self, primary: bool) {
        self.primary = primary;
    }

    pub fn is_lazy_init(&self) -> bool {
        self.lazy_init
    }

    pub fn is_primary(&self) -> bool {
        self.primary
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
}

use crate::component_key::ComponentKey;
