//! ChildBeanDefinition — Spring 风格的子 Bean 定义。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ChildBeanDefinition`。
//!
//! 在 Spring 中，`ChildBeanDefinition` 继承自父 Bean 定义，
//! 可以覆盖父定义的部分属性。它与 `GenericBeanDefinition` 不同，
//! 不能独立存在，必须指定父 Bean 名称。
//!
//! ## 设计说明
//!
//! Spring 的 `ChildBeanDefinition` 在运行时会与父定义合并。
//! 在 vernal 中，`ChildBeanDefinition` 保存父名称，合并逻辑由容器处理。

use crate::factory::support::abstract_bean_definition::AbstractBeanDefinition;
use crate::factory::config::bean_definition::BeanDefinition;
use crate::component_key::ComponentKey;
use crate::component_scope::Scope;

/// 子 Bean 定义。
///
/// 对应 Spring 的 `ChildBeanDefinition`。
///
/// 继承父 Bean 定义，可以覆盖部分属性。
/// 必须指定 `parent_name`。
#[derive(Clone, Debug)]
pub struct ChildBeanDefinition {
    /// 父 Bean 定义名称
    pub parent_name: String,
    /// 基础 Bean 定义
    pub base: AbstractBeanDefinition,
}

impl ChildBeanDefinition {
    /// 创建新的子 Bean 定义。
    ///
    /// # 参数
    /// - `parent_name` — 父 Bean 定义名称
    pub fn new(parent_name: impl Into<String>) -> Self {
        Self {
            parent_name: parent_name.into(),
            base: AbstractBeanDefinition::new(),
        }
    }

    /// 创建带 Bean 类名的子 Bean 定义。
    pub fn with_class_name(parent_name: impl Into<String>, class_name: impl Into<String>) -> Self {
        let mut base = AbstractBeanDefinition::new();
        base.bean_class_name = Some(class_name.into());
        Self {
            parent_name: parent_name.into(),
            base,
        }
    }

    /// 获取父 Bean 定义名称。
    pub fn parent_name(&self) -> &str { &self.parent_name }

    /// 设置 Bean 类名。
    pub fn set_bean_class_name(&mut self, name: impl Into<String>) {
        self.base.bean_class_name = Some(name.into());
    }

    /// 设置作用域。
    pub fn set_scope(&mut self, scope: Scope) {
        self.base.set_scope(scope);
    }

    /// 设置延迟初始化。
    pub fn set_lazy_init(&mut self, lazy: bool) {
        self.base.set_lazy_init(lazy);
    }
}

impl BeanDefinition for ChildBeanDefinition {
    fn bean_name(&self) -> &ComponentKey {
        unimplemented!("ChildBeanDefinition does not hold ComponentKey")
    }
    fn bean_class_name(&self) -> &str {
        self.base.bean_class_name().unwrap_or("unknown")
    }
    fn scope(&self) -> Scope { self.base.scope() }
    fn is_lazy_init(&self) -> bool { self.base.is_lazy_init() }
    fn is_primary(&self) -> bool { self.base.is_primary() }
    fn parent_name(&self) -> Option<&str> { Some(&self.parent_name) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_child_bean_definition() {
        let child = ChildBeanDefinition::new("parentBean");
        assert_eq!(child.parent_name(), "parentBean");
        assert_eq!(child.bean_class_name(), "unknown");
    }

    #[test]
    fn with_class_name() {
        let child = ChildBeanDefinition::with_class_name("parent", "com.example.MyService");
        assert_eq!(child.parent_name(), "parent");
        assert_eq!(child.bean_class_name(), "com.example.MyService");
    }

    #[test]
    fn set_properties() {
        let mut child = ChildBeanDefinition::new("parent");
        child.set_bean_class_name("com.example.Child");
        child.set_scope(Scope::Transient);
        child.set_lazy_init(true);

        assert_eq!(child.bean_class_name(), "com.example.Child");
        assert_eq!(child.scope(), Scope::Transient);
        assert!(child.is_lazy_init());
    }

    #[test]
    fn parent_name_from_trait() {
        let child = ChildBeanDefinition::new("myParent");
        let trait_ref: &dyn BeanDefinition = &child;
        assert_eq!(trait_ref.parent_name(), Some("myParent"));
    }
}
