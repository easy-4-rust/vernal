//! BeanDefinition 层次结构文件（第三批）的覆盖测试。

use std::any::Any;
use vernal_beans::{
    AbstractBeanDefinition, AbstractBeanDefinitionReader, BeanDefinition, BeanDefinitionDefaults,
    BeanDefinitionOverridingStrategy, BeanDefinitionVisitor, BeanMetadataAttribute,
    BeanMetadataAttributeAccessor, BeanMetadataElement, Scope, SimpleAutowireCandidateResolver,
};

// ── BeanDefinitionVisitor ──────────────────────────────────────────

#[test]
fn visitor_trait_default_impl() {
    struct TestVisitor;
    impl BeanDefinitionVisitor for TestVisitor {}

    let mut v = TestVisitor;
    let def = AbstractBeanDefinition::new();
    v.visit_bean_definition(&def);
    v.visit_constructor_argument_values(def.constructor_argument_values());
    v.visit_property_values(def.property_values());
    v.visit_depends_on(&[]);
}

// ── BeanDefinitionOverridingStrategy ──────────────────────────────

#[test]
fn overriding_always() {
    let s = BeanDefinitionOverridingStrategy::OverrideAlways;
    let def = AbstractBeanDefinition::new();
    assert!(s.should_override(&def, &def));
}

#[test]
fn overriding_never() {
    let s = BeanDefinitionOverridingStrategy::OverrideNever;
    let def = AbstractBeanDefinition::new();
    assert!(!s.should_override(&def, &def));
}

#[test]
fn overriding_if_exists() {
    let s = BeanDefinitionOverridingStrategy::OverrideIfExists;
    let def = AbstractBeanDefinition::new();
    assert!(s.should_override(&def, &def));
}

#[test]
fn overriding_default() {
    assert_eq!(
        BeanDefinitionOverridingStrategy::default(),
        BeanDefinitionOverridingStrategy::OverrideAlways
    );
}

// ── AbstractBeanDefinitionReader ─────────────────────────────────

#[test]
fn abstract_reader_new_with_defaults() {
    use vernal_beans::DefaultBeanNameGenerator;
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;

    struct MockRegistry;
    impl BeanDefinitionRegistry for MockRegistry {
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

    let mut reader = AbstractBeanDefinitionReader::new(Box::new(MockRegistry));
    assert_eq!(reader.defaults().scope(), Scope::Singleton);

    // Set a bean name generator
    reader.set_bean_name_generator(Box::new(DefaultBeanNameGenerator::new()));
    assert!(reader.bean_name_generator().is_some());

    // defaults_mut
    reader.defaults_mut().set_scope(Scope::Transient);
    assert_eq!(reader.defaults().scope(), Scope::Transient);
}

// ── Exception types ──────────────────────────────────────────────

#[test]
fn override_exception_basic() {
    use vernal_beans::bean_definition_override_exception::BeanDefinitionOverrideException;
    let e = BeanDefinitionOverrideException::new("myBean", "Override conflict");
    let msg = format!("{}", e);
    assert!(msg.contains("myBean"));
}

#[test]
fn store_exception_basic() {
    use vernal_beans::bean_definition_store_exception::BeanDefinitionStoreException;
    let e = BeanDefinitionStoreException::new("resource.xml", "Failed");
    let msg = format!("{}", e);
    assert!(msg.contains("resource.xml"));
}

#[test]
fn store_exception_with_cause() {
    use vernal_beans::bean_definition_store_exception::BeanDefinitionStoreException;
    let cause = std::io::Error::new(std::io::ErrorKind::Other, "inner");
    let e = BeanDefinitionStoreException::with_cause("res", "outer", Box::new(cause));
    let msg = format!("{}", e);
    assert!(msg.contains("outer"));
}

#[test]
fn parsing_exception_basic() {
    use vernal_beans::bean_definition_parsing_exception::BeanDefinitionParsingException;
    let e = BeanDefinitionParsingException::new("Invalid XML");
    let msg = format!("{}", e);
    assert!(msg.contains("Invalid XML"));
}

#[test]
fn validation_exception_basic() {
    use vernal_beans::bean_definition_validation_exception::BeanDefinitionValidationException;
    let e = BeanDefinitionValidationException::new("invalid");
    let msg = format!("{}", e);
    assert!(msg.contains("invalid"));
}

#[test]
fn bean_is_abstract_exception_basic() {
    use vernal_beans::bean_is_abstract_exception::BeanIsAbstractException;
    let e = BeanIsAbstractException::new("abstractBean");
    let msg = format!("{}", e);
    assert!(msg.contains("abstractBean"));
}

#[test]
fn factory_bean_not_initialized_basic() {
    use vernal_beans::factory_bean_not_initialized_exception::FactoryBeanNotInitializedException;
    let e = FactoryBeanNotInitializedException::new("not ready");
    let msg = format!("{}", e);
    assert!(msg.contains("not ready"));
}

#[test]
fn factory_bean_not_initialized_default() {
    use vernal_beans::factory_bean_not_initialized_exception::FactoryBeanNotInitializedException;
    let e = FactoryBeanNotInitializedException::default();
    let msg = format!("{}", e);
    assert!(!msg.is_empty());
}

#[test]
fn bean_is_not_a_factory_exception_basic() {
    use vernal_beans::bean_is_not_a_factory_exception::BeanIsNotAFactoryException;
    let e = BeanIsNotAFactoryException::new("myBean");
    let msg = format!("{}", e);
    assert!(msg.contains("myBean"));
}

// ── BeanMetadataElement trait ────────────────────────────────────

#[test]
fn metadata_element_default() {
    struct TestElem;
    impl BeanMetadataElement for TestElem {}
    let e = TestElem;
    assert!(e.get_source().is_none());
}

#[test]
fn metadata_element_with_source() {
    struct TestElem {
        source: i32,
    }
    impl BeanMetadataElement for TestElem {
        fn get_source(&self) -> Option<&dyn Any> {
            Some(&self.source)
        }
    }
    let e = TestElem { source: 42 };
    assert!(e.get_source().is_some());
}

// ── BeanMetadataAttribute ────────────────────────────────────────

#[test]
fn metadata_attribute_new_value() {
    let attr = BeanMetadataAttribute::with_value("key", Box::new(42i32));
    assert_eq!(attr.name(), "key");
    assert!(attr.value().is_some());
}

#[test]
fn metadata_attribute_empty() {
    let attr = BeanMetadataAttribute::new("empty");
    assert_eq!(attr.name(), "empty");
    assert!(attr.value().is_none());
}

#[test]
fn metadata_attribute_source() {
    let attr = BeanMetadataAttribute::with_value("k", Box::new("v".to_string()));
    assert!(attr.get_source().is_none());
}

// ── BeanMetadataAttributeAccessor ────────────────────────────────

#[test]
fn accessor_set_get() {
    let mut a = BeanMetadataAttributeAccessor::new();
    a.set_attribute("color", Box::new("red".to_string()));
    assert!(a.get_attribute("color").is_some());
}

#[test]
fn accessor_has_not() {
    let a = BeanMetadataAttributeAccessor::new();
    assert!(!a.has_attribute("missing"));
}

#[test]
fn accessor_remove() {
    let mut a = BeanMetadataAttributeAccessor::new();
    a.set_attribute("x", Box::new(1i32));
    assert!(a.has_attribute("x"));
    a.remove_attribute("x");
    assert!(!a.has_attribute("x"));
}

#[test]
fn accessor_names() {
    let mut a = BeanMetadataAttributeAccessor::new();
    a.set_attribute("a", Box::new(1i32));
    a.set_attribute("b", Box::new(2i32));
    let names = a.attribute_names();
    assert!(names.len() == 2);
}

// ── SimpleAutowireCandidateResolver ──────────────────────────────

#[test]
fn resolver_is_autowire_candidate_true() {
    let resolver = SimpleAutowireCandidateResolver::new();
    let mut def = AbstractBeanDefinition::new();
    def.set_bean_class_name("test.TestService");
    assert!(resolver.is_autowire_candidate(&def));
}

#[test]
fn resolver_is_autowire_candidate_false() {
    let resolver = SimpleAutowireCandidateResolver::new();
    let mut def = AbstractBeanDefinition::new();
    def.set_autowire_candidate(false);
    assert!(!resolver.is_autowire_candidate(&def));
}

// ── BeanDefinitionDefaults (check re-export path) ─────────────────

#[test]
fn bean_definition_defaults_basic() {
    let d = BeanDefinitionDefaults::new();
    assert_eq!(d.scope(), Scope::Singleton);
    assert!(!d.is_lazy_init());
}
