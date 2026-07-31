use std::any::{Any, TypeId};
use crate::factory::annotation::annotated_bean_definition::{AnnotatedBeanDefinition, BeanMetadata};
use crate::factory::config::bean_definition::BeanDefinition;
use crate::component_key::ComponentKey;
use crate::component_scope::Scope;
use crate::factory::support::generic_bean_definition::GenericBeanDefinition;

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
    fn get_metadata(&self) -> Option<BeanMetadata> {
        self.annotation_metadata.as_ref().map(|name| BeanMetadata::new(name.clone(), 0))
    }
    fn is_factory_method(&self, _name: &str) -> bool { false }
    fn get_factory_method_name(&self) -> Option<String> { None }
    fn get_bean_type(&self) -> TypeId { TypeId::of::<Self>() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_with_annotation_metadata() {
        let def = AnnotatedGenericBeanDefinition::new("MyAnnotation");
        assert_eq!(def.annotation_metadata, Some("MyAnnotation".to_string()));
    }

    #[test]
    fn bean_class_name_from_inner() {
        let def = AnnotatedGenericBeanDefinition::new("TestAnnotation");
        // GenericBeanDefinition::new() has no class name set, so returns "unknown"
        assert_eq!(BeanDefinition::bean_class_name(&def), "unknown");
    }

    #[test]
    fn scope_defaults_to_singleton() {
        let def = AnnotatedGenericBeanDefinition::new("TestAnnotation");
        assert_eq!(BeanDefinition::scope(&def), Scope::Singleton);
    }

    #[test]
    fn is_lazy_init_defaults_false() {
        let def = AnnotatedGenericBeanDefinition::new("TestAnnotation");
        assert!(!BeanDefinition::is_lazy_init(&def));
    }

    #[test]
    fn is_primary_defaults_false() {
        let def = AnnotatedGenericBeanDefinition::new("TestAnnotation");
        assert!(!BeanDefinition::is_primary(&def));
    }

    #[test]
    fn get_metadata_returns_class_name() {
        let def = AnnotatedGenericBeanDefinition::new("Service");
        let metadata = AnnotatedBeanDefinition::get_metadata(&def);
        assert!(metadata.is_some());
        let metadata = metadata.unwrap();
        assert_eq!(metadata.class_name, "Service");
    }

    #[test]
    fn get_metadata_none_when_no_annotation() {
        let mut def = AnnotatedGenericBeanDefinition::new("Test");
        def.annotation_metadata = None;
        assert!(AnnotatedBeanDefinition::get_metadata(&def).is_none());
    }

    #[test]
    fn is_factory_method_always_false() {
        let def = AnnotatedGenericBeanDefinition::new("Test");
        assert!(!AnnotatedBeanDefinition::is_factory_method(&def, "any_method"));
    }

    #[test]
    fn get_factory_method_name_always_none() {
        let def = AnnotatedGenericBeanDefinition::new("Test");
        assert!(AnnotatedBeanDefinition::get_factory_method_name(&def).is_none());
    }

    #[test]
    fn get_bean_type_returns_self_type() {
        let def = AnnotatedGenericBeanDefinition::new("Test");
        let type_id = AnnotatedBeanDefinition::get_bean_type(&def);
        assert_eq!(type_id, TypeId::of::<AnnotatedGenericBeanDefinition>());
    }

    #[test]
    fn clone_works() {
        let def = AnnotatedGenericBeanDefinition::new("Test");
        let cloned = def.clone();
        assert_eq!(cloned.annotation_metadata, def.annotation_metadata);
    }

    #[test]
    fn debug_format() {
        let def = AnnotatedGenericBeanDefinition::new("Test");
        let debug_str = format!("{:?}", def);
        assert!(debug_str.contains("AnnotatedGenericBeanDefinition"));
    }
}
