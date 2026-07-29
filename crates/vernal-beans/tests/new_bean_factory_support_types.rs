//! BeanFactory 支持类型（第四批）的覆盖测试。

use std::sync::Arc;
use vernal_beans::{
    AbstractBeanDefinition, Autowire, AutowireCandidateQualifier, AutowireUtils, BeanDefinition,
    BeanDefinitionCustomizer, BeanFactoryInitializer, BeanReference, FactoryBeanRegistrySupport,
    RuntimeBeanNameReference, RuntimeBeanReference, SimpleInstantiationStrategy, TypedStringValue,
};

// ── BeanReference trait ─────────────────────────────────────────────

#[test]
fn bean_reference_trait_object() {
    let r = RuntimeBeanReference::new("myBean");
    let r_ref: &dyn BeanReference = &r;
    assert_eq!(r_ref.get_bean_name(), "myBean");
    assert!(r_ref.get_source().is_none());
}

// ── RuntimeBeanReference ────────────────────────────────────────────

#[test]
fn runtime_bean_reference_new() {
    let r = RuntimeBeanReference::new("testBean");
    assert_eq!(r.get_bean_name(), "testBean");
}

#[test]
fn runtime_bean_reference_with_source() {
    let r = RuntimeBeanReference::with_source("myBean", Box::new(42i32));
    assert_eq!(r.get_bean_name(), "myBean");
    assert!(r.get_source().is_some());
}

#[test]
fn runtime_bean_reference_eq() {
    let a = RuntimeBeanReference::new("same");
    let b = RuntimeBeanReference::new("same");
    assert_eq!(a, b);
}

#[test]
fn runtime_bean_reference_ne() {
    let a = RuntimeBeanReference::new("a");
    let b = RuntimeBeanReference::new("b");
    assert_ne!(a, b);
}

// ── RuntimeBeanNameReference ─────────────────────────────────────────

#[test]
fn name_ref_new() {
    let r = RuntimeBeanNameReference::new("myRef");
    assert_eq!(r.get_bean_name(), "myRef");
}

#[test]
fn name_ref_source() {
    let r = RuntimeBeanNameReference::with_source("b", Box::new("src".to_string()));
    assert_eq!(r.get_bean_name(), "b");
}

#[test]
fn name_ref_set_source() {
    let mut r = RuntimeBeanNameReference::new("x");
    assert!(r.get_source().is_none());
    r.set_source(Box::new(99i32));
    assert!(r.get_source().is_some());
}

// ── TypedStringValue ────────────────────────────────────────────────

#[test]
fn typed_str_new() {
    let tv = TypedStringValue::new("hello");
    assert_eq!(tv.value(), "hello");
    assert!(tv.get_target_type_name().is_none());
}

#[test]
fn typed_str_with_type() {
    let tv = TypedStringValue::with_target_type("42", "java.lang.Integer");
    assert_eq!(tv.value(), "42");
    assert_eq!(tv.get_target_type_name(), Some("java.lang.Integer"));
}

#[test]
fn typed_str_set_type() {
    let mut tv = TypedStringValue::new("test");
    tv.set_target_type_name("com.example.MyType");
    assert_eq!(tv.get_target_type_name(), Some("com.example.MyType"));
}

// ── FactoryBeanRegistrySupport ─────────────────────────────────────

#[test]
fn fbrs_cache_ops() {
    let s = FactoryBeanRegistrySupport::new();
    let obj: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42i32);
    assert!(s.get_cached_object("k").is_none());
    s.cache_object("k", obj.clone());
    assert!(s.get_cached_object("k").is_some());
    s.remove_cached_object("k");
    assert!(s.get_cached_object("k").is_none());
}

#[test]
fn fbrs_clear() {
    let s = FactoryBeanRegistrySupport::new();
    s.cache_object("a", Arc::new(1i32));
    s.cache_object("b", Arc::new(2i32));
    s.clear_cache();
    assert!(s.get_cached_object("a").is_none());
    assert!(s.get_cached_object("b").is_none());
}

// ── SimpleInstantiationStrategy ────────────────────────────────────

#[test]
fn sis_new() {
    let _s = SimpleInstantiationStrategy::new();
}

// ── BeanDefinitionCustomizer ───────────────────────────────────────

#[test]
fn bdc_trait() {
    struct C;
    impl BeanDefinitionCustomizer for C {
        fn customize(&self, _bd: &mut dyn BeanDefinition) {}
    }
    let mut def = AbstractBeanDefinition::new();
    C.customize(&mut def);
}

// ── AutowireCandidateQualifier ─────────────────────────────────────

#[test]
fn acq_new() {
    let q = AutowireCandidateQualifier::new("com.example.Qual");
    assert_eq!(q.type_name(), "com.example.Qual");
}

#[test]
fn acq_attrs() {
    let mut q = AutowireCandidateQualifier::new("test.Q");
    q.set_attribute("value", Box::new("primary".to_string()));
    assert!(q.get_attribute("value").is_some());
}

// ── AutowireUtils ───────────────────────────────────────────────────

#[test]
fn autowire_utils_basic() {
    assert!(AutowireUtils::is_autowire_mode_match(
        Autowire::ByName,
        &AbstractBeanDefinition::new() as &dyn BeanDefinition
    ));
    assert!(AutowireUtils::is_autowire_mode_match(
        Autowire::ByName,
        &AbstractBeanDefinition::new() as &dyn BeanDefinition
    ));

    let def = AbstractBeanDefinition::new();
    assert!(AutowireUtils::is_autowire_applicable(&def));
    assert!(AutowireUtils::is_factory_method("myBean"));
}

// ── BeanFactoryInitializer trait ───────────────────────────────────

#[test]
fn bfi_trait() {
    use vernal_beans::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
    struct Init;
    impl BeanFactoryInitializer for Init {
        fn initialize(
            &self,
            _f: &mut dyn ConfigurableListableBeanFactory,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }
    let mut c = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    assert!(Init.initialize(&mut c).is_ok());
}
