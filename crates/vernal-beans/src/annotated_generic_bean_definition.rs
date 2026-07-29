use std::any::Any;
use crate::annotated_bean_definition::AnnotatedBeanDefinition;
use crate::bean_definition::BeanDefinition;
use crate::component_key::ComponentKey;
use crate::component_scope::Scope;
use crate::generic_bean_definition::GenericBeanDefinition;

/// Spring 风格的注解驱动通用 Bean 定义。
#[derive(Clone, Debug)]
pub struct AnnotatedGenericBeanDefinition {
    pub inner: GenericBeanDefinition,
    pub annotation_metadata: Option<String>,
}

impl AnnotatedGenericBeanDefinition {
    pub fn new(annotation_type: &str) -> Self {
        Self {
            inner: GenericBeanDefinition::new(),
            annotation_metadata: Some(annotation_type.to_string()),
        }
    }
}

impl BeanDefinition for AnnotatedGenericBeanDefinition {
    fn bean_name(&self) -> &ComponentKey { unimplemented!() }
    fn bean_class_name(&self) -> &str { self.inner.get_bean_class_name().unwrap_or("unknown") }
    fn scope(&self) -> Scope { self.inner.scope() }
    fn is_lazy_init(&self) -> bool { self.inner.is_lazy_init() }
    fn is_primary(&self) -> bool { self.inner.is_primary() }
}

impl AnnotatedBeanDefinition for AnnotatedGenericBeanDefinition {
    fn annotation_metadata(&self) -> &dyn Any {
        &self.annotation_metadata
    }
}
