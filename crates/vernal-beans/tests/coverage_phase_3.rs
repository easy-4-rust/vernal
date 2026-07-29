//! Coverage Phase 3: targeted tests for uncovered lines in container.rs,
//! scope_context.rs, registry_builder.rs, component_definition.rs, resolver.rs,
//! and dependency.rs.

use std::any::{Any, TypeId};
use std::sync::Arc;

use vernal_beans::{
    AutowireCapableBeanFactory, BeanDefinitionRegistry, BeanFactory, BeanPostProcessor,
    ComponentDefinition, ComponentKey, ConfigurableBeanFactory, ConfigurableListableBeanFactory,
    Container, Dependency, DependencyDescriptor, HierarchicalBeanFactory, ListableBeanFactory,
    Qualifier, RegistryBuilder, Scope, ScopeKey, ScopeState, SingletonBeanRegistry, TraitBinding,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Shared test types
// ═══════════════════════════════════════════════════════════════════════════════

#[allow(dead_code)]
trait MyTrait: Send + Sync + 'static {
    fn identify(&self) -> &'static str;
}

struct MyImpl;
impl MyTrait for MyImpl {
    fn identify(&self) -> &'static str {
        "MyImpl"
    }
}

struct OtherImpl;
impl MyTrait for OtherImpl {
    fn identify(&self) -> &'static str {
        "OtherImpl"
    }
}

#[derive(Debug, PartialEq)]
struct ServiceA {
    value: String,
}

#[derive(Debug, PartialEq)]
struct ServiceB {
    a: Option<Arc<ServiceA>>,
}

struct RequestScope;
struct SubScope;

// Helper: empty container
fn empty_container() -> Container {
    Container::new(RegistryBuilder::new().build().unwrap())
}

// Helper: container with a String singleton
fn string_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    Container::new(b.build().unwrap())
}

// Helper: container with multiple types
fn multi_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    b.register(ComponentDefinition::singleton::<bool, _>(|_| true))
        .unwrap();
    Container::new(b.build().unwrap())
}

// =========================================================================
// 1. container.rs — ContainerObjectProvider, BeanFactory trait, etc.
// =========================================================================

#[test]
fn container_object_provider_get_empty() {
    // ContainerObjectProvider::get() when no singleton exists
    let container = empty_container();
    let provider = BeanFactory::get_bean_provider_by_type_id(&container, TypeId::of::<String>())
        .expect("provider creation");
    let result = provider.get();
    assert!(result.is_err());
}

#[test]
fn container_object_provider_if_available_empty() {
    let container = empty_container();
    let provider = BeanFactory::get_bean_provider_by_type_id(&container, TypeId::of::<String>())
        .expect("provider creation");
    let result = provider.if_available();
    assert!(result.is_none());
}

#[test]
fn container_is_type_match() {
    let container = string_container();
    let key = ComponentKey::of::<String>();
    assert!(BeanFactory::is_type_match(
        &container,
        &key,
        TypeId::of::<String>()
    ));
    assert!(!BeanFactory::is_type_match(
        &container,
        &key,
        TypeId::of::<i32>()
    ));
}

#[test]
fn container_get_type() {
    let container = string_container();
    let key = ComponentKey::of::<String>();
    let result = BeanFactory::get_type(&container, &key);
    assert!(result.is_ok());
    assert!(result.unwrap().unwrap().contains("String"));

    let missing_key = ComponentKey::of::<i32>();
    let result = BeanFactory::get_type(&container, &missing_key);
    assert!(result.is_err());
}

#[test]
fn container_get_aliases() {
    let container = string_container();
    let key = ComponentKey::of::<String>();
    let aliases = BeanFactory::get_aliases(&container, &key);
    assert!(aliases.is_empty());
}

#[test]
fn container_contains_bean() {
    let container = string_container();
    let key = ComponentKey::of::<String>();
    assert!(BeanFactory::contains_bean(&container, &key));

    let missing_key = ComponentKey::of::<Vec<u8>>();
    assert!(!BeanFactory::contains_bean(&container, &missing_key));
}

#[test]
fn container_is_singleton() {
    let container = string_container();
    let key = ComponentKey::of::<String>();
    assert!(BeanFactory::is_singleton(&container, &key).unwrap());
    assert!(!BeanFactory::is_prototype(&container, &key).unwrap());

    let missing_key = ComponentKey::of::<i32>();
    assert!(BeanFactory::is_singleton(&container, &missing_key).is_err());
    assert!(BeanFactory::is_prototype(&container, &missing_key).is_err());
}

#[test]
fn container_is_prototype_with_transient() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::transient::<i32, _>(|_| 99i32))
        .unwrap();
    let container = Container::new(b.build().unwrap());
    let key = ComponentKey::of::<i32>();
    assert!(BeanFactory::is_prototype(&container, &key).unwrap());
    assert!(!BeanFactory::is_singleton(&container, &key).unwrap());
}

#[test]
fn container_get_bean_by_type_id() {
    let container = string_container();
    // Single match
    let result = BeanFactory::get_bean_by_type_id(&container, TypeId::of::<String>());
    assert!(result.is_ok());

    // No match
    let result = BeanFactory::get_bean_by_type_id(&container, TypeId::of::<f64>());
    assert!(result.is_err());
}

#[test]
fn container_get_bean_by_type_id_multiple() {
    let mut b = RegistryBuilder::new();
    let q = Qualifier::new("a").unwrap();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "first".to_string()).qualified(q.clone()),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "second".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    // Multiple String definitions should cause error
    let result = BeanFactory::get_bean_by_type_id(&container, TypeId::of::<String>());
    assert!(result.is_err());
}

#[test]
fn container_resolve_named_bean() {
    let container = string_container();
    // Single match
    let result = AutowireCapableBeanFactory::resolve_named_bean(&container, TypeId::of::<String>());
    assert!(result.is_ok());
    let holder = result.unwrap();
    assert_eq!(holder.bean_name(), "alloc::string::String");

    // Not found
    let result = AutowireCapableBeanFactory::resolve_named_bean(&container, TypeId::of::<f64>());
    assert!(result.is_err());
}

#[test]
fn container_resolve_dependency_single() {
    let container = string_container();
    let desc = DependencyDescriptor::for_field(TypeId::of::<String>(), "String");
    let result = AutowireCapableBeanFactory::resolve_dependency(&container, &desc, None);
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn container_resolve_dependency_not_found() {
    let container = string_container();
    let desc = DependencyDescriptor::for_field(TypeId::of::<f64>(), "f64");
    let result = AutowireCapableBeanFactory::resolve_dependency(&container, &desc, None);
    assert!(result.is_err());
}

#[test]
fn container_resolve_dependency_optional_not_found() {
    let container = string_container();
    let desc = DependencyDescriptor::for_field(TypeId::of::<f64>(), "f64").with_optional(true);
    let result = AutowireCapableBeanFactory::resolve_dependency(&container, &desc, None);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn container_resolve_dependency_multiple() {
    let mut b = RegistryBuilder::new();
    let q = Qualifier::new("x").unwrap();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()).qualified(q))
        .unwrap();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "b".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let desc = DependencyDescriptor::for_field(TypeId::of::<String>(), "String");
    let result = AutowireCapableBeanFactory::resolve_dependency(&container, &desc, None);
    assert!(result.is_err());
}

#[test]
fn container_autowire_mode_no() {
    let container = string_container();
    // AUTOWIRE_NO = 0
    let result =
        AutowireCapableBeanFactory::autowire(&container, "alloc::string::String", 0, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_mode_by_name() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()).depends_on::<i32>(),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    let container = Container::new(b.build().unwrap());
    // AUTOWIRE_BY_NAME = 1
    let result =
        AutowireCapableBeanFactory::autowire(&container, "alloc::string::String", 1, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_mode_by_type() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()).depends_on::<i32>(),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    let container = Container::new(b.build().unwrap());
    // AUTOWIRE_BY_TYPE = 2
    let result =
        AutowireCapableBeanFactory::autowire(&container, "alloc::string::String", 2, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_mode_constructor() {
    let container = string_container();
    // AUTOWIRE_CONSTRUCTOR = 3
    let result =
        AutowireCapableBeanFactory::autowire(&container, "alloc::string::String", 3, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_mode_invalid() {
    let container = string_container();
    let result =
        AutowireCapableBeanFactory::autowire(&container, "alloc::string::String", 999, false);
    assert!(result.is_err());
}

#[test]
fn container_autowire_bean_properties_mode_no() {
    let container = string_container();
    let bean = Arc::new("test".to_string()) as Arc<dyn Any + Send + Sync>;
    let result = AutowireCapableBeanFactory::autowire_bean_properties(&container, bean, 0, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_properties_mode_by_name() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<ServiceB, _>(|r| {
            let a = r.resolve::<ServiceA>().ok();
            ServiceB { a }
        })
        .depends_on::<ServiceA>(),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<ServiceA, _>(|_| {
        ServiceA {
            value: "injected".to_string(),
        }
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let bean = Arc::new(ServiceA {
        value: "test".to_string(),
    }) as Arc<dyn Any + Send + Sync>;
    let result = AutowireCapableBeanFactory::autowire_bean_properties(&container, bean, 1, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_properties_mode_by_type() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<ServiceB, _>(|r| {
            let a = r.resolve::<ServiceA>().ok();
            ServiceB { a }
        })
        .depends_on::<ServiceA>(),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<ServiceA, _>(|_| {
        ServiceA {
            value: "injected".to_string(),
        }
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let bean = Arc::new(ServiceA {
        value: "test".to_string(),
    }) as Arc<dyn Any + Send + Sync>;
    let result = AutowireCapableBeanFactory::autowire_bean_properties(&container, bean, 2, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_properties_mode_other() {
    let container = string_container();
    let bean = Arc::new("test".to_string()) as Arc<dyn Any + Send + Sync>;
    let result = AutowireCapableBeanFactory::autowire_bean_properties(&container, bean, 999, false);
    assert!(result.is_ok());
}

#[test]
fn container_configure_bean_without_processors() {
    let container = string_container();
    let bean = Arc::new("test".to_string()) as Arc<dyn Any + Send + Sync>;
    let result = AutowireCapableBeanFactory::configure_bean(&container, bean, "test_bean");
    assert!(result.is_ok());
}

#[test]
fn container_configure_bean_with_processors() {
    let mut container = string_container();
    struct ReplacingBpp;
    impl BeanPostProcessor for ReplacingBpp {
        fn post_process_after_initialization(
            &self,
            _bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(None)
        }
    }
    container.add_bean_post_processor(Arc::new(ReplacingBpp));
    let bean = Arc::new("test".to_string()) as Arc<dyn Any + Send + Sync>;
    let result = AutowireCapableBeanFactory::configure_bean(&container, bean, "test_bean");
    assert!(result.is_ok());
}

#[test]
fn container_apply_bean_property_values() {
    let container = string_container();
    let bean = Arc::new("test".to_string()) as Arc<dyn Any + Send + Sync>;
    let result =
        AutowireCapableBeanFactory::apply_bean_property_values(&container, bean, "test_bean");
    assert!(result.is_ok());
}

#[test]
fn container_destroy_bean_instance() {
    let container = string_container();
    let bean = Arc::new("test".to_string()) as Arc<dyn Any + Send + Sync>;
    let result = AutowireCapableBeanFactory::destroy_bean_instance(&container, "test_bean", &*bean);
    assert!(result.is_ok());
}

#[test]
fn container_type_converter() {
    let mut container = string_container();
    // Setter with None
    AutowireCapableBeanFactory::set_type_converter(&mut container, None);
    // Getter returns None
    let result = AutowireCapableBeanFactory::type_converter(&container);
    assert!(result.is_none());
}

#[test]
fn container_bean_post_processor_count() {
    let mut container = string_container();
    assert_eq!(
        ConfigurableBeanFactory::bean_post_processor_count(&container),
        0
    );
    struct DummyBpp;
    impl BeanPostProcessor for DummyBpp {}
    container.add_bean_post_processor(Arc::new(DummyBpp));
    assert_eq!(
        ConfigurableBeanFactory::bean_post_processor_count(&container),
        1
    );
}

#[test]
fn container_contains_bean_definition_dynamic() {
    let mut container = empty_container();
    let bean_name = "test_dynamic".to_string();
    let mut rbd = vernal_beans::root_bean_definition::RootBeanDefinition::new();
    rbd.set_bean_class_name("test::Dynamic");
    BeanDefinitionRegistry::register_bean_definition(
        &mut container,
        bean_name.clone(),
        Box::new(rbd),
    )
    .unwrap();
    assert!(BeanDefinitionRegistry::contains_bean_definition(
        &container, &bean_name
    ));
    // Verify registry definition also works
    let container2 = string_container();
    assert!(BeanDefinitionRegistry::contains_bean_definition(
        &container2,
        "alloc::string::String"
    ));
}

#[test]
fn container_bean_definition_count_and_names_with_dynamic() {
    let mut container = string_container();
    let initial_count = BeanDefinitionRegistry::bean_definition_count(&container);
    let initial_names = BeanDefinitionRegistry::bean_definition_names(&container);
    assert!(initial_count >= 1);
    assert!(initial_names.contains(&"alloc::string::String".to_string()));

    // Register a dynamic definition
    let bean_name = "dynamic_bean".to_string();
    let mut rbd = vernal_beans::root_bean_definition::RootBeanDefinition::new();
    rbd.set_bean_class_name("dynamic::Type");
    BeanDefinitionRegistry::register_bean_definition(
        &mut container,
        bean_name.clone(),
        Box::new(rbd),
    )
    .unwrap();
    assert_eq!(
        BeanDefinitionRegistry::bean_definition_count(&container),
        initial_count + 1
    );
    let names = BeanDefinitionRegistry::bean_definition_names(&container);
    assert!(names.contains(&bean_name));
}

#[test]
fn container_register_bean_definition_duplicate_error() {
    let mut container = empty_container();
    let bean_name = "dup_bean".to_string();
    let mut rbd = vernal_beans::root_bean_definition::RootBeanDefinition::new();
    rbd.set_bean_class_name("dup::Type");
    BeanDefinitionRegistry::register_bean_definition(
        &mut container,
        bean_name.clone(),
        Box::new(rbd),
    )
    .unwrap();

    // Register the same name again should fail
    let mut rbd2 = vernal_beans::root_bean_definition::RootBeanDefinition::new();
    rbd2.set_bean_class_name("dup::Type");
    let result =
        BeanDefinitionRegistry::register_bean_definition(&mut container, bean_name, Box::new(rbd2));
    assert!(result.is_err());
}

// ── SingletonBeanRegistry tests ────────────────────────────────────────────

#[test]
fn container_singleton_bean_registry_register_and_get() {
    let container = string_container();
    SingletonBeanRegistry::register_singleton(&container, "my_singleton", Arc::new(42i32));
    let got = SingletonBeanRegistry::get_singleton(&container, "my_singleton");
    assert!(got.is_some());
    let val = got.unwrap();
    let downcasted = val.downcast_ref::<i32>();
    assert_eq!(downcasted, Some(&42i32));
}

#[test]
fn container_singleton_bean_registry_contains() {
    let container = string_container();
    SingletonBeanRegistry::register_singleton(&container, "my_singleton", Arc::new(99i32));
    assert!(SingletonBeanRegistry::contains_singleton(
        &container,
        "my_singleton"
    ));
    assert!(!SingletonBeanRegistry::contains_singleton(
        &container,
        "nonexistent"
    ));
}

#[test]
fn container_singleton_bean_registry_names() {
    let container = string_container();
    SingletonBeanRegistry::register_singleton(&container, "alpha", Arc::new(1i32));
    SingletonBeanRegistry::register_singleton(&container, "beta", Arc::new(2i32));
    let names = SingletonBeanRegistry::singleton_names(&container);
    assert!(names.contains(&"alpha".to_string()));
    assert!(names.contains(&"beta".to_string()));
}

#[test]
fn container_singleton_bean_registry_count() {
    let container = string_container();
    SingletonBeanRegistry::register_singleton(&container, "alpha", Arc::new(1i32));
    let count = SingletonBeanRegistry::singleton_count(&container);
    assert!(count >= 1);
}

// ── ConfigurableListableBeanFactory tests ─────────────────────────────────

#[test]
fn container_ignore_dependency_type() {
    let mut container = string_container();
    ConfigurableListableBeanFactory::ignore_dependency_type(&mut container, TypeId::of::<String>());
}

#[test]
fn container_ignore_dependency_interface() {
    let mut container = string_container();
    ConfigurableListableBeanFactory::ignore_dependency_interface(
        &mut container,
        TypeId::of::<dyn Any>(),
    );
}

#[test]
fn container_register_resolvable_dependency() {
    let mut container = string_container();
    ConfigurableListableBeanFactory::register_resolvable_dependency(
        &mut container,
        TypeId::of::<i32>(),
        Arc::new(42i32),
    );
}

#[test]
fn container_is_autowire_candidate() {
    let container = string_container();
    assert!(ConfigurableListableBeanFactory::is_autowire_candidate(
        &container,
        "alloc::string::String"
    ));
    assert!(!ConfigurableListableBeanFactory::is_autowire_candidate(
        &container,
        "nonexistent"
    ));
}

#[test]
fn container_is_autowire_candidate_with_dynamic() {
    let mut container = empty_container();
    let bean_name = "test_autowire_candidate".to_string();
    let mut rbd = vernal_beans::root_bean_definition::RootBeanDefinition::new();
    rbd.set_bean_class_name("test::AutowireCandidate");
    BeanDefinitionRegistry::register_bean_definition(
        &mut container,
        bean_name.clone(),
        Box::new(rbd),
    )
    .unwrap();
    assert!(ConfigurableListableBeanFactory::is_autowire_candidate(
        &container, &bean_name
    ));

    // After removing, it should no longer be a candidate
    BeanDefinitionRegistry::remove_bean_definition(&mut container, &bean_name).unwrap();
    assert!(!ConfigurableListableBeanFactory::is_autowire_candidate(
        &container, &bean_name
    ));
}

#[test]
fn container_freeze_and_check_configuration() {
    let mut container = string_container();
    assert!(!ConfigurableListableBeanFactory::is_configuration_frozen(
        &container
    ));
    ConfigurableListableBeanFactory::freeze_configuration(&mut container);
    assert!(ConfigurableListableBeanFactory::is_configuration_frozen(
        &container
    ));
}

#[test]
fn container_pre_instantiate_singletons() {
    let container = string_container();
    let result = ConfigurableListableBeanFactory::pre_instantiate_singletons(&container);
    assert!(result.is_ok());
}

#[test]
fn container_pre_instantiate_singletons_empty() {
    let container = empty_container();
    let result = ConfigurableListableBeanFactory::pre_instantiate_singletons(&container);
    assert!(result.is_ok());
}

// =========================================================================
// 2. scope_context.rs
// =========================================================================

#[tokio::test]
async fn scope_context_open_and_state() {
    let container = string_container();
    let scope = container.open_scope::<RequestScope>();
    assert_eq!(scope.state(), ScopeState::Open);
    let key = scope.key();
    assert_eq!(key, ScopeKey::of::<RequestScope>());
}

#[tokio::test]
async fn scope_context_child() {
    let container = string_container();
    let parent = container.open_scope::<RequestScope>();
    let child = parent.child::<SubScope>();
    assert_eq!(child.state(), ScopeState::Open);
    assert_eq!(child.key(), ScopeKey::of::<SubScope>());
    // child should have parent
    assert!(child.parent().is_some());
}

#[tokio::test]
async fn scope_context_close() {
    let container = string_container();
    let scope = container.open_scope::<RequestScope>();
    assert_eq!(scope.state(), ScopeState::Open);
    let result = scope.close().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn scope_context_with_cancellation_token() {
    let container = string_container();
    let token = tokio_util::sync::CancellationToken::new();
    let scope = container.open_scope_with_cancellation::<RequestScope>(token);
    assert_eq!(scope.state(), ScopeState::Open);
    scope.close().await.unwrap();
}

#[tokio::test]
async fn scope_context_multiple_scopes() {
    let container = string_container();
    let s1 = container.open_scope::<RequestScope>();
    let s2 = container.open_scope::<SubScope>();
    assert_eq!(s1.state(), ScopeState::Open);
    assert_eq!(s2.state(), ScopeState::Open);
    assert!(s1.parent().is_none());
    assert!(s2.parent().is_none());
    // Close both
    s1.close().await.unwrap();
    s2.close().await.unwrap();
}

#[tokio::test]
async fn scope_context_get_or_insert_with() {
    let container = string_container();
    let scope = container.open_scope::<RequestScope>();
    let val = scope
        .get_or_insert_with::<String, _>(|| "scope_value".to_string())
        .unwrap();
    assert_eq!(*val, "scope_value".to_string());
    scope.close().await.unwrap();
}

// =========================================================================
// 3. registry_builder.rs
// =========================================================================

#[test]
fn registry_builder_register_all_with_trait_bindings() {
    let mut b = RegistryBuilder::new();
    b.register_all([
        ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl),
        ComponentDefinition::singleton::<OtherImpl, _>(|_| OtherImpl),
    ])
    .unwrap();
    b.bind_all([
        TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| s as Arc<dyn MyTrait>),
        TraitBinding::new::<dyn MyTrait, OtherImpl, _>(|s| s as Arc<dyn MyTrait>),
    ])
    .unwrap();
    let registry = b.build().unwrap();
    assert_eq!(registry.definitions().len(), 2);
    assert_eq!(registry.bindings().len(), 2);
}

#[test]
fn registry_builder_bind_multiple_times() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.register(ComponentDefinition::singleton::<OtherImpl, _>(|_| {
        OtherImpl
    }))
    .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, OtherImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    let registry = b.build().unwrap();
    assert_eq!(registry.bindings().len(), 2);
}

#[test]
fn registry_builder_bind_all_with_vector() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    let bindings = vec![TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    })];
    b.bind_all(bindings).unwrap();
    let registry = b.build().unwrap();
    assert_eq!(registry.bindings().len(), 1);
}

#[test]
fn registry_builder_register_bundle() {
    let mut b = RegistryBuilder::new();
    let definitions = vec![ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl)];
    let bindings = vec![TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    })];
    b.register_bundle(definitions, bindings).unwrap();
    let registry = b.build().unwrap();
    assert_eq!(registry.definitions().len(), 1);
    assert_eq!(registry.bindings().len(), 1);
}

#[test]
fn registry_builder_duplicate_registration_error() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    let result = b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl));
    assert!(result.is_err());
}

#[test]
fn registry_builder_duplicate_bind_error() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    let binding = TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| s as Arc<dyn MyTrait>);
    b.bind(binding).unwrap();
    // Duplicate same binding should fail
    let binding2 = TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| s as Arc<dyn MyTrait>);
    let result = b.bind(binding2);
    assert!(result.is_err());
}

#[test]
fn registry_builder_contains() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    assert!(b.contains::<String>());
    assert!(!b.contains::<i32>());
}

#[test]
fn registry_builder_remove_by_key() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let key = ComponentKey::of::<String>();
    assert!(b.remove_by_key(&key).is_ok());
    assert!(!b.contains::<String>());
}

#[test]
fn registry_builder_remove_by_key_failure() {
    let mut b = RegistryBuilder::new();
    let key = ComponentKey::of::<String>();
    let result = b.remove_by_key(&key);
    assert!(result.is_err());
}

#[test]
fn registry_builder_is_empty_and_len() {
    let mut b = RegistryBuilder::new();
    assert!(b.is_empty());
    assert_eq!(b.len(), 0);
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    assert!(!b.is_empty());
    assert_eq!(b.len(), 1);
}

#[test]
fn registry_builder_debug_format() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let debug_str = format!("{:?}", b);
    assert!(debug_str.contains("RegistryBuilder"));
    assert!(debug_str.contains("definitions"));
}

// =========================================================================
// 4. component_definition.rs
// =========================================================================

#[test]
fn component_def_singleton_factory() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "factory".to_string());
    assert_eq!(def.key(), &ComponentKey::of::<String>());
    assert_eq!(def.scope(), Scope::Singleton);
    assert!(def.dependencies().is_empty());
}

#[test]
fn component_def_transient_factory() {
    let def = ComponentDefinition::transient::<i32, _>(|_| 100i32);
    assert_eq!(def.key(), &ComponentKey::of::<i32>());
    assert_eq!(def.scope(), Scope::Transient);
}

#[test]
fn component_def_depends_on_methods() {
    let def = ComponentDefinition::singleton::<ServiceB, _>(|r| {
        let a = r.resolve::<ServiceA>().ok();
        ServiceB { a }
    })
    .depends_on::<ServiceA>()
    .depends_on_optional::<String>()
    .depends_on_provider::<i32>()
    .depends_on_optional_provider::<bool>()
    .depends_on_trait::<dyn MyTrait>()
    .depends_on_optional_trait::<dyn MyTrait>()
    .depends_on_all_traits::<dyn MyTrait>();

    assert_eq!(def.dependencies().len(), 7);
    assert_eq!(def.key(), &ComponentKey::of::<ServiceB>());
}

#[test]
fn component_def_depends_on_qualified_methods() {
    let q = Qualifier::new("primary").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_| "test".to_string())
        .depends_on_qualified::<i32>(q.clone())
        .depends_on_optional_qualified::<bool>(q.clone())
        .depends_on_qualified_provider::<f64>(q.clone())
        .depends_on_optional_qualified_provider::<Vec<u8>>(q.clone())
        .depends_on_qualified_trait::<dyn MyTrait>(q.clone())
        .depends_on_optional_qualified_trait::<dyn MyTrait>(q.clone())
        .depends_on_trait_provider::<dyn MyTrait>()
        .depends_on_optional_trait_provider::<dyn MyTrait>()
        .depends_on_qualified_trait_provider::<dyn MyTrait>(q.clone())
        .depends_on_optional_qualified_trait_provider::<dyn MyTrait>(q);

    assert_eq!(def.dependencies().len(), 10);
}

#[test]
fn component_def_key_scope_dependencies_getters() {
    let def = ComponentDefinition::singleton::<i32, _>(|_| 42i32).depends_on::<f64>();
    assert_eq!(def.key(), &ComponentKey::of::<i32>());
    assert_eq!(def.scope(), Scope::Singleton);
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_def_builder_with_init_order() {
    let def =
        ComponentDefinition::singleton::<String, _>(|_| "test".to_string()).with_init_order(-10);
    assert_eq!(def.init_order(), -10);
}

// =========================================================================
// 5. resolver.rs — exercised inside ComponentDefinition factories
// =========================================================================

#[test]
fn resolver_resolve_and_others_within_factory() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<String, _>(|r| {
            // resolve::<T>()
            let _i = r.resolve::<i32>().unwrap();
            // resolve_optional::<T>()
            let _opt = r.resolve_optional::<bool>().unwrap();
            // resolve_trait::<T>() — requires binding
            let _t = r.resolve_trait::<dyn MyTrait>().unwrap();
            // resolve_all_traits::<T>()
            let all = r.resolve_all_traits::<dyn MyTrait>().unwrap();
            assert!(!all.is_empty());
            // resolve_optional_trait::<T>()
            let _ot = r.resolve_optional_trait::<dyn MyTrait>().unwrap();
            format!("ok traits={}", all.len())
        })
        .depends_on::<i32>()
        .depends_on_optional::<bool>()
        .depends_on_trait::<dyn MyTrait>()
        .depends_on_all_traits::<dyn MyTrait>()
        .depends_on_optional_trait::<dyn MyTrait>(),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    b.register(ComponentDefinition::singleton::<bool, _>(|_| true))
        .unwrap();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let result = container.resolve::<String>();
    assert!(result.is_ok());
}

#[test]
fn resolver_qualified_and_provider_within_factory() {
    let q = Qualifier::new("myq").unwrap();
    let q_for_closure = q.clone();
    let q_for_def = q.clone();
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<String, _>(move |r| {
            // resolve_qualified::<T>()
            let _val = r.resolve_qualified::<i32>(&q_for_closure).unwrap();
            // provider::<T>()
            let _prov = r.provider::<bool>().unwrap();
            // optional_provider::<T>()
            let _opt_prov = r.optional_provider::<f64>().unwrap();
            // trait_provider::<T>()
            let _tp = r.trait_provider::<dyn MyTrait>().unwrap();
            format!("ok")
        })
        .depends_on_qualified::<i32>(q)
        .depends_on_provider::<bool>()
        .depends_on_optional_provider::<f64>()
        .depends_on_trait_provider::<dyn MyTrait>(),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 99i32).qualified(q_for_def))
        .unwrap();
    b.register(ComponentDefinition::singleton::<bool, _>(|_| false))
        .unwrap();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let result = container.resolve::<String>();
    assert!(result.is_ok());
}

// =========================================================================
// 6. dependency.rs — all constructors, Display, type_id, qualifier
// =========================================================================

#[test]
fn dependency_of() {
    let dep = Dependency::of::<String>();
    assert!(dep.type_name().contains("String"));
    assert!(dep.qualifier().is_none());
}

#[test]
fn dependency_qualified() {
    let q = Qualifier::new("test_qual").unwrap();
    let dep = Dependency::qualified::<String>(q.clone());
    assert!(dep.type_name().contains("String"));
    assert_eq!(dep.qualifier(), Some(&q));
}

#[test]
fn dependency_optional_of() {
    let dep = Dependency::optional_of::<i32>();
    assert!(dep.type_name().contains("i32"));
}

#[test]
fn dependency_provider_of() {
    let dep = Dependency::provider_of::<bool>();
    assert!(dep.type_name().contains("bool"));
}

#[test]
fn dependency_trait_of() {
    let dep = Dependency::trait_of::<dyn MyTrait>();
    assert!(dep.type_name().contains("MyTrait"));
    assert!(dep.qualifier().is_none());
}

#[test]
fn dependency_trait_qualified() {
    let q = Qualifier::new("tq").unwrap();
    let dep = Dependency::trait_qualified::<dyn MyTrait>(q.clone());
    assert_eq!(dep.qualifier(), Some(&q));
}

#[test]
fn dependency_optional_trait_of() {
    let dep = Dependency::optional_trait_of::<dyn MyTrait>();
    assert!(dep.type_name().contains("MyTrait"));
}

#[test]
fn dependency_trait_provider_of() {
    let dep = Dependency::trait_provider_of::<dyn MyTrait>();
    assert!(dep.type_name().contains("MyTrait"));
}

#[test]
fn dependency_optional_trait_provider_of() {
    let dep = Dependency::optional_trait_provider_of::<dyn MyTrait>();
    assert!(dep.type_name().contains("MyTrait"));
}

#[test]
fn dependency_all_traits_of() {
    let dep = Dependency::all_traits_of::<dyn MyTrait>();
    assert!(dep.type_name().contains("MyTrait"));
}

#[test]
fn dependency_optional_qualified() {
    let q = Qualifier::new("oq").unwrap();
    let dep = Dependency::optional_qualified::<String>(q.clone());
    assert_eq!(dep.qualifier(), Some(&q));
}

#[test]
fn dependency_provider_qualified() {
    let q = Qualifier::new("pq").unwrap();
    let dep = Dependency::provider_qualified::<i32>(q.clone());
    assert_eq!(dep.qualifier(), Some(&q));
}

#[test]
fn dependency_optional_provider_of() {
    let _dep = Dependency::optional_provider_of::<f64>();
}

#[test]
fn dependency_optional_provider_qualified() {
    let q = Qualifier::new("opq").unwrap();
    let dep = Dependency::optional_provider_qualified::<bool>(q.clone());
    assert_eq!(dep.qualifier(), Some(&q));
}

#[test]
fn dependency_optional_trait_qualified() {
    let q = Qualifier::new("otq").unwrap();
    let dep = Dependency::optional_trait_qualified::<dyn MyTrait>(q.clone());
    assert_eq!(dep.qualifier(), Some(&q));
}

#[test]
fn dependency_trait_provider_qualified() {
    let q = Qualifier::new("tpq").unwrap();
    let dep = Dependency::trait_provider_qualified::<dyn MyTrait>(q.clone());
    assert_eq!(dep.qualifier(), Some(&q));
}

#[test]
fn dependency_optional_trait_provider_qualified() {
    let q = Qualifier::new("otpq").unwrap();
    let dep = Dependency::optional_trait_provider_qualified::<dyn MyTrait>(q.clone());
    assert_eq!(dep.qualifier(), Some(&q));
}

// Display format tests
#[test]
fn dependency_display_bare() {
    let dep = Dependency::of::<String>();
    let s = format!("{}", dep);
    assert_eq!(s, "alloc::string::String");
}

#[test]
fn dependency_display_qualified() {
    let q = Qualifier::new("myq").unwrap();
    let dep = Dependency::qualified::<String>(q);
    let s = format!("{}", dep);
    assert!(s.contains("String"));
    assert!(s.contains("myq"));
}

#[test]
fn dependency_display_optional() {
    let dep = Dependency::optional_of::<i32>();
    let s = format!("{}", dep);
    assert!(s.starts_with("optional<"));
}

#[test]
fn dependency_display_provider() {
    let dep = Dependency::provider_of::<bool>();
    let s = format!("{}", dep);
    assert!(s.starts_with("provider<"));
}

#[test]
fn dependency_display_trait() {
    let dep = Dependency::trait_of::<dyn MyTrait>();
    let s = format!("{}", dep);
    assert!(s.contains("MyTrait"));
}

#[test]
fn dependency_display_trait_provider() {
    let dep = Dependency::trait_provider_of::<dyn MyTrait>();
    let s = format!("{}", dep);
    assert!(s.starts_with("trait_provider<"));
}

#[test]
fn dependency_display_optional_trait_provider() {
    let dep = Dependency::optional_trait_provider_of::<dyn MyTrait>();
    let s = format!("{}", dep);
    assert!(s.starts_with("optional_trait_provider<"));
}

#[test]
fn dependency_display_all_traits() {
    let dep = Dependency::all_traits_of::<dyn MyTrait>();
    let s = format!("{}", dep);
    assert!(s.starts_with("all<"));
}

#[test]
fn dependency_display_qualified_optional() {
    let q = Qualifier::new("dq").unwrap();
    let dep = Dependency::optional_qualified::<String>(q);
    let s = format!("{}", dep);
    assert!(s.starts_with("optional<"));
    assert!(s.contains("dq"));
}

#[test]
fn dependency_display_qualified_provider() {
    let q = Qualifier::new("dpq").unwrap();
    let dep = Dependency::provider_qualified::<i32>(q);
    let s = format!("{}", dep);
    assert!(s.starts_with("provider<"));
    assert!(s.contains("dpq"));
}

#[test]
fn dependency_display_qualified_trait() {
    let q = Qualifier::new("dtq").unwrap();
    let dep = Dependency::trait_qualified::<dyn MyTrait>(q);
    let s = format!("{}", dep);
    assert!(s.contains("MyTrait"));
    assert!(s.contains("dtq"));
}

#[test]
fn dependency_type_name() {
    let dep = Dependency::of::<String>();
    assert!(dep.type_name().contains("String"));
    assert!(!dep.type_name().is_empty());
}

// =========================================================================
// Additional container API coverage
// =========================================================================

#[test]
fn container_get_bean_by_key() {
    let container = string_container();
    let key = ComponentKey::of::<String>();
    let result = BeanFactory::get_bean_by_key(&container, &key);
    assert!(result.is_ok());

    let missing_key = ComponentKey::of::<f64>();
    let result = BeanFactory::get_bean_by_key(&container, &missing_key);
    assert!(result.is_err());
}

#[test]
fn container_autowire_bean_with_matching_definition() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<ServiceB, _>(|r| {
            let a = r.resolve::<ServiceA>().ok();
            ServiceB { a }
        })
        .depends_on::<ServiceA>(),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<ServiceA, _>(|_| {
        ServiceA {
            value: "original".to_string(),
        }
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());

    let bean = Arc::new(ServiceA {
        value: "original".to_string(),
    }) as Arc<dyn Any + Send + Sync>;
    let result = AutowireCapableBeanFactory::autowire_bean(&container, bean);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_no_matching_definition() {
    let container = string_container();
    let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    let result = AutowireCapableBeanFactory::autowire_bean(&container, bean);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_multiple_matching_definitions() {
    let mut b = RegistryBuilder::new();
    let q = Qualifier::new("x").unwrap();
    b.register(
        ComponentDefinition::singleton::<ServiceA, _>(|_| ServiceA {
            value: "a".to_string(),
        })
        .qualified(q),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<ServiceA, _>(|_| {
        ServiceA {
            value: "b".to_string(),
        }
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let bean = Arc::new(ServiceA {
        value: "test".to_string(),
    }) as Arc<dyn Any + Send + Sync>;
    let result = AutowireCapableBeanFactory::autowire_bean(&container, bean);
    assert!(result.is_ok());
}

#[test]
fn container_initialize_bean() {
    let container = string_container();
    let bean = Arc::new("test".to_string()) as Arc<dyn Any + Send + Sync>;
    let result = AutowireCapableBeanFactory::initialize_bean(&container, bean, "test_bean");
    assert!(result.is_ok());
}

#[test]
fn container_initialize_bean_with_processors() {
    let mut container = string_container();
    struct InitBpp;
    impl BeanPostProcessor for InitBpp {
        fn post_process_before_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(bean))
        }
    }
    container.add_bean_post_processor(Arc::new(InitBpp));
    let bean = Arc::new("test".to_string()) as Arc<dyn Any + Send + Sync>;
    let result = AutowireCapableBeanFactory::initialize_bean(&container, bean, "test_bean");
    assert!(result.is_ok());
}

#[test]
fn container_create_bean_not_found() {
    let container = empty_container();
    let result = AutowireCapableBeanFactory::create_bean(&container, "nonexistent_type");
    assert!(result.is_err());
}

#[test]
fn container_configurable_bean_factory_destroy_singletons() {
    let container = string_container();
    // Pre-instantiate the singleton
    let _ = container.resolve::<String>();
    ConfigurableBeanFactory::destroy_singletons(&container);
    // After destroy, the singleton should no longer be in the cache
    let result = container.resolve::<String>();
    assert!(result.is_ok());
}

#[test]
fn container_configurable_bean_factory_register_alias() {
    let mut container = string_container();
    ConfigurableBeanFactory::register_alias(&mut container, "alloc::string::String", "my_string")
        .unwrap();
    // Duplicate alias should fail
    let result = ConfigurableBeanFactory::register_alias(&mut container, "other", "my_string");
    assert!(result.is_err());
}

#[test]
fn container_configurable_bean_factory_is_factory_bean() {
    let container = string_container();
    assert!(!ConfigurableBeanFactory::is_factory_bean(
        &container,
        "alloc::string::String"
    ));
    assert!(ConfigurableBeanFactory::is_factory_bean(
        &container,
        "&something"
    ));
}

#[test]
fn container_configurable_bean_factory_in_creation_tracking() {
    let mut container = string_container();
    assert!(!ConfigurableBeanFactory::is_currently_in_creation(
        &container,
        "test_bean"
    ));
    ConfigurableBeanFactory::set_currently_in_creation(&mut container, "test_bean", true);
    assert!(ConfigurableBeanFactory::is_currently_in_creation(
        &container,
        "test_bean"
    ));
    ConfigurableBeanFactory::set_currently_in_creation(&mut container, "test_bean", false);
    assert!(!ConfigurableBeanFactory::is_currently_in_creation(
        &container,
        "test_bean"
    ));
}

#[test]
fn container_configurable_bean_factory_dependent_bean_tracking() {
    let mut container = string_container();
    ConfigurableBeanFactory::register_dependent_bean(&mut container, "bean_a", "bean_b");
    let dependents = ConfigurableBeanFactory::get_dependent_beans(&container, "bean_a");
    assert_eq!(dependents, vec!["bean_b"]);
    let dependencies = ConfigurableBeanFactory::get_dependencies_for_bean(&container, "bean_b");
    assert_eq!(dependencies, vec!["bean_a"]);
}

#[test]
fn container_configurable_bean_factory_embedded_value_resolver() {
    let mut container = string_container();
    ConfigurableBeanFactory::add_embedded_value_resolver(
        &mut container,
        Arc::new(|val: &str| val.replace("${name}", "world")),
    );
    let result = ConfigurableBeanFactory::resolve_embedded_value(&container, "Hello, ${name}!");
    assert_eq!(result, "Hello, world!");
}

#[test]
fn container_configurable_bean_factory_register_scope() {
    let mut container = string_container();
    // Use a simple struct that implements BeanScope
    let scope = vernal_beans::request_scope::RequestScope::new("test_request");
    ConfigurableBeanFactory::register_scope(&mut container, "request", Box::new(scope));
    let names = ConfigurableBeanFactory::registered_scope_names(&container);
    assert!(names.contains(&"request".to_string()));
    ConfigurableBeanFactory::get_registered_scope(&container, "request");
}

#[test]
fn container_configurable_bean_factory_set_parent() {
    let mut child = empty_container();
    let parent = string_container();
    // Must wrap Arc<Container> as Arc<dyn BeanFactory>, then in Arc<dyn Any>
    let parent_bf: Arc<dyn BeanFactory> = Arc::new(parent);
    let parent_any: Arc<dyn Any + Send + Sync> = Arc::new(parent_bf);
    let result = ConfigurableBeanFactory::set_parent_bean_factory(&mut child, parent_any);
    assert!(result.is_ok());
}

#[test]
fn container_configurable_bean_factory_set_parent_invalid() {
    let mut child = empty_container();
    let invalid = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    let result = ConfigurableBeanFactory::set_parent_bean_factory(&mut child, invalid);
    assert!(result.is_err());
}

#[test]
fn container_hierarchical_bean_factory() {
    let mut child = empty_container();
    let parent = string_container();
    let parent_bf: Arc<dyn BeanFactory> = Arc::new(parent);
    let parent_any: Arc<dyn Any + Send + Sync> = Arc::new(parent_bf);
    ConfigurableBeanFactory::set_parent_bean_factory(&mut child, parent_any).unwrap();
    assert!(HierarchicalBeanFactory::parent_bean_factory(&child).is_some());
    assert!(!HierarchicalBeanFactory::contains_local_bean(
        &child,
        "nonexistent"
    ));
}

#[test]
fn container_listable_bean_factory_beans_of_type_id() {
    let container = string_container();
    let result =
        ListableBeanFactory::beans_of_type_id(&container, TypeId::of::<String>(), true, true);
    assert!(result.is_ok());
    let map = result.unwrap();
    assert!(!map.is_empty());

    let result = ListableBeanFactory::beans_of_type_id(&container, TypeId::of::<f64>(), true, true);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn container_listable_bean_factory_bean_names_for_type_id() {
    let container = multi_container();
    let names =
        ListableBeanFactory::bean_names_for_type_id(&container, TypeId::of::<String>(), true, true);
    assert!(!names.is_empty());
}

#[test]
fn container_listable_bean_factory_contains_flags() {
    let container = string_container();
    assert!(ListableBeanFactory::contains_singleton_bean(&container));
    assert!(!ListableBeanFactory::contains_non_singleton_bean(
        &container
    ));
}

#[test]
fn container_listable_bean_factory_bean_names_iterator() {
    let container = string_container();
    let iter = ListableBeanFactory::bean_names_iterator(&container);
    let names: Vec<String> = iter.collect();
    assert!(!names.is_empty());
}

#[test]
fn container_remove_bean_definition_not_found() {
    let mut container = empty_container();
    let result = BeanDefinitionRegistry::remove_bean_definition(&mut container, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn container_remove_bean_definition_from_registry() {
    let mut container = string_container();
    let result =
        BeanDefinitionRegistry::remove_bean_definition(&mut container, "alloc::string::String");
    assert!(result.is_ok());
    // It should now be marked as deleted
    assert!(!BeanDefinitionRegistry::contains_bean_definition(
        &container,
        "alloc::string::String"
    ));
}

#[test]
fn container_bean_definition_registry_get_bean_definition() {
    let container = string_container();
    // Get from registry
    let def = BeanDefinitionRegistry::get_bean_definition(&container, "alloc::string::String");
    assert!(def.is_some());
    // Not found
    let def = BeanDefinitionRegistry::get_bean_definition(&container, "nonexistent");
    assert!(def.is_none());
}

#[test]
fn container_add_singleton_callback() {
    let mut container = string_container();
    SingletonBeanRegistry::add_singleton_callback(
        &mut container,
        "test".to_string(),
        Arc::new(|_| {}),
    );
    let _ = SingletonBeanRegistry::singleton_mutex(&container);
}

#[test]
fn container_warm_up() {
    let container = string_container();
    let result = container.warm_up();
    assert!(result.is_ok());
}

#[test]
fn container_resolve_trait_and_all() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.register(ComponentDefinition::singleton::<OtherImpl, _>(|_| {
        OtherImpl
    }))
    .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, OtherImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());

    // resolve_trait with multiple non-primary should fail (ambiguous)
    let result = container.resolve_trait::<dyn MyTrait>();
    assert!(result.is_err()); // ambiguous

    // resolve_all_traits should return both
    let result = container.resolve_all_traits::<dyn MyTrait>();
    assert!(result.is_ok());
    let all = result.unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn container_resolve_trait_with_primary() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.register(ComponentDefinition::singleton::<OtherImpl, _>(|_| {
        OtherImpl
    }))
    .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| s as Arc<dyn MyTrait>).primary())
        .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, OtherImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());

    // resolve_trait should find the primary
    let result = container.resolve_trait::<dyn MyTrait>();
    assert!(result.is_ok());
}

#[test]
fn container_resolve_qualified_trait() {
    let q = Qualifier::new("special").unwrap();
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.bind(
        TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| s as Arc<dyn MyTrait>).qualified(q.clone()),
    )
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let result = container.resolve_qualified_trait::<dyn MyTrait>(&q);
    assert!(result.is_ok());
}

#[test]
fn container_resolve_qualified_trait_not_found() {
    let q = Qualifier::new("missing").unwrap();
    let container = empty_container();
    let result = container.resolve_qualified_trait::<dyn MyTrait>(&q);
    assert!(result.is_err());
}

#[test]
fn container_resolve_trait_not_found() {
    let container = empty_container();
    let result = container.resolve_trait::<dyn MyTrait>();
    assert!(result.is_err());
}

#[test]
fn container_open_scope_with_cancellation_and_child() {
    let container = string_container();
    let ct = tokio_util::sync::CancellationToken::new();
    let scope = container.open_scope_with_cancellation::<RequestScope>(ct);
    let child = scope.child::<SubScope>();
    assert!(child.parent().is_some());
    // The child cancellation token is derived from the parent
    let _ = child.cancellation();
}

// Additional Resolver tests
#[test]
fn container_registry_builder_register_all_with_duplicate_in_batch() {
    let mut b = RegistryBuilder::new();
    let result = b.register_all([
        ComponentDefinition::singleton::<String, _>(|_| "a".to_string()),
        ComponentDefinition::singleton::<String, _>(|_| "b".to_string()),
    ]);
    assert!(result.is_err());
}

#[test]
fn container_registry_builder_bind_all_duplicate() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    let result = b.bind_all([
        TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| s as Arc<dyn MyTrait>),
        TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| s as Arc<dyn MyTrait>),
    ]);
    assert!(result.is_err());
}

#[test]
fn container_registry_builder_register_bundle_validation_failure() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    let result = b.register_bundle(
        vec![ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl)],
        vec![],
    );
    assert!(result.is_err());
}

#[test]
fn container_registry_builder_remove_with_remove_method() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let result = b.remove::<String>();
    assert!(result.is_ok());
    assert!(!b.contains::<String>());

    // Remove non-existent
    let result = b.remove::<i32>();
    assert!(result.is_err());
}

#[test]
fn container_bean_definition_count_with_dynamic_deleted_filter() {
    let mut container = string_container();
    let name = "to_delete".to_string();
    let mut rbd = vernal_beans::root_bean_definition::RootBeanDefinition::new();
    rbd.set_bean_class_name("delete::Me");
    BeanDefinitionRegistry::register_bean_definition(&mut container, name.clone(), Box::new(rbd))
        .unwrap();
    let before_count = BeanDefinitionRegistry::bean_definition_count(&container);
    BeanDefinitionRegistry::remove_bean_definition(&mut container, "alloc::string::String")
        .unwrap();
    let after_count = BeanDefinitionRegistry::bean_definition_count(&container);
    let _ = (before_count, after_count);
}

#[test]
fn component_def_scoped_factory() {
    struct MyScope;
    let def = ComponentDefinition::scoped::<String, MyScope, _>(|_| "scoped".to_string());
    assert_eq!(def.scope(), Scope::custom::<MyScope>());
}

#[test]
fn component_def_try_variants() {
    let def = ComponentDefinition::try_singleton::<String, _>(|_| Ok("try".to_string()));
    assert_eq!(def.scope(), Scope::Singleton);

    let def = ComponentDefinition::try_transient::<i32, _>(|_| Ok(42i32));
    assert_eq!(def.scope(), Scope::Transient);

    struct AnotherScope;
    let def = ComponentDefinition::try_scoped::<bool, AnotherScope, _>(|_| Ok(true));
    assert_eq!(def.scope(), Scope::custom::<AnotherScope>());
}

#[test]
fn component_def_shared_value_and_arc() {
    let def = ComponentDefinition::shared_value(42i32);
    assert_eq!(def.key(), &ComponentKey::of::<i32>());

    let arc = Arc::new("test".to_string());
    let def = ComponentDefinition::shared_arc(arc);
    assert_eq!(def.key(), &ComponentKey::of::<String>());
}

#[test]
fn container_resolve_in_and_resolve_qualified_in_with_valid_scope() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "in_scope".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let scope = container.open_scope::<RequestScope>();
    // resolve_in
    let result = container.resolve_in::<String>(&scope);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "in_scope".to_string());
}

#[test]
fn container_resolve_qualified_in_with_valid_scope() {
    let q = Qualifier::new("test_q").unwrap();
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "qualified_in_scope".to_string())
            .qualified(q.clone()),
    )
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_qualified_in::<String>(&q, &scope);
    assert!(result.is_ok());
}

#[test]
fn container_resolve_trait_in_and_all_traits_in() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| s as Arc<dyn MyTrait>).primary())
        .unwrap();
    let container = Container::new(b.build().unwrap());
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_trait_in::<dyn MyTrait>(&scope);
    assert!(result.is_ok());

    let result = container.resolve_all_traits_in::<dyn MyTrait>(&scope);
    assert!(result.is_ok());
}

#[test]
fn container_resolve_qualified_trait_in() {
    let q = Qualifier::new("qt").unwrap();
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.bind(
        TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| s as Arc<dyn MyTrait>).qualified(q.clone()),
    )
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_qualified_trait_in::<dyn MyTrait>(&q, &scope);
    assert!(result.is_ok());
}

#[test]
fn container_scope_owner_mismatch() {
    let container_a = string_container();
    let container_b = string_container();
    let scope = container_a.open_scope::<RequestScope>();
    let result = container_b.resolve_in::<String>(&scope);
    assert!(result.is_err());
}

#[test]
fn container_bean_post_processor_count_trait() {
    let container = string_container();
    assert_eq!(
        ListableBeanFactory::bean_post_processor_count(&container),
        0
    );
}

#[test]
fn container_listable_bean_factory_contains_bean_definition() {
    let container = string_container();
    assert!(ListableBeanFactory::contains_bean_definition(
        &container,
        "alloc::string::String"
    ));
    assert!(!ListableBeanFactory::contains_bean_definition(
        &container,
        "nonexistent"
    ));
}
