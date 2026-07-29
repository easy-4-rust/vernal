//! BeanDefinitionDefaults — Spring 风格的 BeanDef 默认值集合。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionDefaults`。
//!
//! 存储 Bean 定义的默认值：作用域、惰性初始化、autowire 模式、
//! init/destroy 方法名等。`BeanDefinitionReader` 构建定义时使用。

use crate::autowire::Autowire;
use crate::component_scope::Scope;

/// Spring 风格的 BeanDefinition 默认值集合。
///
/// 对应 Spring 的 `BeanDefinitionDefaults`。
#[derive(Clone, Debug)]
pub struct BeanDefinitionDefaults {
    /// 默认作用域（默认为 Singleton）。
    scope: Scope,
    /// 默认惰性初始化（默认为 false）。
    lazy_init: bool,
    /// 默认 autowire 模式（默认为 Autowire::No）。
    autowire_mode: Autowire,
    /// 默认 init 方法名。
    init_method_name: Option<String>,
    /// 默认 destroy 方法名。
    destroy_method_name: Option<String>,
}

impl BeanDefinitionDefaults {
    /// 创建默认值集合。
    pub fn new() -> Self {
        Self {
            scope: Scope::Singleton,
            lazy_init: false,
            autowire_mode: Autowire::No,
            init_method_name: None,
            destroy_method_name: None,
        }
    }

    /// 获取默认作用域。
    pub fn scope(&self) -> Scope {
        self.scope
    }

    /// 设置默认作用域。
    pub fn set_scope(&mut self, scope: Scope) {
        self.scope = scope;
    }

    /// 获取默认惰性初始化。
    pub fn is_lazy_init(&self) -> bool {
        self.lazy_init
    }

    /// 设置默认惰性初始化。
    pub fn set_lazy_init(&mut self, lazy: bool) {
        self.lazy_init = lazy;
    }

    /// 获取默认 autowire 模式。
    pub fn autowire_mode(&self) -> Autowire {
        self.autowire_mode
    }

    /// 设置默认 autowire 模式。
    pub fn set_autowire_mode(&mut self, mode: Autowire) {
        self.autowire_mode = mode;
    }

    /// 获取默认 init 方法名。
    pub fn init_method_name(&self) -> Option<&str> {
        self.init_method_name.as_deref()
    }

    /// 设置默认 init 方法名。
    pub fn set_init_method_name(&mut self, name: impl Into<String>) {
        self.init_method_name = Some(name.into());
    }

    /// 获取默认 destroy 方法名。
    pub fn destroy_method_name(&self) -> Option<&str> {
        self.destroy_method_name.as_deref()
    }

    /// 设置默认 destroy 方法名。
    pub fn set_destroy_method_name(&mut self, name: impl Into<String>) {
        self.destroy_method_name = Some(name.into());
    }
}

impl Default for BeanDefinitionDefaults {
    fn default() -> Self {
        Self::new()
    }
}
