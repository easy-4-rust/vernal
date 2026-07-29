//! 为低覆盖率文件的针对性覆盖测试。

use std::sync::Arc;
use vernal_beans::{
    AbstractBeanDefinition, AbstractBeanDefinitionReader, AutowireUtils, BeanDefinition,
    BeanDefinitionRegistry, BeanFactory, BeanMetadataAttributeAccessor, ComponentKey, Container,
    RegistryBuilder, RuntimeBeanReference, SimpleInstantiationStrategy, TypedStringValue,
    bean_definition_value_resolver::BeanDefinitionValueResolver,
};

// ── BeanDefinitionValueResolver (0%) ────────────────────────────────

#[test]
fn vresolver_new() {
    let r: Arc<dyn BeanDefinitionRegistry> =
        Arc::new(Container::new(RegistryBuilder::new().build().unwrap()));
    let _resolver = BeanDefinitionValueResolver::new(r);
}

#[test]
fn vresolver_resolve_ref_not_found() {
    let r: Arc<dyn BeanDefinitionRegistry> =
        Arc::new(Container::new(RegistryBuilder::new().build().unwrap()));
    let resolver = BeanDefinitionValueResolver::new(r);
    let val: Arc<dyn std::any::Any + Send + Sync> =
        Arc::new(RuntimeBeanReference::new("noSuchBean"));
    let result = resolver.resolve_value_if_necessary(val);
    assert!(result.is_err());
}

#[test]
fn vresolver_resolve_typed_string() {
    let r: Arc<dyn BeanDefinitionRegistry> =
        Arc::new(Container::new(RegistryBuilder::new().build().unwrap()));
    let resolver = BeanDefinitionValueResolver::new(r);
    let val: Arc<dyn std::any::Any + Send + Sync> = Arc::new(TypedStringValue::new("hello"));
    let result = resolver.resolve_value_if_necessary(val);
    assert!(result.is_ok());
}

#[test]
fn vresolver_resolve_unknown() {
    let r: Arc<dyn BeanDefinitionRegistry> =
        Arc::new(Container::new(RegistryBuilder::new().build().unwrap()));
    let resolver = BeanDefinitionValueResolver::new(r);
    let val: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42i32);
    let result = resolver.resolve_value_if_necessary(val);
    assert!(result.is_ok());
}

// ── SimpleInstantiationStrategy (34%) ──────────────────────────────

#[test]
fn sis_new_and_default() {
    let s = SimpleInstantiationStrategy::new();
    assert_eq!(s.constructor_count(), 0);
    let s2: SimpleInstantiationStrategy = Default::default();
    assert_eq!(s2.constructor_count(), 0);
}

#[test]
fn sis_register_constructor() {
    let mut s = SimpleInstantiationStrategy::new();
    s.register_constructor::<String>(|_args| {
        Ok(Arc::new("constructed".to_string()) as Arc<dyn std::any::Any + Send + Sync>)
    });
    assert_eq!(s.constructor_count(), 1);
}

// ── AbstractBeanDefinitionReader (41%) ─────────────────────────────

#[test]
fn abreader_new_and_methods() {
    use vernal_beans::DefaultBeanNameGenerator;

    struct MockReg;
    impl BeanDefinitionRegistry for MockReg {
        fn register_bean_definition(
            &mut self,
            _: String,
            _: Box<dyn BeanDefinition>,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn remove_bean_definition(
            &mut self,
            _: &str,
        ) -> Result<Box<dyn BeanDefinition>, Box<dyn std::error::Error + Send + Sync>> {
            unimplemented!()
        }
        fn get_bean_definition(&self, _: &str) -> Option<&'static dyn BeanDefinition> {
            None
        }
        fn contains_bean_definition(&self, _: &str) -> bool {
            false
        }
        fn bean_definition_count(&self) -> usize {
            0
        }
        fn bean_definition_names(&self) -> Vec<String> {
            vec![]
        }
    }

    let mut r = AbstractBeanDefinitionReader::new(Box::new(MockReg));
    assert!(r.get_registry().bean_definition_names().is_empty());
    assert!(r.bean_name_generator().is_none());

    r.set_bean_name_generator(Box::new(DefaultBeanNameGenerator::new()));
    assert!(r.bean_name_generator().is_some());

    r.defaults_mut().set_lazy_init(true);
    assert!(r.defaults().is_lazy_init());
}

// ── AutowireUtils (58%) ────────────────────────────────────────────

#[test]
fn autowire_utils_determine_empty() {
    let factory = Container::new(RegistryBuilder::new().build().unwrap());
    let result = AutowireUtils::determine_autowire_candidates(
        &factory,
        std::any::TypeId::of::<String>(),
        &["b1".to_string()],
        &|_name| true,
        &|_name| ComponentKey::of::<String>(),
    );
    assert!(result.is_ok());
}

#[test]
fn autowire_utils_determine_filtered() {
    let mut b = RegistryBuilder::new();
    b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(
        |_| "hello".to_string(),
    ));
    let factory = Container::new(b.build().unwrap());
    let type_name = std::any::type_name::<String>();
    let result = AutowireUtils::determine_autowire_candidates(
        &factory,
        std::any::TypeId::of::<String>(),
        &[type_name.to_string()],
        &|_| false,
        &|_name| ComponentKey::of::<String>(),
    );
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

// ── BeanMetadataAttributeAccessor (60%) ────────────────────────────

#[test]
fn accessor_get_metadata_attribute() {
    let mut a = BeanMetadataAttributeAccessor::new();
    a.set_attribute("k", Box::new(42i32));
    let attr = a.get_metadata_attribute("k");
    assert!(attr.is_some());
    assert_eq!(attr.unwrap().name(), "k");
}

#[test]
fn accessor_clear_len_empty() {
    let mut a = BeanMetadataAttributeAccessor::new();
    assert!(a.is_empty());
    a.set_attribute("a", Box::new(1i32));
    a.set_attribute("b", Box::new(2i32));
    assert!(!a.is_empty());
    assert_eq!(a.len(), 2);
    a.clear();
    assert!(a.is_empty());
}

#[test]
fn accessor_has_not() {
    let a = BeanMetadataAttributeAccessor::new();
    assert!(!a.has_attribute("nil"));
}

#[test]
fn accessor_get_dyn_any() {
    let mut a = BeanMetadataAttributeAccessor::new();
    a.set_attribute("num", Box::new(100i32));
    let v = a.get_attribute("num");
    assert!(v.is_some());
    assert!(v.unwrap().downcast_ref::<i32>().is_some());
}
