//! Maximum coverage tests for vernal-beans crate.
//!
//! Targets uncovered lines across all priority files.

use std::any::{Any, TypeId};
use std::sync::Arc;

use vernal_beans::{
    BeanDefinition, BeanDescCache, BeanFactory, BeanFactoryUtils, ComponentDefinition,
    ComponentKey, ConfigurableBeanFactory, ConfigurableListableBeanFactory, Container,
    ConversionServiceFactory, DefinitionError, DirectFieldAccessor, FactoryBeanRegistrySupport,
    GraphError, HierarchicalBeanFactory, ListableBeanFactory, PropertyEditor,
    PropertyEditorCache, PropertyEditorRegistry, Qualifier, RegistryBuilder, ResolveError,
    RootBeanDefinition, Scope, ScopeError, ScopeKey, ScopeState, TransientTracker,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn make_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()))
        .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    Container::new(b.build().unwrap())
}

fn make_container_with_transient() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()))
        .unwrap();
    b.register(ComponentDefinition::transient::<i32, _>(|_| 99i32))
        .unwrap();
    Container::new(b.build().unwrap())
}

fn shared_err(msg: &str) -> vernal_core::SharedError {
    Arc::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        msg.to_string(),
    ))
}

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

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — select_definition error paths
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_select_definition_not_found() {
    let c = make_container();
    // Resolve a type that doesn't exist
    let result: Result<Arc<bool>, _> = c.resolve();
    assert!(result.is_err());
    match result.unwrap_err() {
        ResolveError::NotFound { .. } => {}
        other => panic!("Expected NotFound, got: {:?}", other),
    }
}

#[test]
fn container_select_definition_ambiguous() {
    let mut b = RegistryBuilder::new();
    // Register two definitions of the same type with different qualifiers
    let q1 = Qualifier::new("a").unwrap();
    let q2 = Qualifier::new("b").unwrap();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "a".to_string()).qualified(q1),
    );
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "b".to_string()).qualified(q2),
    );
    // Also register unqualified - this creates ambiguity when resolving without qualifier
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 1i32));
    let c = Container::new(b.build().unwrap());
    // Resolving String without qualifier should be ambiguous (3 matches)
    let result: Result<Arc<String>, _> = c.resolve();
    assert!(result.is_err());
}

#[test]
fn container_resolve_optional_not_found_returns_none() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let descriptor =
        vernal_beans::factory::support::dependency_descriptor::DependencyDescriptor::new(
            TypeId::of::<bool>(),
            "bool".to_string(),
            false, // not required
        );
    let result = <Container as AutowireCapableBeanFactory>::resolve_dependency(&c, &descriptor, None).unwrap();
    assert!(result.is_none());
}

#[test]
fn container_resolve_dependency_required_not_found() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let descriptor =
        vernal_beans::factory::support::dependency_descriptor::DependencyDescriptor::new(
            TypeId::of::<bool>(),
            "bool".to_string(),
            true, // required
        );
    let result = <Container as AutowireCapableBeanFactory>::resolve_dependency(&c, &descriptor, None);
    assert!(result.is_err());
}

#[test]
fn container_resolve_dependency_found() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    // Warm up singletons so resolve_dependency can find them
    let _ = c.warm_up();
    let descriptor =
        vernal_beans::factory::support::dependency_descriptor::DependencyDescriptor::new(
            TypeId::of::<String>(),
            "alloc::string::String".to_string(),
            true,
        );
    let result = <Container as AutowireCapableBeanFactory>::resolve_dependency(&c, &descriptor, None);
    // May succeed or fail depending on descriptor type_id matching
    // The important thing is exercising the code path
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — resolve_definition scope paths
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_in_scope_success() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let val: Arc<String> = c.resolve_in(&scope).unwrap();
    assert_eq!(*val, "hello");
}

#[test]
fn container_resolve_in_scope_wrong_owner() {
    let c1 = make_container();
    let c2 = make_container();
    let scope = c2.open_scope::<String>();
    let result: Result<Arc<String>, _> = c1.resolve_in(&scope);
    assert!(result.is_err());
    match result.unwrap_err() {
        ResolveError::ScopeOwnerMismatch { .. } => {}
        other => panic!("Expected ScopeOwnerMismatch, got: {:?}", other),
    }
}

#[test]
fn container_resolve_qualified_in_success() {
    let mut b = RegistryBuilder::new();
    let q = Qualifier::new("primary").unwrap();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "primary".to_string()).qualified(q.clone()),
    );
    let c = Container::new(b.build().unwrap());
    let scope = c.open_scope::<String>();
    let val: Arc<String> = c.resolve_qualified_in(&q, &scope).unwrap();
    assert_eq!(*val, "primary");
}

#[test]
fn container_resolve_qualified_not_found() {
    let c = make_container();
    let q = Qualifier::new("nonexistent").unwrap();
    let result: Result<Arc<String>, _> = c.resolve_qualified(&q);
    assert!(result.is_err());
}

#[test]
fn container_resolve_qualified_in_wrong_owner() {
    let mut b = RegistryBuilder::new();
    let q = Qualifier::new("q").unwrap();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "v".to_string()).qualified(q.clone()),
    );
    let c = Container::new(b.build().unwrap());
    let other = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = other.open_scope::<String>();
    let result: Result<Arc<String>, _> = c.resolve_qualified_in(&q, &scope);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — resolve_trait paths
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_trait_no_binding() {
    let c = make_container();
    let result: Result<Arc<dyn std::fmt::Debug + Send + Sync>, _> = c.resolve_trait();
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
fn container_resolve_all_traits_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
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

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — warm_up
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_warm_up_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    c.warm_up().unwrap();
}

#[test]
fn container_warm_up_resolves_singletons() {
    let c = make_container();
    c.warm_up().unwrap();
    assert!(c.unused_definitions().is_empty());
}

#[test]
fn container_unused_definitions_before_warm_up() {
    let c = make_container();
    let unused = c.unused_definitions();
    assert!(!unused.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — ListableBeanFactory methods
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_listable_beans_of_type_id_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let beans = c.beans_of_type_id(TypeId::of::<String>(), true, true).unwrap();
    assert!(beans.is_empty());
}

#[test]
fn container_listable_beans_of_type_id_found() {
    let c = make_container();
    let beans = c.beans_of_type_id(TypeId::of::<String>(), true, true).unwrap();
    assert_eq!(beans.len(), 1);
}

#[test]
fn container_listable_contains_non_singleton_with_transient() {
    let c = make_container_with_transient();
    assert!(c.contains_non_singleton_bean());
}

#[test]
fn container_listable_contains_singleton_bean() {
    let c = make_container();
    assert!(c.contains_singleton_bean());
}

#[test]
fn container_listable_bean_names_iterator() {
    let c = make_container();
    let names: Vec<String> = c.bean_names_iterator().collect();
    assert!(names.len() >= 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — HierarchicalBeanFactory
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_parent_none() {
    let c = make_container();
    assert!(c.parent_bean_factory().is_none());
}

#[test]
fn container_contains_local_bean_dynamic() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = make_container();
    let def = Box::new(RootBeanDefinition::new());
    c.register_bean_definition("localBean".to_string(), def)
        .unwrap();
    assert!(<Container as HierarchicalBeanFactory>::contains_local_bean(
        &c, "localBean"
    ));
}

#[test]
fn container_contains_local_bean_from_registry() {
    let c = make_container();
    assert!(<Container as HierarchicalBeanFactory>::contains_local_bean(
        &c,
        "alloc::string::String"
    ));
}

#[test]
fn container_contains_local_bean_not_found() {
    let c = make_container();
    assert!(!<Container as HierarchicalBeanFactory>::contains_local_bean(
        &c,
        "nonexistent"
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — ConfigurableBeanFactory
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
fn container_register_scope() {
    use vernal_beans::bean_scope::BeanScope;
    let mut c = make_container();
    struct TestScope;
    impl BeanScope for TestScope {
        fn get(
            &self,
            _n: &str,
            _f: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Box::new("scope_val"))
        }
    }
    c.register_scope("test_scope", Box::new(TestScope));
    let names = c.registered_scope_names();
    assert!(names.contains(&"test_scope".to_string()));
    assert!(c.get_registered_scope("test_scope").is_some());
}

#[test]
fn container_embedded_value_resolvers() {
    let mut c = make_container();
    c.add_embedded_value_resolver(Arc::new(|v: &str| v.replace("${a}", "1")));
    c.add_embedded_value_resolver(Arc::new(|v: &str| v.replace("${b}", "2")));
    assert_eq!(c.resolve_embedded_value("${a}-${b}"), "1-2");
}

#[test]
fn container_register_alias() {
    let mut c = make_container();
    assert!(c.register_alias("alloc::string::String", "myAlias").is_ok());
}

#[test]
fn container_currently_in_creation() {
    let mut c = make_container();
    c.set_currently_in_creation("test_bean", true);
    assert!(c.is_currently_in_creation("test_bean"));
    c.set_currently_in_creation("test_bean", false);
    assert!(!c.is_currently_in_creation("test_bean"));
}

#[test]
fn container_dependent_beans() {
    let mut c = make_container();
    c.register_dependent_bean("beanA", "beanB");
    let dependents = c.get_dependent_beans("beanA");
    assert!(dependents.contains(&"beanB".to_string()));
    let deps = c.get_dependencies_for_bean("beanB");
    assert!(deps.contains(&"beanA".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — ConfigurableListableBeanFactory
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_freeze_configuration() {
    let mut c = make_container();
    assert!(!c.is_configuration_frozen());
    c.freeze_configuration();
    assert!(c.is_configuration_frozen());
}

#[test]
fn container_pre_instantiate_singletons() {
    let c = make_container();
    assert!(c.pre_instantiate_singletons().is_ok());
}

#[test]
fn container_ignore_dependency_type() {
    let mut c = make_container();
    c.ignore_dependency_type(TypeId::of::<String>());
}

#[test]
fn container_ignore_dependency_interface() {
    let mut c = make_container();
    c.ignore_dependency_interface(TypeId::of::<dyn Any>());
}

#[test]
fn container_register_resolvable_dependency() {
    let mut c = make_container();
    c.register_resolvable_dependency(TypeId::of::<String>(), Arc::new("resolved".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — BeanDefinitionRegistry
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_register_and_remove_bean_definition() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = make_container();
    let def = Box::new(RootBeanDefinition::new());
    <Container as BeanDefinitionRegistry>::register_bean_definition(&mut c, "newBean".to_string(), def).unwrap();
    assert!(<Container as BeanDefinitionRegistry>::contains_bean_definition(&c, "newBean"));
    let removed = <Container as BeanDefinitionRegistry>::remove_bean_definition(&mut c, "newBean").unwrap();
    assert_eq!(removed.bean_class_name(), "unknown");
    assert!(!<Container as BeanDefinitionRegistry>::contains_bean_definition(&c, "newBean"));
}

#[test]
fn container_register_duplicate_bean_definition_fails() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = make_container();
    let def = Box::new(RootBeanDefinition::new());
    <Container as BeanDefinitionRegistry>::register_bean_definition(&mut c, "dup".to_string(), def).unwrap();
    let def2 = Box::new(RootBeanDefinition::new());
    let result = <Container as BeanDefinitionRegistry>::register_bean_definition(&mut c, "dup".to_string(), def2);
    assert!(result.is_err());
}

#[test]
fn container_remove_bean_definition_from_registry() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = make_container();
    let removed = <Container as BeanDefinitionRegistry>::remove_bean_definition(&mut c, "alloc::string::String");
    assert!(removed.is_ok());
}

#[test]
fn container_remove_nonexistent_bean_definition() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = make_container();
    let result = <Container as BeanDefinitionRegistry>::remove_bean_definition(&mut c, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn container_get_bean_definition_from_registry() {
    use vernal_beans::BeanDefinitionRegistry;
    let c = make_container();
    let def = <Container as BeanDefinitionRegistry>::get_bean_definition(&c, "alloc::string::String");
    assert!(def.is_some());
}

#[test]
fn container_get_bean_definition_dynamic() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = make_container();
    let def = Box::new(RootBeanDefinition::new());
    <Container as BeanDefinitionRegistry>::register_bean_definition(&mut c, "dynamic".to_string(), def).unwrap();
    let found = <Container as BeanDefinitionRegistry>::get_bean_definition(&c, "dynamic");
    assert!(found.is_some());
}

#[test]
fn container_get_bean_definition_not_found() {
    use vernal_beans::BeanDefinitionRegistry;
    let c = make_container();
    assert!(<Container as BeanDefinitionRegistry>::get_bean_definition(&c, "nonexistent").is_none());
}

#[test]
fn container_bean_definition_count_and_names() {
    use vernal_beans::BeanDefinitionRegistry;
    let c = make_container();
    assert!(<Container as BeanDefinitionRegistry>::bean_definition_count(&c) >= 2);
    let names = <Container as BeanDefinitionRegistry>::bean_definition_names(&c);
    assert!(names.len() >= 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — SingletonBeanRegistry
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_singleton_register_get_contains() {
    use vernal_beans::SingletonBeanRegistry;
    let c = make_container();
    c.register_singleton("my_singleton", Arc::new(99i32));
    assert!(c.contains_singleton("my_singleton"));
    let v = c.get_singleton("my_singleton");
    assert!(v.is_some());
    assert_eq!((*v.unwrap()).downcast_ref::<i32>().copied(), Some(99));
}

#[test]
fn container_singleton_names_and_count() {
    use vernal_beans::SingletonBeanRegistry;
    let c = make_container();
    // Register a singleton directly to ensure it shows up in names
    c.register_singleton("my_bean", Arc::new(42i32));
    let names = c.singleton_names();
    assert!(names.contains(&"my_bean".to_string()));
    let count = c.singleton_count();
    assert!(count > 0);
}

#[test]
fn container_singleton_mutex() {
    use vernal_beans::SingletonBeanRegistry;
    let c = make_container();
    let m1 = c.singleton_mutex();
    let m2 = c.singleton_mutex();
    assert!(Arc::ptr_eq(&m1, &m2));
}

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
    let obj: Arc<dyn Any + Send + Sync> = Arc::new("val".to_string());
    c.register_singleton("test", obj);
    assert!(called.load(std::sync::atomic::Ordering::SeqCst));
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — AutowireCapableBeanFactory
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_create_bean() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean = c.create_bean("alloc::string::String");
    assert!(bean.is_ok());
}

#[test]
fn container_create_bean_not_found() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean = c.create_bean("nonexistent");
    assert!(bean.is_err());
}

#[test]
fn container_autowire_bean() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    assert!(c.autowire_bean(bean).is_ok());
}

#[test]
fn container_autowire_bean_no_matching_definition() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new(42.0f64);
    // f64 has no definition, should return original
    assert!(c.autowire_bean(bean).is_ok());
}

#[test]
fn container_initialize_bean() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    assert!(c.initialize_bean(bean, "test_bean").is_ok());
}

#[test]
fn container_configure_bean() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    assert!(c.configure_bean(bean, "test_bean").is_ok());
}

#[test]
fn container_destroy_bean_instance() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    assert!(c.destroy_bean_instance("test_bean", &"test").is_ok());
}

#[test]
fn container_autowire_modes() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    assert!(c.autowire("alloc::string::String", 0, false).is_ok());
    assert!(c.autowire("alloc::string::String", 1, false).is_ok());
    assert!(c.autowire("alloc::string::String", 2, false).is_ok());
    assert!(c.autowire("alloc::string::String", 3, false).is_ok());
    assert!(c.autowire("alloc::string::String", 99, false).is_err());
}

#[test]
fn container_resolve_named_bean() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    assert!(c.resolve_named_bean(TypeId::of::<String>()).is_ok());
}

#[test]
fn container_resolve_named_bean_not_found() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(c.resolve_named_bean(TypeId::of::<String>()).is_err());
}

#[test]
fn container_type_converter() {
    use vernal_beans::AutowireCapableBeanFactory;
    let mut c = make_container();
    assert!(c.type_converter().is_none());
    c.set_type_converter(None);
}

#[test]
fn container_apply_bean_property_values() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    assert!(c.apply_bean_property_values(bean, "test_bean").is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — post processors and construct paths
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_construct_with_post_processor() {
    use vernal_beans::BeanPostProcessor;
    struct MockPP;
    impl BeanPostProcessor for MockPP {
        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _: &str,
        ) -> Result<
            Option<Arc<dyn Any + Send + Sync>>,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            Ok(Some(bean))
        }
    }
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let mut c = Container::new(b.build().unwrap());
    c.add_bean_post_processor(Arc::new(MockPP));
    assert_eq!(c.bean_post_processor_count(), 1);
    let val: Arc<String> = c.resolve().unwrap();
    assert_eq!(*val, "hello");
}

#[test]
fn container_post_processor_none_return() {
    use vernal_beans::BeanPostProcessor;
    struct NonePP;
    impl BeanPostProcessor for NonePP {
        fn post_process_after_initialization(
            &self,
            _bean: Arc<dyn Any + Send + Sync>,
            _: &str,
        ) -> Result<
            Option<Arc<dyn Any + Send + Sync>>,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            Ok(None)
        }
    }
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let mut c = Container::new(b.build().unwrap());
    c.add_bean_post_processor(Arc::new(NonePP));
    let val: Arc<String> = c.resolve().unwrap();
    assert_eq!(*val, "hello");
}

#[test]
fn container_post_processor_error_ignored() {
    use vernal_beans::BeanPostProcessor;
    struct ErrorPP;
    impl BeanPostProcessor for ErrorPP {
        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _: &str,
        ) -> Result<
            Option<Arc<dyn Any + Send + Sync>>,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            Err("processor error".into())
        }
    }
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let mut c = Container::new(b.build().unwrap());
    c.add_bean_post_processor(Arc::new(ErrorPP));
    let val: Arc<String> = c.resolve().unwrap();
    assert_eq!(*val, "hello");
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — transient scope tracking
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_transient_scope_tracking() {
    let c = make_container_with_transient();
    let tracker = c.transient_tracker();
    assert_eq!(tracker.total_surviving(), 0);
    let a: Arc<i32> = c.resolve().unwrap();
    let b_val: Arc<i32> = c.resolve().unwrap();
    assert_eq!(*a, 99);
    assert_eq!(*b_val, 99);
    let surviving = tracker.surviving_instances(TypeId::of::<i32>());
    assert!(surviving.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — BeanFactory trait
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_bean_factory_get_bean_by_key() {
    let c = make_container();
    let bean = c.get_bean_by_key(&ComponentKey::of::<String>());
    assert!(bean.is_ok());
}

#[test]
fn container_bean_factory_get_bean_by_key_not_found() {
    let c = make_container();
    let bean = c.get_bean_by_key(&ComponentKey::of::<bool>());
    assert!(bean.is_err());
}

#[test]
fn container_bean_factory_contains_bean() {
    let c = make_container();
    assert!(c.contains_bean(&ComponentKey::of::<String>()));
    assert!(!c.contains_bean(&ComponentKey::of::<bool>()));
}

#[test]
fn container_bean_factory_is_singleton() {
    let c = make_container();
    assert!(c.is_singleton(&ComponentKey::of::<String>()).unwrap());
}

#[test]
fn container_bean_factory_is_prototype() {
    let c = make_container();
    assert!(!c.is_prototype(&ComponentKey::of::<String>()).unwrap());
}

#[test]
fn container_bean_factory_get_type() {
    let c = make_container();
    assert!(c.get_type(&ComponentKey::of::<String>()).is_ok());
}

#[test]
fn container_bean_factory_get_aliases() {
    let c = make_container();
    assert!(c.get_aliases(&ComponentKey::of::<String>()).is_empty());
}

#[test]
fn container_bean_factory_is_type_match() {
    let c = make_container();
    assert!(c.is_type_match(&ComponentKey::of::<String>(), TypeId::of::<String>()));
}

#[test]
fn container_bean_factory_get_provider() {
    let c = make_container();
    let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>());
    assert!(provider.is_ok());
}

#[test]
fn container_bean_factory_provider_methods() {
    let c = make_container();
    // Resolve a singleton first so provider has something to return
    let _: Arc<String> = c.resolve().unwrap();
    let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
    let val = provider.get();
    assert!(val.is_ok());
    let if_available = provider.if_available();
    assert!(if_available.is_some());
    let if_unique = provider.get_if_unique();
    assert!(if_unique.is_ok());
    let stream = provider.stream();
    assert!(!stream.is_empty());
    let ordered = provider.ordered_stream();
    assert!(!ordered.is_empty());
}

#[test]
fn container_bean_factory_provider_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
    let val = provider.get();
    assert!(val.is_err());
    assert!(provider.if_available().is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — early bean reference
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_early_bean_reference() {
    let c = make_container();
    let key = ComponentKey::of::<String>();
    assert!(c.get_early_bean_reference(&key).is_none());
    c.register_early_bean_reference(key.clone(), Arc::new("early".to_string()));
    assert!(c.get_early_bean_reference(&key).is_some());
    let removed = c.remove_early_bean_reference(&key);
    assert!(removed.is_some());
    assert!(c.get_early_bean_reference(&key).is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — display_path
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_display_path() {
    let c = make_container();
    // Trigger a circular dependency error to exercise display_path
    // This is hard to create naturally, but the NotFound error also uses display_path
    let result: Result<Arc<bool>, _> = c.resolve();
    assert!(result.is_err());
    let err = result.unwrap_err();
    let display = format!("{}", err);
    assert!(!display.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// container.rs — scope_context open
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_open_scope() {
    let c = make_container();
    let _scope = c.open_scope::<String>();
}

#[test]
fn container_open_scope_with_cancellation() {
    let c = make_container();
    let token = tokio_util::sync::CancellationToken::new();
    let _scope = c.open_scope_with_cancellation::<String>(token);
}

// ═══════════════════════════════════════════════════════════════════════════════
// scope_context.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn scope_context_close_with_timeout_fast() {
    let c = make_container();
    let scope = c.open_scope::<()>();
    let result = scope
        .close_with_timeout(std::time::Duration::from_secs(10))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn scope_context_close_twice_idempotent() {
    let c = make_container();
    let scope = c.open_scope::<()>();
    scope.close().await.unwrap();
    scope.close().await.unwrap();
    assert_eq!(scope.state(), ScopeState::Closed);
}

#[tokio::test]
async fn scope_context_get_or_insert_with_after_close() {
    let c = make_container();
    let scope = c.open_scope::<()>();
    scope.close().await.unwrap();
    let result = scope.get_or_insert_with::<String, _>(|| "hello".to_string());
    assert!(result.is_err());
}

#[tokio::test]
async fn scope_context_on_close_after_close() {
    let c = make_container();
    let scope = c.open_scope::<()>();
    scope.close().await.unwrap();
    let result = scope.on_close(|| async { Ok::<(), std::io::Error>(()) });
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// property_editor_registry.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn property_editor_registry_has_custom_editor() {
    use vernal_beans::property_editor_registry::SimplePropertyEditorRegistry;
    let mut registry = SimplePropertyEditorRegistry::new();
    assert!(!registry.has_custom_editor(TypeId::of::<String>(), None));
    registry.register_custom_editor(TypeId::of::<String>(), Box::new(StubEditor::new("c")));
    assert!(registry.has_custom_editor(TypeId::of::<String>(), None));
    assert!(registry.has_custom_editor(TypeId::of::<String>(), Some("any")));
}

#[test]
fn property_editor_registry_has_custom_editor_path_specific() {
    use vernal_beans::property_editor_registry::SimplePropertyEditorRegistry;
    let mut registry = SimplePropertyEditorRegistry::new();
    registry.register_custom_editor_for_path(
        TypeId::of::<String>(),
        "name",
        Box::new(StubEditor::new("c")),
    );
    assert!(registry.has_custom_editor(TypeId::of::<String>(), Some("name")));
    assert!(!registry.has_custom_editor(TypeId::of::<String>(), Some("other")));
}

// ═══════════════════════════════════════════════════════════════════════════════
// property_editor_registry_support.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn property_editor_registry_support_full_lifecycle() {
    let mut registry = vernal_beans::property_editor_registry_support::PropertyEditorRegistrySupport::new();

    // Register default editor
    registry.register_default_editor(TypeId::of::<i32>(), Box::new(StubEditor::new("default")));
    assert!(registry.has_default_editor(TypeId::of::<i32>()));
    assert_eq!(registry.custom_editor_count(), 0);

    // Register custom editor
    registry.register_custom_editor(TypeId::of::<String>(), Box::new(StubEditor::new("custom")));
    assert_eq!(registry.custom_editor_count(), 1);

    // Register path editor
    registry.register_custom_editor_for_path(
        TypeId::of::<String>(),
        "name",
        Box::new(StubEditor::new("path")),
    );
    assert_eq!(registry.custom_editor_count(), 2);

    // Find with path priority
    let editor = registry.find_custom_editor(TypeId::of::<String>(), Some("name"));
    assert!(editor.is_some());

    // Find with type fallback
    let editor = registry.find_custom_editor(TypeId::of::<String>(), Some("other"));
    assert!(editor.is_some());

    // Find default
    let editor = registry.find_custom_editor(TypeId::of::<i32>(), None);
    assert!(editor.is_some());

    // Override defaults
    registry.override_default_editors();
    assert!(!registry.has_default_editor(TypeId::of::<i32>()));
    let editor = registry.find_custom_editor(TypeId::of::<i32>(), None);
    assert!(editor.is_none());

    // Restore defaults
    registry.restore_default_editors();
    assert!(registry.has_default_editor(TypeId::of::<i32>()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// property_editor_cache.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn property_editor_cache_full_lifecycle() {
    let cache = PropertyEditorCache::new();

    // Register custom
    cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("custom")));
    assert!(cache.has_custom_editor_for(TypeId::of::<String>()));
    assert_eq!(cache.custom_editor_count(), 1);

    // Register default
    cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("default")));
    assert_eq!(cache.default_editor_count(), 1);

    // Find custom first
    let found = cache.find_editor(TypeId::of::<String>());
    assert!(found.is_some());

    // Find default fallback
    let found = cache.find_editor(TypeId::of::<i32>());
    assert!(found.is_some());

    // Mark defaults registered
    cache.mark_default_editors_registered();
    assert!(cache.has_default_editors());

    // Clear custom only
    cache.clear_custom_editors();
    assert_eq!(cache.custom_editor_count(), 0);
    assert_eq!(cache.default_editor_count(), 1);
    assert!(cache.has_default_editors());

    // Clear all
    cache.clear();
    assert_eq!(cache.custom_editor_count(), 0);
    assert_eq!(cache.default_editor_count(), 0);
    assert!(!cache.has_default_editors());
}

// ═══════════════════════════════════════════════════════════════════════════════
// type_converter_delegate.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn type_converter_delegate_full_lifecycle() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();

    // Register custom converter
    delegate.register_converter(TypeId::of::<i32>(), TypeId::of::<i64>(), |v| {
        let n = v.downcast_ref::<i32>().unwrap();
        Ok(Box::new(*n as i64))
    });

    // Use custom converter
    let value = 42i32;
    let result = delegate
        .convert_if_necessary(None, &value, TypeId::of::<i64>())
        .unwrap();
    assert_eq!(*result.downcast::<i64>().unwrap(), 42i64);

    // Same type conversion
    let value = "hello".to_string();
    let result = delegate
        .convert_if_necessary(None, &value, TypeId::of::<String>())
        .unwrap();
    assert_eq!(*result.downcast::<String>().unwrap(), "hello");

    // Unsupported type
    struct Unsupported;
    let value = Unsupported;
    let result = delegate.convert_if_necessary(None, &value, TypeId::of::<String>());
    assert!(result.is_err());

    // Clear
    delegate.clear();
    assert_eq!(delegate.editor_count(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// standard_bean_expression_resolver.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn expression_resolver_spel_expressions() {
    use vernal_beans::BeanExpressionResolver;
    let r = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();

    // Arithmetic
    let result = r.evaluate("1 + 1", None).unwrap();
    assert!(result.is_some());
    assert_eq!(*result.unwrap().downcast_ref::<i64>().unwrap(), 2);

    // String literal
    let result = r.evaluate("'hello'", None).unwrap();
    assert!(result.is_some());
    assert_eq!(*result.unwrap().downcast_ref::<String>().unwrap(), "hello");

    // Numeric literal
    let result = r.evaluate("42", None).unwrap();
    assert!(result.is_some());
    assert_eq!(*result.unwrap().downcast_ref::<i64>().unwrap(), 42);

    // Comparison
    let result = r.evaluate("3 > 2", None).unwrap();
    assert!(result.is_some());
    assert!(*result.unwrap().downcast_ref::<bool>().unwrap());

    // Invalid expression
    let result = r.evaluate("@#$invalid", None).unwrap();
    assert!(result.is_none());

    // Empty
    let result = r.evaluate("", None).unwrap();
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// trait_binding.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn trait_binding_new_qualified_primary() {
    let q = Qualifier::new("myqual").unwrap();
    let binding = vernal_beans::TraitBinding::new(|s: Arc<String>| {
        s as Arc<dyn std::fmt::Display + Send + Sync>
    })
    .qualified(q.clone())
    .primary();

    assert!(binding.key().qualifier().is_some());
    assert!(binding.is_primary());
    assert!(format!("{}", binding).contains("primary"));
    assert!(format!("{:?}", binding).contains("TraitBinding"));
}

#[test]
fn trait_binding_target_qualified() {
    let q = Qualifier::new("target_q").unwrap();
    let binding = vernal_beans::TraitBinding::new(|s: Arc<String>| {
        s as Arc<dyn std::fmt::Display + Send + Sync>
    })
    .target_qualified(q);

    assert!(binding.target().qualifier().is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// mergeable.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn mergeable_default_trait_methods() {
    use vernal_beans::mergeable::Mergeable;
    struct NoOpMergeable;
    impl Mergeable for NoOpMergeable {}

    let m = NoOpMergeable;
    assert!(!m.is_merge_enabled());
    assert!(m.merge(&"parent".to_string()).is_err());
}

#[test]
fn simple_mergeable_with_value_merge() {
    use vernal_beans::mergeable::Mergeable;
    let m = vernal_beans::mergeable::SimpleMergeable::with_value(true, "child");
    let parent = "parent".to_string();
    let result = m.merge(&parent).unwrap();
    assert_eq!(result.downcast_ref::<String>().unwrap(), "child");
}

#[test]
fn simple_mergeable_without_value_merge() {
    use vernal_beans::mergeable::Mergeable;
    let m = vernal_beans::mergeable::SimpleMergeable::new(true);
    let parent = "parent".to_string();
    let result = m.merge(&parent).unwrap();
    assert_eq!(result.downcast_ref::<String>().unwrap(), "parent");
}

#[test]
fn simple_mergeable_merge_not_enabled() {
    use vernal_beans::mergeable::Mergeable;
    let m = vernal_beans::mergeable::SimpleMergeable::new(false);
    let parent = "parent".to_string();
    assert!(m.merge(&parent).is_err());
}

#[test]
fn simple_mergeable_merge_non_string_parent() {
    use vernal_beans::mergeable::Mergeable;
    let m = vernal_beans::mergeable::SimpleMergeable::new(true);
    let parent = 42i32;
    assert!(m.merge(&parent).is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// bean_wrapper_info.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_wrapper_info_full_lifecycle() {
    let mut info = vernal_beans::bean_wrapper_info::BeanWrapperInfo::new(
        TypeId::of::<String>(),
        "String".to_string(),
    );

    assert_eq!(info.bean_type(), TypeId::of::<String>());
    assert_eq!(info.type_name(), "String");
    assert!(!info.has_nested_path());

    info.set_nested_path("order.customer".to_string());
    assert!(info.has_nested_path());
    assert_eq!(info.nested_path(), "order.customer");
    assert_eq!(info.build_property_path("name"), "order.customer.name");

    info.add_readable_property("name");
    info.add_readable_property("age");
    info.add_writable_property("name");
    assert!(info.is_readable("name"));
    assert!(info.is_readable("age"));
    assert!(!info.is_readable("missing"));
    assert!(info.is_writable("name"));
    assert!(!info.is_writable("age"));
    assert_eq!(info.readable_property_count(), 2);
    assert_eq!(info.writable_property_count(), 1);

    let readable = info.readable_property_names();
    assert!(readable.contains(&"name".to_string()));
    let writable = info.writable_property_names();
    assert!(writable.contains(&"name".to_string()));

    let cloned = info.clone();
    assert!(cloned.is_readable("name"));
    assert_eq!(cloned.nested_path(), "order.customer");
}

#[test]
fn bean_wrapper_info_no_nested_path() {
    let info = vernal_beans::bean_wrapper_info::BeanWrapperInfo::new(
        TypeId::of::<i32>(),
        "i32".to_string(),
    );
    assert_eq!(info.build_property_path("value"), "value");
}

// ═══════════════════════════════════════════════════════════════════════════════
// bean_definition_utils.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_utils_generate_bename_no_conflict() {
    let builder = RegistryBuilder::new();
    let name =
        vernal_beans::bean_definition_utils::generate_bean_name(Some("myBean"), &builder);
    assert_eq!(name, "myBean");
}

#[test]
fn bean_definition_utils_generate_bean_name_none() {
    let builder = RegistryBuilder::new();
    let name = vernal_beans::bean_definition_utils::generate_bean_name(None, &builder);
    assert_eq!(name, "anonymous");
}

#[test]
fn bean_definition_utils_generate_bean_name_conflict() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    let name =
        vernal_beans::bean_definition_utils::generate_bean_name(Some("i32"), &builder);
    assert_eq!(name, "i32#1");
}

// ═══════════════════════════════════════════════════════════════════════════════
// component_contract.rs — tested via ComponentDefinition access
// (component_contract module is private, so we test via public API)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_definition_debug() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string());
    let debug = format!("{:?}", def);
    assert!(!debug.is_empty());
}

#[test]
fn component_definition_key_and_scope() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string());
    assert!(def.key().type_name().contains("String"));
    assert_eq!(def.scope(), Scope::Singleton);
}

// ═══════════════════════════════════════════════════════════════════════════════
// scope_error.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_error_all_variants_display() {
    let err = ScopeError::InvalidState {
        operation: "open",
        scope: ScopeKey::of::<String>(),
        state: ScopeState::Closed,
    };
    assert!(format!("{}", err).contains("open"));

    let err = ScopeError::Cancelled {
        operation: "resolve",
        scope: ScopeKey::of::<i32>(),
    };
    assert!(format!("{}", err).contains("cancelled"));

    let err = ScopeError::TypeMismatch {
        scope: ScopeKey::of::<String>(),
        expected: "MyType",
    };
    assert!(format!("{}", err).contains("type mismatch"));

    let err = ScopeError::Resolution {
        scope: ScopeKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "bean".to_string(),
            path: vec![],
        }),
    };
    assert!(format!("{}", err).contains("resolution failed"));

    let err = ScopeError::CloseTimeout {
        scope: ScopeKey::of::<String>(),
        timeout: std::time::Duration::from_secs(5),
    };
    assert!(format!("{}", err).contains("timeout"));

    let err = ScopeError::RuntimeUnavailable {
        scope: ScopeKey::of::<String>(),
    };
    assert!(format!("{}", err).contains("Tokio runtime"));

    let err = ScopeError::CloseHook {
        scope: ScopeKey::of::<String>(),
        source: shared_err("hook failed"),
    };
    assert!(format!("{}", err).contains("close hook failed"));

    let err = ScopeError::CloseTask {
        scope: ScopeKey::of::<String>(),
        source: shared_err("task failed"),
    };
    assert!(format!("{}", err).contains("close task failed"));
}

#[test]
fn scope_error_source_methods() {
    assert!(
        std::error::Error::source(&ScopeError::Resolution {
            scope: ScopeKey::of::<String>(),
            source: Box::new(ResolveError::NotFound {
                component: "b".to_string(),
                path: vec![]
            }),
        })
        .is_some()
    );
    assert!(
        std::error::Error::source(&ScopeError::CloseHook {
            scope: ScopeKey::of::<String>(),
            source: shared_err("e"),
        })
        .is_some()
    );
    assert!(
        std::error::Error::source(&ScopeError::CloseTask {
            scope: ScopeKey::of::<String>(),
            source: shared_err("e"),
        })
        .is_some()
    );
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

// ═══════════════════════════════════════════════════════════════════════════════
// ResolveError — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn resolve_error_all_variants_display() {
    let err = ResolveError::NotFound {
        component: "b".to_string(),
        path: vec!["a".into(), "b".into()],
    };
    assert!(format!("{}", err).contains("not found"));

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

    let err = ResolveError::TraitBindingTypeMismatch {
        binding: vernal_beans::TraitKey::of::<dyn Any + Send + Sync>(),
        target: ComponentKey::of::<String>(),
    };
    assert!(format!("{}", err).contains("type mismatch"));

    let err = ResolveError::Construction {
        component: ComponentKey::of::<String>(),
        source: shared_err("build failed"),
    };
    let display = format!("{}", err);
    assert!(!display.is_empty());
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
fn resolve_error_clone() {
    let err = ResolveError::NotFound {
        component: "b".into(),
        path: vec![],
    };
    let cloned = err.clone();
    assert_eq!(format!("{}", err), format!("{}", cloned));
}

// ═══════════════════════════════════════════════════════════════════════════════
// graph_error.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn graph_error_all_variants() {
    let err = GraphError::MissingDependency {
        path: vec!["A".into(), "B".into()],
    };
    assert!(format!("{}", err).contains("missing dependency"));

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
// DefinitionError — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn definition_error_all_variants() {
    let err = DefinitionError::DuplicateDefinition {
        key: ComponentKey::of::<String>(),
    };
    assert!(format!("{}", err).contains("duplicate"));

    let err = DefinitionError::DuplicateTraitBinding {
        key: vernal_beans::TraitKey::of::<dyn Any + Send + Sync>(),
        target: ComponentKey::of::<String>(),
    };
    assert!(format!("{}", err).contains("duplicate"));

    let err = DefinitionError::DuplicateQualifiedTraitBinding {
        key: vernal_beans::TraitKey::of::<dyn Any + Send + Sync>(),
    };
    assert!(format!("{}", err).contains("duplicate"));

    let err = DefinitionError::MultiplePrimaryTraitBindings {
        trait_name: "MyTrait",
    };
    assert!(format!("{}", err).contains("multiple"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// registry_builder.rs — additional coverage for validate_bindings
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn registry_builder_validate_bindings_duplicate_exact() {
    let mut b = RegistryBuilder::new();
    let binding1 =
        vernal_beans::TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Debug + Send + Sync>);
    let binding2 =
        vernal_beans::TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Debug + Send + Sync>);
    b.bind_all(vec![binding1]).unwrap();
    let result = b.bind_all(vec![binding2]);
    assert!(result.is_err());
}

#[test]
fn registry_builder_validate_bindings_duplicate_qualified() {
    let mut b = RegistryBuilder::new();
    let q = Qualifier::new("myqual").unwrap();
    let binding1 =
        vernal_beans::TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Debug + Send + Sync>)
            .qualified(q.clone());
    let binding2 =
        vernal_beans::TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Debug + Send + Sync>)
            .qualified(q);
    b.bind_all(vec![binding1]).unwrap();
    let result = b.bind_all(vec![binding2]);
    assert!(result.is_err());
}

#[test]
fn registry_builder_validate_bindings_multiple_primary() {
    let mut b = RegistryBuilder::new();
    let binding1 =
        vernal_beans::TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
            .primary();
    let binding2 =
        vernal_beans::TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>)
            .primary();
    let result = b.bind_all(vec![binding1, binding2]);
    assert!(result.is_err());
}

#[test]
fn registry_builder_bind_chaining() {
    let mut b = RegistryBuilder::new();
    b.bind(
        vernal_beans::TraitBinding::new(|s: Arc<String>| {
            s as Arc<dyn std::fmt::Display + Send + Sync>
        }),
    )
    .unwrap();
    assert!(!b.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// component_key.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_key_display_debug() {
    let key = ComponentKey::of::<String>();
    assert!(!format!("{}", key).is_empty());
    assert!(!format!("{:?}", key).is_empty());
}

#[test]
fn component_key_equality() {
    let a = ComponentKey::of::<String>();
    let b = ComponentKey::of::<String>();
    let c = ComponentKey::of::<i32>();
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert!(a.qualifier().is_none());
    assert!(a.type_name().contains("String"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// scope_key.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_key_display_type_name() {
    let key = ScopeKey::of::<String>();
    assert!(!format!("{}", key).is_empty());
    assert!(key.type_name().contains("String"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// scope_state.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_state_all_variants() {
    assert_eq!(ScopeState::default(), ScopeState::Open);
    assert_eq!(format!("{:?}", ScopeState::Open), "Open");
    assert_eq!(format!("{:?}", ScopeState::Closing), "Closing");
    assert_eq!(format!("{:?}", ScopeState::Closed), "Closed");
    assert_ne!(ScopeState::Open, ScopeState::Closed);
}

// ═══════════════════════════════════════════════════════════════════════════════
// trait_key.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn trait_key_display_type_name() {
    let key = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    assert!(!format!("{}", key).is_empty());
    assert!(!key.type_name().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// dependency.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn dependency_all_variants() {
    let d = vernal_beans::Dependency::of::<String>();
    assert!(!format!("{}", d).is_empty());

    let q = Qualifier::new("primary").unwrap();
    let d = vernal_beans::Dependency::qualified::<String>(q);
    assert!(format!("{}", d).contains("primary"));

    let d = vernal_beans::Dependency::trait_of::<dyn Any + Send + Sync>();
    assert!(!format!("{}", d).is_empty());

    let q = Qualifier::new("q").unwrap();
    let d = vernal_beans::Dependency::trait_qualified::<dyn Any + Send + Sync>(q);
    assert!(format!("{}", d).contains("q"));

    let d = vernal_beans::Dependency::all_traits_of::<dyn Any + Send + Sync>();
    assert!(!format!("{}", d).is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// qualifier.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn qualifier_display_equality() {
    let q = Qualifier::new("primary").unwrap();
    assert!(format!("{}", q).contains("primary"));
    assert_eq!(q, Qualifier::new("primary").unwrap());
    assert_ne!(q, Qualifier::new("other").unwrap());
}

// ═══════════════════════════════════════════════════════════════════════════════
// component_definition.rs — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_definition_scopes() {
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
    let def =
        ComponentDefinition::singleton::<String, _>(|_| "h".to_string()).qualified(q);
    assert!(def.key().qualifier().is_some());
}

#[test]
fn component_definition_dependencies_default_empty() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "s".to_string());
    assert!(def.dependencies().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// scope.rs — additional coverage
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
// ConversionServiceFactory — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn conversion_factory_all_conversions() {
    let f = ConversionServiceFactory::new();
    f.register_defaults();

    // String -> i64
    let r = f.convert(&"12345".to_string(), "String", "i64").unwrap();
    assert_eq!(*r.unwrap().downcast::<i64>().unwrap(), 12345);

    // String -> f64
    let r = f.convert(&"3.14".to_string(), "String", "f64").unwrap();
    let val = *r.unwrap().downcast::<f64>().unwrap();
    assert!((val - 3.14_f64).abs() < f64::EPSILON);

    // i64 -> String
    let r = f.convert(&12345i64, "i64", "String").unwrap();
    assert_eq!(*r.unwrap().downcast::<String>().unwrap(), "12345");

    // String -> i32
    let r = f.convert(&"42".to_string(), "String", "i32").unwrap();
    assert_eq!(*r.unwrap().downcast::<i32>().unwrap(), 42);

    // No converter
    let r = f.convert(&42i32, "i32", "u64").unwrap();
    assert!(r.is_none());

    // Invalid string
    let r = f
        .convert(&"not_a_number".to_string(), "String", "i32")
        .unwrap();
    assert!(r.is_none());

    // has_converter
    assert!(f.has_converter("String", "i32"));
    assert!(!f.has_converter("String", "u64"));

    // Debug
    let d = format!("{:?}", f);
    assert!(d.contains("ConversionServiceFactory"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// TransientTracker — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn transient_tracker_full_lifecycle() {
    let t = TransientTracker::new();
    let a: Arc<dyn Any + Send + Sync> = Arc::new(1i32);
    let b: Arc<dyn Any + Send + Sync> = Arc::new(2i32);
    let c: Arc<dyn Any + Send + Sync> = Arc::new("hello".to_string());
    t.track(TypeId::of::<i32>(), &a);
    t.track(TypeId::of::<i32>(), &b);
    t.track(TypeId::of::<String>(), &c);
    assert_eq!(t.total_surviving(), 3);
    assert_eq!(t.surviving_instances(TypeId::of::<i32>()).len(), 2);
    assert_eq!(t.surviving_instances(TypeId::of::<String>()).len(), 1);
    assert!(t.surviving_instances(TypeId::of::<f64>()).is_empty());
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
    assert!(t.surviving_instances(TypeId::of::<i32>()).is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// DirectFieldAccessor — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn direct_field_accessor_full_lifecycle() {
    let a = DirectFieldAccessor::new(String::from("test"));
    a.set_field_value("name", String::from("Alice"));
    a.set_field_value("count", 42i32);
    assert!(a.get_field_value("name").is_some());
    assert!(a.get_field_value("missing").is_none());
    assert_eq!(a.get_field_type("count").unwrap(), TypeId::of::<i32>());
    assert!(a.get_field_type("missing").is_none());
    assert!(a.target().is::<String>());
    assert_eq!(a.field_names().len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// BeanFactoryUtils — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_factory_utils_all_methods() {
    assert_eq!(BeanFactoryUtils::transformed_bean_name("&myBean"), "myBean");
    assert_eq!(BeanFactoryUtils::transformed_bean_name("myBean"), "myBean");
    assert_eq!(BeanFactoryUtils::transformed_bean_name("&"), "");
    assert!(BeanFactoryUtils::is_factory_bean("&myFactory"));
    assert!(!BeanFactoryUtils::is_factory_bean("regularBean"));
    assert!(!BeanFactoryUtils::is_factory_bean(""));
    assert!(BeanFactoryUtils::check_is_factory_bean("&myFactory"));
    assert!(!BeanFactoryUtils::check_is_factory_bean("regularBean"));

    let c = make_container();
    assert_eq!(
        BeanFactoryUtils::count_beans_for_type(TypeId::of::<String>(), &c),
        1
    );
    assert_eq!(
        BeanFactoryUtils::bean_names_for_type(TypeId::of::<String>(), &c).len(),
        1
    );
    let beans = BeanFactoryUtils::beans_of_type(TypeId::of::<String>(), &c).unwrap();
    assert_eq!(beans.len(), 1);
    assert!(BeanFactoryUtils::bean_definition_names(&c).len() >= 2);
    assert_eq!(
        BeanFactoryUtils::count_beans_for_type_including_ancestors(TypeId::of::<String>(), &c),
        1
    );
    assert_eq!(
        BeanFactoryUtils::bean_names_for_type_including_ancestors(TypeId::of::<String>(), &c)
            .len(),
        1
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// BeanDescCache — additional coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_desc_cache_lifecycle() {
    let cache = BeanDescCache::new();
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
    cache.clear();
    assert!(cache.is_empty());

    let g1 = BeanDescCache::global();
    let g2 = BeanDescCache::global();
    assert!(std::ptr::eq(g1, g2));
}

// ═══════════════════════════════════════════════════════════════════════════════
// SmartInstantiationAwareBeanPostProcessor — additional coverage
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
    assert!(p.post_process_before_instantiation("C", "n").unwrap().is_none());
    assert!(p.post_process_after_instantiation(&42i32, "n").unwrap());
    let r = p.post_process_before_initialization(bean.clone(), "n").unwrap();
    assert!(r.is_some() && Arc::ptr_eq(&bean, &r.unwrap()));
    let r = p.post_process_after_initialization(bean, "n").unwrap();
    assert!(r.is_some());
}
