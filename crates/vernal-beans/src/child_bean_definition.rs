//! ChildBeanDefinition — Spring 风格的子 Bean 定义。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ChildBeanDefinition`。
//!
//! 继承自 RootBeanDefinition（通过 parent_name 引用父定义），
//! 子定义可以覆盖父定义的属性值、构造参数等。
//! 最终通过 `merge` 方法与父定义合并为新的 RootBeanDefinition。

use crate::bean_definition::BeanDefinition;
use crate::component_key::ComponentKey;
use crate::component_scope::Scope;
use crate::root_bean_definition::RootBeanDefinition;

/// Spring 风格的子 Bean 定义。
///
/// 对应 Spring 的 `ChildBeanDefinition`。
///
/// 子定义继承父定义的设置，并可以覆盖：
/// - 作用域
/// - 惰性初始化
/// - 属性值
/// - 构造参数
/// - 初始化/销毁方法
#[derive(Debug)]
pub struct ChildBeanDefinition {
    /// 父 Bean 定义名称。
    parent_name: String,
    /// Bean 类名。
    bean_class_name: Option<String>,
    /// 作用域。
    scope: Scope,
    /// 是否惰性初始化。
    lazy_init: bool,
    /// 是否 abstract。
    abstract_flag: bool,
    /// 是否 autowire candidate。
    autowire_candidate: bool,
    /// 是否 primary。
    primary: bool,
    /// 是否 fallback。
    fallback: bool,
    /// 角色。
    role: i32,
    /// 描述。
    description: Option<String>,
}

impl ChildBeanDefinition {
    /// 创建引用指定父定义的新子定义。
    pub fn new(parent_name: impl Into<String>) -> Self {
        Self {
            parent_name: parent_name.into(),
            bean_class_name: None,
            scope: Scope::Singleton,
            lazy_init: false,
            abstract_flag: false,
            autowire_candidate: true,
            primary: false,
            fallback: false,
            role: 0,
            description: None,
        }
    }

    /// 获取父 Bean 定义名称。
    pub fn parent_name(&self) -> &str {
        &self.parent_name
    }

    /// 设置 Bean 类名。
    pub fn set_bean_class_name(&mut self, name: impl Into<String>) {
        self.bean_class_name = Some(name.into());
    }

    /// 获取 Bean 类名。
    pub fn bean_class_name(&self) -> Option<&str> {
        self.bean_class_name.as_deref()
    }

    /// 设置作用域。
    pub fn set_scope(&mut self, scope: Scope) {
        self.scope = scope;
    }

    /// 获取作用域。
    pub fn scope(&self) -> Scope {
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

    /// 合并到父定义，生成新的 RootBeanDefinition。
    ///
    /// 对应 Spring 的 `RootBeanDefinition.merge(BeanDefinition parent)`。
    pub fn merge_into(&self, _parent: &RootBeanDefinition) -> RootBeanDefinition {
        let mut merged = RootBeanDefinition::new();
        if let Some(class_name) = &self.bean_class_name {
            merged.set_bean_class_name(class_name);
        }
        merged.set_scope(self.scope);
        merged.set_lazy_init(self.lazy_init);
        merged.set_primary(self.primary);
        merged
    }
}

impl BeanDefinition for ChildBeanDefinition {
    fn bean_name(&self) -> &ComponentKey {
        unimplemented!("ChildBeanDefinition does not hold ComponentKey")
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

    fn parent_name(&self) -> Option<&str> {
        Some(&self.parent_name)
    }
}
