//! Coverage recovery test file — targets specific uncovered code.
use std::sync::Arc;
use vernal_beans::{
    bean_definition::BeanDefinition,
    bean_factory::BeanFactory,
    abstract_bean_definition::AbstractBeanDefinition,
    ComponentDefinition, ComponentKey, Container, RegistryBuilder,
};

#[test]
fn bean_def_trait_coverage() {
    let mut def = AbstractBeanDefinition::new();
    def.set_scope(vernal_beans::Scope::Transient);
    def.set_lazy_init(true);
    def.set_primary(true);
    
    let bd: &dyn BeanDefinition = &def;
    assert_eq!(bd.scope(), vernal_beans::Scope::Transient);
    assert!(bd.is_lazy_init());
    assert!(bd.is_primary());
    assert!(!bd.is_fallback());
    assert!(bd.is_autowire_candidate());
    assert!(!bd.is_abstract());
    assert!(!bd.is_singleton());
    assert!(bd.is_prototype());
    assert!(bd.resource_description().is_none());
    assert!(bd.originating_bean_definition().is_none());
}

#[test]
fn container_bean_methods() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let c = Container::new(b.build().unwrap());
    assert!(c.contains_bean(&ComponentKey::of::<String>()));
    assert!(!c.contains_bean(&ComponentKey::of::<i32>()));
    assert!(c.get_type(&ComponentKey::of::<String>()).is_ok());
    assert!(c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<String>()));
    assert!(!c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<i32>()));
    assert!(c.get_aliases(&ComponentKey::of::<String>()).is_empty());
    assert!(c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).is_ok());
    assert_eq!(c.bean_post_processor_count(), 0);
}

#[test]
fn container_scope_open() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let c = Container::new(b.build().unwrap());
    let _scope = c.open_scope::<String>();
}
