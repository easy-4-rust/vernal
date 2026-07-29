//! ChildBeanDefinition — Spring 风格的子 Bean 定义。
//! 对应 Java 类：`org.springframework.beans.factory.support.ChildBeanDefinition`。
use crate::abstract_bean_definition::AbstractBeanDefinition;
use crate::bean_definition::BeanDefinition;
use crate::component_key::ComponentKey;
use crate::component_scope::Scope;

/// Spring 风格的子 Bean 定义。
#[derive(Clone, Debug)]
pub struct ChildBeanDefinition {
    pub parent_name: String,
    pub base: AbstractBeanDefinition,
}

impl ChildBeanDefinition {
    pub fn new(parent_name: impl Into<String>) -> Self {
        Self {
            parent_name: parent_name.into(),
            base: AbstractBeanDefinition::new(),
        }
    }
    pub fn parent_name(&self) -> &str { &self.parent_name }
}

impl BeanDefinition for ChildBeanDefinition {
    fn bean_name(&self) -> &ComponentKey { unimplemented!("ChildBeanDefinition does not hold ComponentKey") }
    fn bean_class_name(&self) -> &str { self.base.bean_class_name().unwrap_or("unknown") }
    fn scope(&self) -> Scope { self.base.scope() }
    fn is_lazy_init(&self) -> bool { self.base.is_lazy_init() }
    fn is_primary(&self) -> bool { self.base.is_primary() }
}
