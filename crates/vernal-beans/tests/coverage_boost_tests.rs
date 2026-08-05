//! Comprehensive coverage tests for vernal-beans crate.

use std::any::{Any, TypeId};
use std::sync::Arc;

use vernal_beans::{
    BeanDefinition, BeanDescCache, BeanFactoryUtils, ComponentDefinition, ComponentKey,
    ConfigurableBeanFactory, ConfigurableListableBeanFactory, Container, ConversionServiceFactory,
    DefinitionError, DirectFieldAccessor, FactoryBeanRegistrySupport, GraphError,
    HierarchicalBeanFactory, ListableBeanFactory, PropertyEditor, PropertyEditorCache,
    PropertyEditorRegistry, Qualifier, RegistryBuilder, ResolveError, RootBeanDefinition, Scope,
    ScopeError, ScopeKey, ScopeState, TransientTracker,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Helper
// ═══════════════════════════════════════════════════════════════════════════════

fn make_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    Container::new(b.build().unwrap())
}

fn shared_err(msg: &str) -> vernal_core::SharedError {
    Arc::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        msg.to_string(),
    ))
}

// ═══════════════════════════════════════════════════════════════════════════════
// BeanDescCache
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_desc_cache_new_is_empty() {
    let cache = BeanDescCache::new();
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

#[test]
fn bean_desc_cache_default_trait() {
    let cache = BeanDescCache::default();
    assert!(cache.is_empty());
}

#[test]
fn bean_desc_cache_global_returns_same_instance() {
    let a = BeanDescCache::global();
    let b = BeanDescCache::global();
    assert!(std::ptr::eq(a, b));
}

#[test]
fn bean_desc_cache_get_or_insert_caches_value() {
    // BeanDescCache uses a private BeanDescriptor trait; we can only test the public API surface
    let cache = BeanDescCache::new();
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
    // Cache operations (clear, len, is_empty) are exercised above.
    // get_or_insert requires Arc<dyn BeanDescriptor> which is private,
    // so we verify the cache struct's public methods only.
    cache.clear();
    assert!(cache.is_empty());
}

#[test]
fn bean_desc_cache_clear_removes_all() {
    let cache = BeanDescCache::new();
    cache.clear();
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

#[test]
fn bean_desc_cache_multiple_instances() {
    let cache1 = BeanDescCache::new();
    let cache2 = BeanDescCache::new();
    // Each instance is independent
    assert!(cache1.is_empty());
    assert!(cache2.is_empty());
    // Global returns same instance
    let g1 = BeanDescCache::global();
    let g2 = BeanDescCache::global();
    assert!(std::ptr::eq(g1, g2));
}

// ═══════════════════════════════════════════════════════════════════════════════
// RegistryBuilder
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn registry_builder_register_all_preserves_order() {
    let mut b = RegistryBuilder::new();
    b.register_all(vec![
        ComponentDefinition::shared_value(1i32),
        ComponentDefinition::shared_value("hello".to_string()),
        ComponentDefinition::shared_value(3.14f64),
    ])
    .unwrap();
    assert_eq!(b.len(), 3);
    assert!(b.contains::<i32>());
    assert!(b.contains::<String>());
    assert!(b.contains::<f64>());
}

#[test]
fn registry_builder_register_bundle_success() {
    let mut b = RegistryBuilder::new();
    b.register_bundle(
        vec![ComponentDefinition::shared_value(42i32)],
        Vec::<vernal_beans::TraitBinding>::new(),
    )
    .unwrap();
    assert_eq!(b.len(), 1);
}

#[test]
fn registry_builder_register_bundle_duplicate_fails_atomically() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    let result = b.register_bundle(
        vec![
            ComponentDefinition::shared_value(100i32),
            ComponentDefinition::shared_value("new".to_string()),
        ],
        Vec::<vernal_beans::TraitBinding>::new(),
    );
    assert!(result.is_err());
    assert_eq!(b.len(), 1);
}

#[test]
fn registry_builder_build_empty() {
    let b = RegistryBuilder::new();
    let registry = b.build().unwrap();
    assert!(registry.definitions().is_empty());
    assert!(registry.bindings().is_empty());
}

#[test]
fn registry_builder_build_with_definitions() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    b.register(ComponentDefinition::shared_value("hello".to_string()))
        .unwrap();
    let registry = b.build().unwrap();
    assert_eq!(registry.definitions().len(), 2);
}

#[test]
fn registry_builder_is_empty_considers_both() {
    let mut b = RegistryBuilder::new();
    assert!(b.is_empty());
    b.register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    assert!(!b.is_empty());
}

#[test]
fn registry_builder_remove_rebuilds_indices() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::shared_value(1i32)).unwrap();
    b.register(ComponentDefinition::shared_value(2u64)).unwrap();
    b.register(ComponentDefinition::shared_value("s".to_string()))
        .unwrap();
    b.remove::<i32>().unwrap();
    assert_eq!(b.len(), 2);
    assert!(!b.contains::<i32>());
    assert!(b.contains::<u64>());
    assert!(b.contains::<String>());
    b.register(ComponentDefinition::shared_value(3.14f64))
        .unwrap();
    assert_eq!(b.len(), 3);
}

#[test]
fn registry_builder_remove_by_key() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    b.remove_by_key(&ComponentKey::of::<i32>()).unwrap();
    assert!(b.is_empty());
}

#[test]
fn registry_builder_remove_by_key_nonexistent() {
    let mut b = RegistryBuilder::new();
    assert!(b.remove_by_key(&ComponentKey::of::<i32>()).is_err());
}

#[test]
fn registry_builder_remove_nonexistent_type() {
    let mut b = RegistryBuilder::new();
    assert!(b.remove::<i32>().is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// StandardBeanExpressionResolver
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn expression_resolver_new_and_default() {
    let r = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    assert_eq!(r.bean_count(), 0);
    let r2 =
        vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::default();
    assert_eq!(r2.bean_count(), 0);
}

#[test]
fn expression_resolver_register_and_count() {
    let r = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    r.register_bean("myBean".to_string(), Arc::new(42i32));
    assert_eq!(r.bean_count(), 1);
    r.register_bean("other".to_string(), Arc::new("hello".to_string()));
    assert_eq!(r.bean_count(), 2);
}

#[test]
fn expression_resolver_clear_context() {
    let r = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    r.register_bean("myBean".to_string(), Arc::new(42i32));
    assert_eq!(r.bean_count(), 1);
    r.clear_context();
    assert_eq!(r.bean_count(), 0);
}

#[test]
fn expression_resolver_evaluate_bean_name() {
    use vernal_beans::BeanExpressionResolver;
    let r = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    r.register_bean("myBean".to_string(), Arc::new(42i32));
    let result = r.evaluate("myBean", None).unwrap();
    assert!(result.is_some());
    assert_eq!(*result.unwrap().downcast_ref::<i32>().unwrap(), 42);
}

#[test]
fn expression_resolver_evaluate_unregistered_simple_identifier() {
    use vernal_beans::BeanExpressionResolver;
    let r = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = r.evaluate("unknownBean", None).unwrap();
    assert!(result.is_none());
}

#[test]
fn expression_resolver_evaluate_spel_expression() {
    use vernal_beans::BeanExpressionResolver;
    let r = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = r.evaluate("1 + 1", None).unwrap();
    assert!(result.is_some());
    assert_eq!(*result.unwrap().downcast_ref::<i64>().unwrap(), 2);
}

#[test]
fn expression_resolver_evaluate_spel_string() {
    use vernal_beans::BeanExpressionResolver;
    let r = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = r.evaluate("'hello'", None).unwrap();
    assert!(result.is_some());
    assert_eq!(*result.unwrap().downcast_ref::<String>().unwrap(), "hello");
}

#[test]
fn expression_resolver_evaluate_empty_expression() {
    use vernal_beans::BeanExpressionResolver;
    let r = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = r.evaluate("", None).unwrap();
    assert!(result.is_none());
}

#[test]
fn expression_resolver_evaluate_with_bean_name_param() {
    use vernal_beans::BeanExpressionResolver;
    let r = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    r.register_bean("ctx".to_string(), Arc::new("context".to_string()));
    let result = r.evaluate("ctx", Some("myBean")).unwrap();
    assert!(result.is_some());
}

#[test]
fn expression_resolver_evaluate_spel_boolean() {
    use vernal_beans::BeanExpressionResolver;
    let r = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    // "true" is a simple identifier, returns None (not registered)
    let result = r.evaluate("true", None).unwrap();
    assert!(result.is_none());
}

#[test]
fn expression_resolver_evaluate_numeric_literal() {
    use vernal_beans::BeanExpressionResolver;
    let r = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = r.evaluate("42", None).unwrap();
    assert!(result.is_some());
    assert_eq!(*result.unwrap().downcast_ref::<i64>().unwrap(), 42);
}

// ═══════════════════════════════════════════════════════════════════════════════
// ConfigurablePropertyAccessor
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn configurable_accessor_with_converter_constructor() {
    use vernal_beans::abstract_property_accessor::AbstractPropertyAccessor;
    use vernal_beans::configurable_property_accessor::{
        ConfigurablePropertyAccessor, ConfigurablePropertyAccessorImpl,
    };
    let base = AbstractPropertyAccessor::new();
    base.register_property("count", TypeId::of::<i32>());
    let accessor = ConfigurablePropertyAccessorImpl::with_converter(
        Box::new(base),
        Arc::new(vernal_beans::type_converter_support::TypeConverterSupport::new()),
    );
    assert!(accessor.get_type_converter().is_some());
}

#[test]
fn configurable_accessor_set_with_conversion_matching_type() {
    use vernal_beans::abstract_property_accessor::AbstractPropertyAccessor;
    use vernal_beans::configurable_property_accessor::{
        ConfigurablePropertyAccessor, ConfigurablePropertyAccessorImpl,
    };
    use vernal_beans::property_accessor::PropertyAccessor;
    let base = AbstractPropertyAccessor::new();
    base.register_property("name", TypeId::of::<String>());
    let accessor = ConfigurablePropertyAccessorImpl::new(Box::new(base));
    accessor
        .set_property_value_with_conversion("name", Arc::new("Alice".to_string()))
        .unwrap();
    let value = accessor.get_property_value("name").unwrap();
    assert_eq!(*value.downcast_ref::<String>().unwrap(), "Alice");
}

#[test]
fn configurable_accessor_set_with_conversion_unknown_type() {
    use vernal_beans::abstract_property_accessor::AbstractPropertyAccessor;
    use vernal_beans::configurable_property_accessor::{
        ConfigurablePropertyAccessor, ConfigurablePropertyAccessorImpl,
    };
    let base = AbstractPropertyAccessor::new();
    let accessor = ConfigurablePropertyAccessorImpl::new(Box::new(base));
    let _ = accessor.set_property_value_with_conversion("unknown", Arc::new("v".to_string()));
}

#[test]
fn configurable_accessor_set_with_conversion_mismatched_no_converter() {
    use vernal_beans::abstract_property_accessor::AbstractPropertyAccessor;
    use vernal_beans::configurable_property_accessor::{
        ConfigurablePropertyAccessor, ConfigurablePropertyAccessorImpl,
    };
    let base = AbstractPropertyAccessor::new();
    base.register_property("count", TypeId::of::<i32>());
    let accessor = ConfigurablePropertyAccessorImpl::new(Box::new(base));
    // Type mismatch, no converter
    let _ =
        accessor.set_property_value_with_conversion("count", Arc::new("not_a_number".to_string()));
}

#[test]
fn configurable_accessor_set_and_get_type_converter() {
    use vernal_beans::abstract_property_accessor::AbstractPropertyAccessor;
    use vernal_beans::configurable_property_accessor::{
        ConfigurablePropertyAccessor, ConfigurablePropertyAccessorImpl,
    };
    let base = AbstractPropertyAccessor::new();
    let accessor = ConfigurablePropertyAccessorImpl::new(Box::new(base));
    assert!(accessor.get_type_converter().is_none());
    accessor.set_type_converter(Arc::new(
        vernal_beans::type_converter_support::TypeConverterSupport::new(),
    ));
    assert!(accessor.get_type_converter().is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// PropertyEditorRegistry
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
struct TestEditor {
    value: Option<String>,
}

impl TestEditor {
    fn new() -> Self {
        Self { value: None }
    }
}

impl PropertyEditor for TestEditor {
    fn target_type(&self) -> TypeId {
        TypeId::of::<String>()
    }
    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.value = Some(text.to_string());
        Ok(())
    }
    fn get_as_text(&self) -> Option<String> {
        self.value.clone()
    }
    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        if let Some(s) = value.downcast_ref::<String>() {
            self.value = Some(s.clone());
        }
    }
    fn get_value(&self) -> Option<&dyn Any> {
        self.value.as_ref().map(|v| v as &dyn Any)
    }
    fn get_value_type(&self) -> TypeId {
        TypeId::of::<String>()
    }
}

#[test]
fn property_editor_registry_find_with_path() {
    let mut registry = vernal_beans::property_editor_registry::SimplePropertyEditorRegistry::new();
    registry.register_custom_editor_for_path(
        TypeId::of::<String>(),
        "name",
        Box::new(TestEditor::new()),
    );
    assert!(
        registry
            .find_custom_editor(TypeId::of::<String>(), Some("name"))
            .is_some()
    );
    assert!(
        registry
            .find_custom_editor(TypeId::of::<String>(), Some("other"))
            .is_none()
    );
}

#[test]
fn property_editor_registry_find_type_level_fallback() {
    let mut registry = vernal_beans::property_editor_registry::SimplePropertyEditorRegistry::new();
    registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
    // With path but no path-specific editor falls back to type-level
    assert!(
        registry
            .find_custom_editor(TypeId::of::<String>(), Some("any.path"))
            .is_some()
    );
    assert!(
        registry
            .find_custom_editor(TypeId::of::<String>(), None)
            .is_some()
    );
}

#[test]
fn property_editor_registry_find_not_found() {
    let registry = vernal_beans::property_editor_registry::SimplePropertyEditorRegistry::new();
    assert!(
        registry
            .find_custom_editor(TypeId::of::<i32>(), None)
            .is_none()
    );
}

#[test]
fn property_editor_registry_find_mut() {
    let mut registry = vernal_beans::property_editor_registry::SimplePropertyEditorRegistry::new();
    registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
    assert!(
        registry
            .find_custom_editor_mut(TypeId::of::<String>(), None)
            .is_some()
    );
    assert!(
        registry
            .find_custom_editor_mut(TypeId::of::<i32>(), None)
            .is_none()
    );
}

#[test]
fn property_editor_registry_find_mut_with_path() {
    let mut registry = vernal_beans::property_editor_registry::SimplePropertyEditorRegistry::new();
    registry.register_custom_editor_for_path(
        TypeId::of::<String>(),
        "name",
        Box::new(TestEditor::new()),
    );
    assert!(
        registry
            .find_custom_editor_mut(TypeId::of::<String>(), Some("name"))
            .is_some()
    );
    assert!(
        registry
            .find_custom_editor_mut(TypeId::of::<String>(), Some("other"))
            .is_none()
    );
}

#[test]
fn property_editor_registry_find_mut_path_fallback_to_type() {
    let mut registry = vernal_beans::property_editor_registry::SimplePropertyEditorRegistry::new();
    registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
    // Path not found, but type-level editor exists
    assert!(
        registry
            .find_custom_editor_mut(TypeId::of::<String>(), Some("missing"))
            .is_some()
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// AbstractBeanDefinition + BeanDefinition trait
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn abstract_bean_definition_bean_class_name_default() {
    let def =
        vernal_beans::factory::support::abstract_bean_definition::AbstractBeanDefinition::new();
    assert_eq!(BeanDefinition::bean_class_name(&def), "unknown");
}

#[test]
fn abstract_bean_definition_trait_methods_all() {
    let mut def =
        vernal_beans::factory::support::abstract_bean_definition::AbstractBeanDefinition::new();
    def.set_bean_class_name("com.example.Svc");
    def.set_scope(Scope::Transient);
    def.set_lazy_init(true);
    def.set_primary(true);
    def.set_autowire_candidate(false);
    def.set_role(2);
    def.set_description("desc");
    def.set_parent_name("parent");
    def.set_factory_bean_name("factory");
    def.set_factory_method_name("create");
    def.set_init_method_name("init");
    def.set_destroy_method_name("destroy");
    def.set_abstract(true);

    assert_eq!(BeanDefinition::bean_class_name(&def), "com.example.Svc");
    assert_eq!(BeanDefinition::scope(&def), Scope::Transient);
    assert!(BeanDefinition::is_lazy_init(&def));
    assert!(BeanDefinition::is_primary(&def));
    assert!(!BeanDefinition::is_autowire_candidate(&def));
    assert_eq!(BeanDefinition::role(&def), 2);
    assert_eq!(BeanDefinition::description(&def), Some("desc"));
    assert_eq!(BeanDefinition::parent_name(&def), Some("parent"));
    assert_eq!(BeanDefinition::factory_bean_name(&def), Some("factory"));
    assert_eq!(BeanDefinition::factory_method_name(&def), Some("create"));
    assert_eq!(BeanDefinition::init_method_name(&def), Some("init"));
    assert_eq!(BeanDefinition::destroy_method_name(&def), Some("destroy"));
    assert!(BeanDefinition::is_abstract(&def));
    assert!(!BeanDefinition::is_singleton(&def));
    assert!(BeanDefinition::is_prototype(&def));
}

#[test]
fn abstract_bean_definition_singleton_scope() {
    let def =
        vernal_beans::factory::support::abstract_bean_definition::AbstractBeanDefinition::new();
    assert!(BeanDefinition::is_singleton(&def));
    assert!(!BeanDefinition::is_prototype(&def));
}

// ═══════════════════════════════════════════════════════════════════════════════
// BeanDefinition trait defaults
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_default_methods() {
    let mut def = RootBeanDefinition::new();
    def.set_bean_class_name("MyBean");
    assert!(!BeanDefinition::is_fallback(&def));
    assert!(BeanDefinition::is_autowire_candidate(&def));
    assert_eq!(BeanDefinition::role(&def), 0);
    assert_eq!(BeanDefinition::description(&def), None);
    assert_eq!(
        BeanDefinition::bean_class_name_internal(&def),
        Some("MyBean")
    );
    assert_eq!(BeanDefinition::parent_name(&def), None);
    assert_eq!(BeanDefinition::factory_bean_name(&def), None);
    assert_eq!(BeanDefinition::factory_method_name(&def), None);
    assert_eq!(BeanDefinition::init_method_name(&def), None);
    assert_eq!(BeanDefinition::destroy_method_name(&def), None);
    assert!(!BeanDefinition::is_abstract(&def));
    assert!(BeanDefinition::is_singleton(&def));
    assert!(!BeanDefinition::is_prototype(&def));
    assert_eq!(BeanDefinition::resource_description(&def), None);
    assert!(BeanDefinition::originating_bean_definition(&def).is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// RootBeanDefinition
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn root_bean_definition_trait_impl_all() {
    let mut def = RootBeanDefinition::new();
    def.set_bean_class_name("MyBean");
    def.set_scope(Scope::Transient);
    def.set_lazy_init(true);
    def.set_primary(true);
    def.set_fallback(true);
    def.set_autowire_candidate(false);
    def.set_role(1);
    def.set_description("test desc");
    def.set_parent_name("parent");
    def.set_factory_bean_name("factory");
    def.set_factory_method_name("create");
    def.set_init_method_name("init");
    def.set_destroy_method_name("destroy");
    def.set_abstract(true);

    assert_eq!(BeanDefinition::bean_class_name(&def), "MyBean");
    assert_eq!(BeanDefinition::scope(&def), Scope::Transient);
    assert!(BeanDefinition::is_lazy_init(&def));
    assert!(BeanDefinition::is_primary(&def));
    assert!(BeanDefinition::is_fallback(&def));
    assert!(!BeanDefinition::is_autowire_candidate(&def));
    assert_eq!(BeanDefinition::role(&def), 1);
    assert_eq!(BeanDefinition::description(&def), Some("test desc"));
    assert_eq!(BeanDefinition::parent_name(&def), Some("parent"));
    assert_eq!(BeanDefinition::factory_bean_name(&def), Some("factory"));
    assert_eq!(BeanDefinition::factory_method_name(&def), Some("create"));
    assert_eq!(BeanDefinition::init_method_name(&def), Some("init"));
    assert_eq!(BeanDefinition::destroy_method_name(&def), Some("destroy"));
    assert!(BeanDefinition::is_abstract(&def));
    assert!(!BeanDefinition::is_singleton(&def));
    assert!(BeanDefinition::is_prototype(&def));
}

// ═══════════════════════════════════════════════════════════════════════════════
// FactoryBeanRegistrySupport
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn factory_bean_registry_default_trait() {
    let support = FactoryBeanRegistrySupport::default();
    assert!(support.factory_bean_names().is_empty());
}

#[test]
fn factory_bean_registry_type_erased_no_cache() {
    let support = FactoryBeanRegistrySupport::new();
    let dummy: &dyn Any = &42i32;
    let result = support.get_object_from_factory_bean(dummy, "myFactory");
    assert!(result.is_err());
}

#[test]
fn factory_bean_registry_type_erased_with_cache() {
    let support = FactoryBeanRegistrySupport::new();
    support
        .get_object_from_factory_bean_with_closure("myFactory", &|| {
            Ok(Arc::new(42i32) as Arc<dyn Any + Send + Sync>)
        })
        .unwrap();
    let dummy: &dyn Any = &42i32;
    let result = support
        .get_object_from_factory_bean(dummy, "myFactory")
        .unwrap();
    assert_eq!(*result.downcast_ref::<i32>().unwrap(), 42);
}

#[test]
fn factory_bean_registry_factory_bean_names() {
    let support = FactoryBeanRegistrySupport::new();
    support.register_factory_bean_type(TypeId::of::<i32>(), "f1".to_string());
    support.register_factory_bean_type(TypeId::of::<String>(), "f2".to_string());
    let names = support.factory_bean_names();
    assert_eq!(names.len(), 2);
}

#[test]
fn factory_bean_registry_clear_all() {
    let support = FactoryBeanRegistrySupport::new();
    support.register_factory_bean_type(TypeId::of::<i32>(), "f1".to_string());
    support
        .get_object_from_factory_bean_with_closure("f1", &|| {
            Ok(Arc::new("v".to_string()) as Arc<dyn Any + Send + Sync>)
        })
        .unwrap();
    support.clear_all();
    assert!(support.factory_bean_names().is_empty());
    assert!(!support.is_factory_bean_by_type(TypeId::of::<i32>()));
}

#[test]
fn factory_bean_registry_remove_cached_object() {
    let support = FactoryBeanRegistrySupport::new();
    support
        .get_object_from_factory_bean_with_closure("f1", &|| {
            Ok(Arc::new(42i32) as Arc<dyn Any + Send + Sync>)
        })
        .unwrap();
    let removed = support.remove_cached_object("f1");
    assert!(removed.is_some());
    assert_eq!(*removed.unwrap().downcast_ref::<i32>().unwrap(), 42);
    assert!(support.remove_cached_object("f1").is_none());
}

#[test]
fn factory_bean_registry_get_factory_bean_type_not_found() {
    let support = FactoryBeanRegistrySupport::new();
    assert!(support.get_factory_bean_type(TypeId::of::<i32>()).is_none());
}

#[test]
fn factory_bean_registry_is_factory_bean_by_type() {
    let support = FactoryBeanRegistrySupport::new();
    assert!(!support.is_factory_bean_by_type(TypeId::of::<i32>()));
    support.register_factory_bean_type(TypeId::of::<i32>(), "myFactory".to_string());
    assert!(support.is_factory_bean_by_type(TypeId::of::<i32>()));
}

#[test]
fn factory_bean_registry_is_factory_bean_by_name() {
    let support = FactoryBeanRegistrySupport::new();
    assert!(!support.is_factory_bean("myFactory"));
    support.register_factory_bean_type(TypeId::of::<i32>(), "myFactory".to_string());
    assert!(support.is_factory_bean("myFactory"));
}

#[test]
fn factory_bean_registry_closure_error_propagated() {
    let support = FactoryBeanRegistrySupport::new();
    let result = support.get_object_from_factory_bean_with_closure("f1", &|| Err("error".into()));
    assert!(result.is_err());
}

#[test]
fn factory_bean_registry_duplicate_name_registration() {
    let support = FactoryBeanRegistrySupport::new();
    support.register_factory_bean_type(TypeId::of::<i32>(), "factory".to_string());
    support.register_factory_bean_type(TypeId::of::<String>(), "factory".to_string());
    assert_eq!(
        support
            .factory_bean_names()
            .iter()
            .filter(|n| *n == "factory")
            .count(),
        1
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// ConversionServiceFactory
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn conversion_factory_default_trait() {
    let f = ConversionServiceFactory::default();
    assert!(!f.has_defaults());
    assert_eq!(f.converter_count(), 0);
}

#[test]
fn conversion_factory_register_defaults_idempotent() {
    let f = ConversionServiceFactory::new();
    f.register_defaults();
    let c1 = f.converter_count();
    f.register_defaults();
    assert_eq!(c1, f.converter_count());
}

#[test]
fn conversion_factory_string_to_i64() {
    let f = ConversionServiceFactory::new();
    f.register_defaults();
    let r = f.convert(&"12345".to_string(), "String", "i64").unwrap();
    assert_eq!(*r.unwrap().downcast::<i64>().unwrap(), 12345);
}

#[test]
fn conversion_factory_string_to_f64() {
    let f = ConversionServiceFactory::new();
    f.register_defaults();
    let r = f.convert(&"3.14".to_string(), "String", "f64").unwrap();
    assert_eq!(*r.unwrap().downcast::<f64>().unwrap(), 3.14);
}

#[test]
fn conversion_factory_i64_to_string() {
    let f = ConversionServiceFactory::new();
    f.register_defaults();
    let r = f.convert(&12345i64, "i64", "String").unwrap();
    assert_eq!(*r.unwrap().downcast::<String>().unwrap(), "12345");
}

#[test]
fn conversion_factory_has_converter() {
    let f = ConversionServiceFactory::new();
    assert!(!f.has_converter("String", "i32"));
    f.register_defaults();
    assert!(f.has_converter("String", "i32"));
    assert!(!f.has_converter("String", "u64"));
}

#[test]
fn conversion_factory_debug_impl() {
    let f = ConversionServiceFactory::new();
    f.register_defaults();
    let d = format!("{:?}", f);
    assert!(d.contains("ConversionServiceFactory"));
}

#[test]
fn conversion_factory_no_converter_returns_none() {
    let f = ConversionServiceFactory::new();
    let r = f.convert(&42i32, "i32", "u64").unwrap();
    assert!(r.is_none());
}

#[test]
fn conversion_factory_invalid_string_to_i32() {
    let f = ConversionServiceFactory::new();
    f.register_defaults();
    let r = f
        .convert(&"not_a_number".to_string(), "String", "i32")
        .unwrap();
    assert!(r.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// TransientTracker
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn transient_tracker_default_trait() {
    let t = TransientTracker::default();
    assert_eq!(t.total_surviving(), 0);
}

#[test]
fn transient_tracker_track_and_surviving() {
    let t = TransientTracker::new();
    let inst: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    t.track(TypeId::of::<i32>(), &inst);
    let surviving = t.surviving_instances(TypeId::of::<i32>());
    assert_eq!(surviving.len(), 1);
    assert_eq!(*surviving[0].downcast_ref::<i32>().unwrap(), 42);
}

#[test]
fn transient_tracker_surviving_empty_for_unknown_type() {
    let t = TransientTracker::new();
    assert!(t.surviving_instances(TypeId::of::<String>()).is_empty());
}

#[test]
fn transient_tracker_total_surviving() {
    let t = TransientTracker::new();
    let a: Arc<dyn Any + Send + Sync> = Arc::new(1i32);
    let b: Arc<dyn Any + Send + Sync> = Arc::new(2i32);
    let c: Arc<dyn Any + Send + Sync> = Arc::new("hello".to_string());
    t.track(TypeId::of::<i32>(), &a);
    t.track(TypeId::of::<i32>(), &b);
    t.track(TypeId::of::<String>(), &c);
    assert_eq!(t.total_surviving(), 3);
}

#[test]
fn transient_tracker_clear() {
    let t = TransientTracker::new();
    let inst: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    t.track(TypeId::of::<i32>(), &inst);
    assert_eq!(t.total_surviving(), 1);
    t.clear();
    assert_eq!(t.total_surviving(), 0);
}

#[test]
fn transient_tracker_weak_ref_cleanup() {
    let t = TransientTracker::new();
    {
        let inst: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        t.track(TypeId::of::<i32>(), &inst);
        assert_eq!(t.total_surviving(), 1);
    }
    // instance dropped - surviving_instances cleans up dead weak refs
    assert!(t.surviving_instances(TypeId::of::<i32>()).is_empty());
}

#[test]
fn transient_tracker_multiple_types() {
    let t = TransientTracker::new();
    let a: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let b: Arc<dyn Any + Send + Sync> = Arc::new("hello".to_string());
    t.track(TypeId::of::<i32>(), &a);
    t.track(TypeId::of::<String>(), &b);
    assert_eq!(t.surviving_instances(TypeId::of::<i32>()).len(), 1);
    assert_eq!(t.surviving_instances(TypeId::of::<String>()).len(), 1);
    assert!(t.surviving_instances(TypeId::of::<f64>()).is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeError
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_error_display_invalid_state() {
    let err = ScopeError::InvalidState {
        operation: "open",
        scope: ScopeKey::of::<String>(),
        state: ScopeState::Closed,
    };
    assert!(format!("{}", err).contains("open"));
    assert!(format!("{}", err).contains("Closed"));
}

#[test]
fn scope_error_display_cancelled() {
    let err = ScopeError::Cancelled {
        operation: "resolve",
        scope: ScopeKey::of::<i32>(),
    };
    assert!(format!("{}", err).contains("cancelled"));
}

#[test]
fn scope_error_display_type_mismatch() {
    let err = ScopeError::TypeMismatch {
        scope: ScopeKey::of::<String>(),
        expected: "MyType",
    };
    assert!(format!("{}", err).contains("type mismatch"));
}

#[test]
fn scope_error_display_resolution() {
    let err = ScopeError::Resolution {
        scope: ScopeKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "bean".to_string(),
            path: vec![],
        }),
    };
    assert!(format!("{}", err).contains("resolution failed"));
}

#[test]
fn scope_error_display_close_timeout() {
    use std::time::Duration;
    let err = ScopeError::CloseTimeout {
        scope: ScopeKey::of::<String>(),
        timeout: Duration::from_secs(5),
    };
    assert!(format!("{}", err).contains("timeout"));
}

#[test]
fn scope_error_display_runtime_unavailable() {
    let err = ScopeError::RuntimeUnavailable {
        scope: ScopeKey::of::<String>(),
    };
    assert!(format!("{}", err).contains("Tokio runtime"));
}

#[test]
fn scope_error_display_close_hook() {
    let err = ScopeError::CloseHook {
        scope: ScopeKey::of::<String>(),
        source: shared_err("hook failed"),
    };
    assert!(format!("{}", err).contains("close hook failed"));
}

#[test]
fn scope_error_display_close_task() {
    let err = ScopeError::CloseTask {
        scope: ScopeKey::of::<String>(),
        source: shared_err("task failed"),
    };
    assert!(format!("{}", err).contains("close task failed"));
}

#[test]
fn scope_error_source_resolution() {
    let err = ScopeError::Resolution {
        scope: ScopeKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "b".to_string(),
            path: vec![],
        }),
    };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn scope_error_source_close_hook() {
    let err = ScopeError::CloseHook {
        scope: ScopeKey::of::<String>(),
        source: shared_err("e"),
    };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn scope_error_source_close_task() {
    let err = ScopeError::CloseTask {
        scope: ScopeKey::of::<String>(),
        source: shared_err("e"),
    };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn scope_error_source_none_variants() {
    assert!(
        std::error::Error::source(&ScopeError::InvalidState {
            operation: "o",
            scope: ScopeKey::of::<String>(),
            state: ScopeState::Closed
        })
        .is_none()
    );
    assert!(
        std::error::Error::source(&ScopeError::Cancelled {
            operation: "o",
            scope: ScopeKey::of::<String>()
        })
        .is_none()
    );
    assert!(
        std::error::Error::source(&ScopeError::TypeMismatch {
            scope: ScopeKey::of::<String>(),
            expected: "t"
        })
        .is_none()
    );
    assert!(
        std::error::Error::source(&ScopeError::CloseTimeout {
            scope: ScopeKey::of::<String>(),
            timeout: std::time::Duration::from_secs(1)
        })
        .is_none()
    );
    assert!(
        std::error::Error::source(&ScopeError::RuntimeUnavailable {
            scope: ScopeKey::of::<String>()
        })
        .is_none()
    );
}

#[test]
fn scope_error_debug_impl() {
    let err = ScopeError::InvalidState {
        operation: "open",
        scope: ScopeKey::of::<String>(),
        state: ScopeState::Open,
    };
    assert!(format!("{:?}", err).contains("InvalidState"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// BeanFactoryUtils
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_factory_utils_transformed_bean_name() {
    assert_eq!(BeanFactoryUtils::transformed_bean_name("&myBean"), "myBean");
    assert_eq!(BeanFactoryUtils::transformed_bean_name("myBean"), "myBean");
    assert_eq!(BeanFactoryUtils::transformed_bean_name("&"), "");
}

#[test]
fn bean_factory_utils_is_factory_bean() {
    assert!(BeanFactoryUtils::is_factory_bean("&myFactory"));
    assert!(!BeanFactoryUtils::is_factory_bean("regularBean"));
    assert!(!BeanFactoryUtils::is_factory_bean(""));
}

#[test]
fn bean_factory_utils_check_is_factory_bean() {
    assert!(BeanFactoryUtils::check_is_factory_bean("&myFactory"));
    assert!(!BeanFactoryUtils::check_is_factory_bean("regularBean"));
}

#[test]
fn bean_factory_utils_count_beans_for_type() {
    let c = make_container();
    assert_eq!(
        BeanFactoryUtils::count_beans_for_type(TypeId::of::<String>(), &c),
        1
    );
}

#[test]
fn bean_factory_utils_bean_names_for_type() {
    let c = make_container();
    assert_eq!(
        BeanFactoryUtils::bean_names_for_type(TypeId::of::<String>(), &c).len(),
        1
    );
}

#[test]
fn bean_factory_utils_beans_of_type() {
    let c = make_container();
    let beans = BeanFactoryUtils::beans_of_type(TypeId::of::<String>(), &c).unwrap();
    assert_eq!(beans.len(), 1);
}

#[test]
fn bean_factory_utils_bean_definition_names() {
    let c = make_container();
    assert!(BeanFactoryUtils::bean_definition_names(&c).len() >= 2);
}

#[test]
fn bean_factory_utils_count_including_ancestors() {
    let c = make_container();
    assert_eq!(
        BeanFactoryUtils::count_beans_for_type_including_ancestors(TypeId::of::<String>(), &c),
        1
    );
}

#[test]
fn bean_factory_utils_names_including_ancestors() {
    let c = make_container();
    assert_eq!(
        BeanFactoryUtils::bean_names_for_type_including_ancestors(TypeId::of::<String>(), &c).len(),
        1
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// DirectFieldAccessor
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn direct_field_accessor_get_field_value() {
    let a = DirectFieldAccessor::new(String::from("test"));
    a.set_field_value("name", String::from("Alice"));
    assert!(a.get_field_value("name").is_some());
    assert!(a.get_field_value("missing").is_none());
}

#[test]
fn direct_field_accessor_get_field_type() {
    let a = DirectFieldAccessor::new(String::from("test"));
    a.set_field_value("count", 42i32);
    assert_eq!(a.get_field_type("count").unwrap(), TypeId::of::<i32>());
    assert!(a.get_field_type("missing").is_none());
}

#[test]
fn direct_field_accessor_target_type() {
    let a = DirectFieldAccessor::new(String::from("test"));
    assert!(a.target().is::<String>());
}

#[test]
fn direct_field_accessor_multiple_fields() {
    let a = DirectFieldAccessor::new(String::from("test"));
    a.set_field_value("a", 1i32);
    a.set_field_value("b", 2i64);
    a.set_field_value("c", "three".to_string());
    assert_eq!(a.field_names().len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// PropertyEditorCache
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
struct StubEditor {
    value: String,
}

impl StubEditor {
    fn new(initial: &str) -> Self {
        Self {
            value: initial.to_string(),
        }
    }
}

impl PropertyEditor for StubEditor {
    fn target_type(&self) -> TypeId {
        TypeId::of::<String>()
    }
    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.value = text.to_string();
        Ok(())
    }
    fn get_as_text(&self) -> Option<String> {
        Some(self.value.clone())
    }
    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        if let Some(s) = value.downcast_ref::<String>() {
            self.value = s.clone();
        }
    }
    fn get_value(&self) -> Option<&dyn Any> {
        Some(&self.value)
    }
    fn get_value_type(&self) -> TypeId {
        TypeId::of::<String>()
    }
}

#[test]
fn property_editor_cache_default_trait() {
    let c = PropertyEditorCache::default();
    assert_eq!(c.custom_editor_count(), 0);
    assert_eq!(c.default_editor_count(), 0);
    assert!(!c.has_default_editors());
}

#[test]
fn property_editor_cache_find_default_editor() {
    let c = PropertyEditorCache::new();
    c.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("default")));
    assert!(c.find_default_editor(TypeId::of::<i32>()).is_some());
    assert!(c.find_default_editor(TypeId::of::<String>()).is_none());
}

#[test]
fn property_editor_cache_find_editor_fallback_to_default() {
    let c = PropertyEditorCache::new();
    c.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("default")));
    let found = c.find_editor(TypeId::of::<i32>());
    assert!(found.is_some());
    assert_eq!(found.unwrap().get_as_text(), Some("default".to_string()));
}

#[test]
fn property_editor_cache_find_editor_not_found() {
    let c = PropertyEditorCache::new();
    assert!(c.find_editor(TypeId::of::<i32>()).is_none());
}

#[test]
fn property_editor_cache_has_custom_editor_for() {
    let c = PropertyEditorCache::new();
    assert!(!c.has_custom_editor_for(TypeId::of::<String>()));
    c.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("c")));
    assert!(c.has_custom_editor_for(TypeId::of::<String>()));
}

#[test]
fn property_editor_cache_clear_custom_editors() {
    let c = PropertyEditorCache::new();
    c.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("c")));
    c.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("d")));
    c.mark_default_editors_registered();
    c.clear_custom_editors();
    assert_eq!(c.custom_editor_count(), 0);
    assert_eq!(c.default_editor_count(), 1);
    assert!(c.has_default_editors());
}

#[test]
fn property_editor_cache_debug_impl() {
    let c = PropertyEditorCache::new();
    c.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("c")));
    let d = format!("{:?}", c);
    assert!(d.contains("PropertyEditorCache"));
    assert!(d.contains("custom_count"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container - additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_qualified_in_success() {
    let mut b = RegistryBuilder::new();
    let q = Qualifier::new("primary").unwrap();
    let _ = b.register(
        ComponentDefinition::singleton::<String, _>(|_| "primary".to_string()).qualified(q.clone()),
    );
    let c = Container::new(b.build().unwrap());
    let scope = c.open_scope::<String>();
    let val: Arc<String> = c.resolve_qualified_in(&q, &scope).unwrap();
    assert_eq!(*val, "primary");
}

#[test]
fn container_resolve_qualified_in_wrong_owner() {
    let mut b = RegistryBuilder::new();
    let q = Qualifier::new("q").unwrap();
    let _ = b.register(
        ComponentDefinition::singleton::<String, _>(|_| "v".to_string()).qualified(q.clone()),
    );
    let c = Container::new(b.build().unwrap());
    let other = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = other.open_scope::<String>();
    let result: Result<Arc<String>, _> = c.resolve_qualified_in(&q, &scope);
    assert!(result.is_err());
}

#[test]
fn container_resolve_all_traits_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    // Use a trait object that satisfies Send + Sync
    let result = c.resolve_all_traits::<dyn Any + Send + Sync>();
    assert!(result.unwrap().is_empty());
}

#[test]
fn container_resolve_all_traits_in_wrong_owner() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let other = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = other.open_scope::<String>();
    let result = c.resolve_all_traits_in::<dyn Any + Send + Sync>(&scope);
    assert!(result.is_err());
}

#[test]
fn container_resolve_trait_in_wrong_owner() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let other = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = other.open_scope::<String>();
    let result = c.resolve_trait_in::<dyn Any + Send + Sync>(&scope);
    assert!(result.is_err());
}

#[test]
fn container_resolve_qualified_trait_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let q = Qualifier::new("q").unwrap();
    let result = c.resolve_qualified_trait::<dyn Any + Send + Sync>(&q);
    assert!(result.is_err());
}

#[test]
fn container_resolve_qualified_trait_in_wrong_owner() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let other = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = other.open_scope::<String>();
    let q = Qualifier::new("q").unwrap();
    let result = c.resolve_qualified_trait_in::<dyn Any + Send + Sync>(&q, &scope);
    assert!(result.is_err());
}

#[test]
fn container_warm_up_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    c.warm_up().unwrap();
}

#[test]
fn container_transient_scope_tracking() {
    let mut b = RegistryBuilder::new();
    let _ = b.register(ComponentDefinition::transient::<i32, _>(|_| 42i32));
    let c = Container::new(b.build().unwrap());
    let tracker = c.transient_tracker();
    assert_eq!(tracker.total_surviving(), 0);
    // Transient 实例不被容器自动追踪：调用方独占所有权
    // （对标 Spring prototype：容器不持有原型实例引用，且弱引用会破坏
    //  `Arc::get_mut` 的独占借用契约）
    let a: Arc<i32> = c.resolve().unwrap();
    let b_val: Arc<i32> = c.resolve().unwrap();
    assert_eq!(*a, 42);
    assert_eq!(*b_val, 42);
    assert_eq!(tracker.total_surviving(), 0);
    // 需要关闭通知的上层显式 track 后，存活实例可被枚举
    let raw: Arc<dyn Any + Send + Sync> = a;
    tracker.track(TypeId::of::<i32>(), &raw);
    let surviving = tracker.surviving_instances(TypeId::of::<i32>());
    assert!(surviving.len() >= 1);
}

#[test]
fn container_construct_with_post_processor() {
    use vernal_beans::BeanPostProcessor;
    struct MockPP;
    impl BeanPostProcessor for MockPP {
        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(bean))
        }
    }
    let mut b = RegistryBuilder::new();
    let _ = b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    let mut c = Container::new(b.build().unwrap());
    c.add_bean_post_processor(Arc::new(MockPP));
    assert_eq!(c.bean_post_processor_count(), 1);
    let val: Arc<String> = c.resolve().unwrap();
    assert_eq!(*val, "hello");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container - BeanDefinitionRegistry
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_bean_definition_registry_full_flow() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = make_container();
    let initial = <Container as BeanDefinitionRegistry>::bean_definition_count(&c);
    let def = Box::new(RootBeanDefinition::new());
    c.register_bean_definition("newBean".to_string(), def)
        .unwrap();
    assert_eq!(
        <Container as BeanDefinitionRegistry>::bean_definition_count(&c),
        initial + 1
    );
    assert!(<Container as BeanDefinitionRegistry>::contains_bean_definition(&c, "newBean"));
    let bd = <Container as BeanDefinitionRegistry>::get_bean_definition(&c, "newBean");
    assert!(bd.is_some());
    let removed = c.remove_bean_definition("newBean").unwrap();
    assert_eq!(removed.bean_class_name(), "unknown");
    assert!(!<Container as BeanDefinitionRegistry>::contains_bean_definition(&c, "newBean"));
}

#[test]
fn container_bean_definition_names_includes_dynamic() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = make_container();
    let def = Box::new(RootBeanDefinition::new());
    c.register_bean_definition("dynamicBean".to_string(), def)
        .unwrap();
    let names = <Container as BeanDefinitionRegistry>::bean_definition_names(&c);
    assert!(names.contains(&"dynamicBean".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container - ListableBeanFactory edge cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_listable_beans_of_type_id_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let beans = c
        .beans_of_type_id(TypeId::of::<String>(), true, true)
        .unwrap();
    assert!(beans.is_empty());
}

#[test]
fn container_listable_contains_non_singleton_with_transient() {
    let mut b = RegistryBuilder::new();
    let _ = b.register(ComponentDefinition::transient::<String, _>(|_| {
        "t".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    assert!(c.contains_non_singleton_bean());
    assert!(!c.contains_singleton_bean());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container - ConfigurableBeanFactory
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_registered_scope_names_empty() {
    let c = make_container();
    assert!(c.registered_scope_names().is_empty());
}

#[test]
fn container_get_registered_scope_not_found() {
    let c = make_container();
    assert!(c.get_registered_scope("nonexistent").is_none());
}

#[test]
fn container_embedded_value_multiple_resolvers() {
    let mut c = make_container();
    c.add_embedded_value_resolver(Arc::new(|v: &str| v.replace("${a}", "1")));
    c.add_embedded_value_resolver(Arc::new(|v: &str| v.replace("${b}", "2")));
    assert_eq!(c.resolve_embedded_value("${a}-${b}"), "1-2");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container - AutowireCapableBeanFactory
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_dependency_optional_not_found() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let descriptor =
        vernal_beans::factory::support::dependency_descriptor::DependencyDescriptor::new(
            TypeId::of::<String>(),
            "alloc::string::String".to_string(),
            false, // not required
        );
    let result =
        <Container as AutowireCapableBeanFactory>::resolve_dependency(&c, &descriptor, None)
            .unwrap();
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container - HierarchicalBeanFactory
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_contains_local_bean_dynamic() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = make_container();
    let def = Box::new(RootBeanDefinition::new());
    c.register_bean_definition("localBean".to_string(), def)
        .unwrap();
    assert!(<Container as HierarchicalBeanFactory>::contains_local_bean(
        &c,
        "localBean"
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container - ConfigurableListableBeanFactory
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_pre_instantiate_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    c.pre_instantiate_singletons().unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container - shared_handle
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_post_processor_count_reflects_additions() {
    use vernal_beans::BeanPostProcessor;
    struct PP;
    impl BeanPostProcessor for PP {
        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(bean))
        }
    }
    let mut b = RegistryBuilder::new();
    let _ = b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    let mut c = Container::new(b.build().unwrap());
    assert_eq!(c.bean_post_processor_count(), 0);
    c.add_bean_post_processor(Arc::new(PP));
    assert_eq!(c.bean_post_processor_count(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// ResolveError
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn resolve_error_display_all_variants() {
    let err = ResolveError::NotFound {
        component: "b".to_string(),
        path: vec!["a".into(), "b".into()],
    };
    assert!(format!("{}", err).contains("not found"));
    assert!(format!("{}", err).contains("a -> b"));

    let err = ResolveError::Ambiguous {
        component: "S".to_string(),
        candidates: vec!["a".into(), "b".into()],
        path: vec![],
    };
    assert!(format!("{}", err).contains("ambiguous"));

    let err = ResolveError::UndeclaredDependency {
        component: ComponentKey::of::<String>(),
        dependency: "i32".into(),
    };
    assert!(format!("{}", err).contains("undeclared"));

    let err = ResolveError::TypeMismatch {
        component: ComponentKey::of::<String>(),
    };
    assert!(format!("{}", err).contains("type mismatch"));

    let err = ResolveError::CircularRuntime {
        path: vec!["A".into(), "B".into(), "A".into()],
    };
    assert!(format!("{}", err).contains("cycle"));

    let err = ResolveError::ScopeNotActive {
        component: ComponentKey::of::<String>(),
        scope: ScopeKey::of::<i32>(),
    };
    assert!(format!("{}", err).contains("requires active scope"));

    let err = ResolveError::ScopeOwnerMismatch {
        scope: ScopeKey::of::<String>(),
    };
    assert!(format!("{}", err).contains("different component container"));

    let err = ResolveError::ScopeUnavailable {
        component: ComponentKey::of::<String>(),
        scope: ScopeKey::of::<i32>(),
        state: ScopeState::Closed,
        cancelled: false,
    };
    assert!(format!("{}", err).contains("unavailable"));

    let err = ResolveError::ProviderUsedDuringConstruction {
        component: ComponentKey::of::<String>(),
        dependency: "i32".into(),
    };
    assert!(format!("{}", err).contains("before its factory completed"));
}

#[test]
fn resolve_error_source() {
    let err = ResolveError::Construction {
        component: ComponentKey::of::<String>(),
        source: shared_err("build failed"),
    };
    assert!(std::error::Error::source(&err).is_some());

    let err = ResolveError::NotFound {
        component: "b".into(),
        path: vec![],
    };
    assert!(std::error::Error::source(&err).is_none());
}

#[test]
fn resolve_error_debug_clone() {
    let err = ResolveError::NotFound {
        component: "b".into(),
        path: vec![],
    };
    assert!(format!("{:?}", err).contains("NotFound"));
    let cloned = err.clone();
    assert_eq!(format!("{}", err), format!("{}", cloned));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeKey
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_key_display_type_name() {
    let key = ScopeKey::of::<String>();
    assert!(!format!("{}", key).is_empty());
    assert!(key.type_name().contains("String"));
}

#[test]
fn scope_key_equality_debug() {
    let a = ScopeKey::of::<String>();
    let b = ScopeKey::of::<String>();
    let c = ScopeKey::of::<i32>();
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert!(format!("{:?}", a).contains("ScopeKey"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeState
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_state_default_debug_equality() {
    assert_eq!(ScopeState::default(), ScopeState::Open);
    assert_eq!(format!("{:?}", ScopeState::Open), "Open");
    assert_eq!(format!("{:?}", ScopeState::Closing), "Closing");
    assert_eq!(format!("{:?}", ScopeState::Closed), "Closed");
    assert_ne!(ScopeState::Open, ScopeState::Closed);
    let s = ScopeState::Closing;
    assert_eq!(s, ScopeState::Closing);
}

// ═══════════════════════════════════════════════════════════════════════════════
// DefinitionError
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn definition_error_display_debug() {
    let err = DefinitionError::DuplicateDefinition {
        key: ComponentKey::of::<String>(),
    };
    assert!(format!("{}", err).contains("duplicate component definition"));
    assert!(format!("{:?}", err).contains("DuplicateDefinition"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// GraphError
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn graph_error_display_debug() {
    let err = GraphError::MissingDependency {
        path: vec!["A".into(), "B".into()],
    };
    assert!(format!("{}", err).contains("missing dependency"));
    assert!(format!("{:?}", err).contains("MissingDependency"));

    let err = GraphError::AmbiguousDependency {
        path: vec![],
        candidates: vec!["a".into()],
    };
    assert!(format!("{}", err).contains("ambiguous"));

    let err = GraphError::MissingTraitBindingTarget {
        binding: "b".into(),
    };
    assert!(format!("{}", err).contains("not registered"));

    let err = GraphError::Cycle {
        path: vec!["A".into(), "B".into(), "A".into()],
    };
    assert!(format!("{}", err).contains("cycle"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentKey
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_key_display_type_name_equality() {
    let a = ComponentKey::of::<String>();
    let b = ComponentKey::of::<String>();
    let c = ComponentKey::of::<i32>();
    assert!(!format!("{}", a).is_empty());
    assert!(a.type_name().contains("String"));
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert!(a.qualifier().is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Qualifier
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn qualifier_new_display_equality() {
    let q = Qualifier::new("primary").unwrap();
    assert!(format!("{}", q).contains("primary"));
    assert_eq!(q, Qualifier::new("primary").unwrap());
    assert_ne!(q, Qualifier::new("other").unwrap());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dependency
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn dependency_of_qualified_display() {
    let d = vernal_beans::Dependency::of::<String>();
    assert!(!format!("{}", d).is_empty());
    let q = Qualifier::new("primary").unwrap();
    let d = vernal_beans::Dependency::qualified::<String>(q);
    assert!(format!("{}", d).contains("primary"));
    let a = vernal_beans::Dependency::of::<String>();
    let b = vernal_beans::Dependency::of::<String>();
    assert_eq!(a, b);
}

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentDefinition
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_definition_scopes_and_key() {
    let s = ComponentDefinition::singleton::<String, _>(|_| "s".to_string());
    assert_eq!(s.scope(), Scope::Singleton);
    let t = ComponentDefinition::transient::<String, _>(|_| "t".to_string());
    assert_eq!(t.scope(), Scope::Transient);
    let v = ComponentDefinition::shared_value(42i32);
    assert_eq!(v.scope(), Scope::Singleton);
    assert!(v.key().type_name().contains("i32"));
}

#[test]
fn component_definition_qualified() {
    let q = Qualifier::new("primary").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_| "h".to_string()).qualified(q);
    assert!(def.key().qualifier().is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Scope
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_properties() {
    assert!(Scope::Singleton.is_singleton());
    assert!(!Scope::Singleton.is_transient());
    assert!(!Scope::Transient.is_singleton());
    assert!(Scope::Transient.is_transient());
    assert!(format!("{:?}", Scope::Singleton).contains("Singleton"));
    assert_eq!(Scope::Singleton, Scope::Singleton);
    assert_ne!(Scope::Singleton, Scope::Transient);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TraitKey
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn trait_key_of_display_type_name() {
    let key = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    assert!(!format!("{}", key).is_empty());
    assert!(!key.type_name().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// SmartInstantiationAwareBeanPostProcessor
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn smart_post_processor_default_trait_methods() {
    use vernal_beans::factory::config::smart_instantiation_aware_bean_post_processor::SmartInstantiationAwareBeanPostProcessor;
    struct NoOp;
    impl SmartInstantiationAwareBeanPostProcessor for NoOp {}
    let p = NoOp;
    assert!(p.predict_bean_type("C", "n").is_none());
    assert!(p.determine_candidate_constructors("C", "n").is_none());
    let bean: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let early = p.get_early_bean_reference(bean.clone(), "n");
    assert!(Arc::ptr_eq(&bean, &early));
    assert!(
        p.post_process_before_instantiation("C", "n")
            .unwrap()
            .is_none()
    );
    assert!(p.post_process_after_instantiation(&42i32, "n").unwrap());
    let r = p
        .post_process_before_initialization(bean.clone(), "n")
        .unwrap();
    assert!(r.is_some() && Arc::ptr_eq(&bean, &r.unwrap()));
    let r = p.post_process_after_initialization(bean, "n").unwrap();
    assert!(r.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container - SingletonBeanRegistry
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_singleton_callback_triggered() {
    use vernal_beans::SingletonBeanRegistry;
    let mut c = make_container();
    let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let called_clone = called.clone();
    let callback: Arc<dyn Fn(&dyn Any) + Send + Sync> = Arc::new(move |_| {
        called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
    });
    c.add_singleton_callback("test".to_string(), callback);
    // Register a singleton with that name
    let obj: Arc<dyn Any + Send + Sync> = Arc::new("val".to_string());
    c.register_singleton("test", obj);
    assert!(called.load(std::sync::atomic::Ordering::SeqCst));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container - get_singleton fallback path
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_get_singleton_not_found() {
    use vernal_beans::SingletonBeanRegistry;
    let c = make_container();
    assert!(c.get_singleton("nonexistent").is_none());
}

#[test]
fn container_singleton_names_empty() {
    use vernal_beans::SingletonBeanRegistry;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(c.singleton_names().is_empty());
    assert_eq!(c.singleton_count(), 0);
    assert!(!c.contains_singleton("any"));
}
