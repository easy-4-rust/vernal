//! Coverage Phase 4: targeted tests for specific uncovered code paths
//! in container.rs, abstract_autowire_capable_bean_factory.rs,
//! registry_builder.rs, scope_context.rs, and resolver.rs.

use std::any::{Any, TypeId};
use std::sync::Arc;

use vernal_beans::{
    BeanDefinition, BeanDefinitionRegistry, BeanFactory, BeanPostProcessor, ComponentDefinition,
    ComponentKey, ConfigurableBeanFactory, ConfigurableListableBeanFactory, Container,
    ListableBeanFactory, Qualifier, RegistryBuilder, ResolveError, Resolver, Scope, ScopeKey,
    ScopeState, SingletonBeanRegistry, TraitBinding,
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

struct RequestScopeTag;
struct SubScopeTag;

// ═══════════════════════════════════════════════════════════════════════════════
// 1. container.rs
// ═══════════════════════════════════════════════════════════════════════════════

// ---------------------------------------------------------------------------
// 1a. ResolveError formatting for paths we cannot easily trigger at runtime
// ---------------------------------------------------------------------------

#[test]
fn resolve_error_circular_runtime_format() {
    let err = ResolveError::CircularRuntime {
        path: vec!["A".to_string(), "B".to_string(), "A".to_string()],
    };
    let msg = format!("{err}");
    assert!(!msg.is_empty());
    // The message should mention the circular dependency
    assert!(
        msg.contains("circular")
            || msg.contains("Circular")
            || msg.contains("cycle")
            || msg.contains("stack")
    );
}

#[test]
fn resolve_error_trait_binding_type_mismatch_format() {
    let tk = vernal_beans::TraitKey::of::<dyn MyTrait>();
    let ck = ComponentKey::of::<MyImpl>();
    let err = ResolveError::TraitBindingTypeMismatch {
        binding: tk,
        target: ck,
    };
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

// ---------------------------------------------------------------------------
// 1b. select_trait_binding — ambiguous (multiple non-primary, no qualifier)
// ---------------------------------------------------------------------------

#[test]
fn select_trait_binding_ambiguous() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.register(ComponentDefinition::singleton::<OtherImpl, _>(|_| {
        OtherImpl
    }))
    .unwrap();
    // Two bindings, neither primary, no qualifier
    b.bind(TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, OtherImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let result = container.resolve_trait::<dyn MyTrait>();
    assert!(result.is_err());
    match result {
        Err(ResolveError::Ambiguous { .. }) => {} // expected
        _ => panic!("Expected Ambiguous error"),
    }
}

// ---------------------------------------------------------------------------
// 1c. ContainerObjectProvider get / if_available / get_if_unique / stream / ordered_stream
// ---------------------------------------------------------------------------

#[test]
fn container_object_provider_get_with_singleton() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    // Resolve first so there is a singleton cell
    let _ = container.resolve::<String>().unwrap();

    let provider = BeanFactory::get_bean_provider_by_type_id(&container, TypeId::of::<String>())
        .expect("provider");
    let result = provider.get();
    assert!(result.is_ok());
    // Check the value
    let val = result.unwrap();
    let s = val.downcast_ref::<String>();
    assert_eq!(s, Some(&"hello".to_string()));
}

#[test]
fn container_object_provider_if_available() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let _ = container.resolve::<String>().unwrap();

    let provider = BeanFactory::get_bean_provider_by_type_id(&container, TypeId::of::<String>())
        .expect("provider");
    let result = provider.if_available();
    assert!(result.is_some());
}

#[test]
fn container_object_provider_get_if_unique() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let _ = container.resolve::<String>().unwrap();

    let provider = BeanFactory::get_bean_provider_by_type_id(&container, TypeId::of::<String>())
        .expect("provider");
    let result = provider.get_if_unique();
    assert!(result.is_ok());
}

#[test]
fn container_object_provider_stream() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    let container = Container::new(b.build().unwrap());
    // Resolve both to populate singleton cache
    let _ = container.resolve::<String>().unwrap();
    let _ = container.resolve::<i32>().unwrap();

    let provider = BeanFactory::get_bean_provider_by_type_id(&container, TypeId::of::<String>())
        .expect("provider");
    let items = provider.stream();
    // stream returns all singleton values regardless of type_id
    assert!(!items.is_empty());
}

#[test]
fn container_object_provider_ordered_stream() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let _ = container.resolve::<String>().unwrap();

    let provider = BeanFactory::get_bean_provider_by_type_id(&container, TypeId::of::<String>())
        .expect("provider");
    let items = provider.ordered_stream();
    assert!(!items.is_empty());
}

// ---------------------------------------------------------------------------
// 1d. SingletonBeanRegistry OnceLock fallback path
// ---------------------------------------------------------------------------

#[test]
fn singleton_registry_get_singleton_once_lock_fallback() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "from_registry".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    // Resolve via container (populates OnceLock)
    let _ = container.resolve::<String>().unwrap();
    // get_singleton should find it via OnceLock by type_name
    let result = SingletonBeanRegistry::get_singleton(&container, "alloc::string::String");
    assert!(result.is_some());
    let val = result.unwrap();
    let s = val.downcast_ref::<String>();
    assert_eq!(s, Some(&"from_registry".to_string()));
}

#[test]
fn singleton_registry_contains_singleton_once_lock_fallback() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "test".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    // Do NOT register manually; resolve via container (populates OnceLock)
    let _ = container.resolve::<String>().unwrap();
    // contains_singleton should find it via OnceLock check
    assert!(SingletonBeanRegistry::contains_singleton(
        &container,
        "alloc::string::String"
    ));
}

// ---------------------------------------------------------------------------
// 1e. get_registered_scope (always returns None)
// ---------------------------------------------------------------------------

#[test]
fn configurable_bean_factory_get_registered_scope_returns_none() {
    let mut container = {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| {
            "test".to_string()
        }))
        .unwrap();
        Container::new(b.build().unwrap())
    };
    // Register a scope first
    let scope = vernal_beans::request_scope::RequestScope::new("test_request");
    ConfigurableBeanFactory::register_scope(&mut container, "request", Box::new(scope));
    // get_registered_scope always returns None
    let result = ConfigurableBeanFactory::get_registered_scope(&container, "request");
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// 1f. bean_definition_count with mixed dynamic + registry definitions
// ---------------------------------------------------------------------------

#[test]
fn bean_definition_count_mixed() {
    let mut container = {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| {
            "registry".to_string()
        }))
        .unwrap();
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
            .unwrap();
        Container::new(b.build().unwrap())
    };
    let initial = BeanDefinitionRegistry::bean_definition_count(&container);
    assert_eq!(initial, 2);

    // Add a dynamic definition
    let mut rbd = vernal_beans::RootBeanDefinition::new();
    rbd.set_bean_class_name("dynamic::Type");
    BeanDefinitionRegistry::register_bean_definition(
        &mut container,
        "dynamic_bean".to_string(),
        Box::new(rbd),
    )
    .unwrap();

    let after = BeanDefinitionRegistry::bean_definition_count(&container);
    assert_eq!(after, initial + 1);
}

// ---------------------------------------------------------------------------
// 1g. bean_names_for_type_id (with or without dynamic definitions)
// ---------------------------------------------------------------------------

#[test]
fn bean_names_for_type_id_with_registry() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    let container = Container::new(b.build().unwrap());

    let names =
        ListableBeanFactory::bean_names_for_type_id(&container, TypeId::of::<String>(), true, true);
    assert_eq!(names.len(), 1);
    assert!(names[0].contains("String"));
}

// ---------------------------------------------------------------------------
// 1h. is_autowire_candidate for unknown bean names
// ---------------------------------------------------------------------------

#[test]
fn is_autowire_candidate_unknown_name() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    // An unknown name that doesn't exist in registry or dynamic_definitions
    let result =
        ConfigurableListableBeanFactory::is_autowire_candidate(&container, "unknown_type_xyz");
    assert!(!result);
}

// ---------------------------------------------------------------------------
// 1i. pre_instantiate_singletons with transient-only definitions
// ---------------------------------------------------------------------------

#[test]
fn pre_instantiate_singletons_transient_only() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::transient::<i32, _>(|_| 42i32))
        .unwrap();
    b.register(ComponentDefinition::transient::<String, _>(|_| {
        "transient".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let result = ConfigurableListableBeanFactory::pre_instantiate_singletons(&container);
    // Should succeed even with transient-only definitions
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// 1j. add_singleton_callback
// ---------------------------------------------------------------------------

#[test]
fn add_singleton_callback_test() {
    let mut container = {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| {
            "test".to_string()
        }))
        .unwrap();
        Container::new(b.build().unwrap())
    };
    // The closure takes &dyn Any, doesn't mutate captures
    SingletonBeanRegistry::add_singleton_callback(
        &mut container,
        "test_bean".to_string(),
        Arc::new(|_bean: &dyn Any| {
            // callback executed later
        }),
    );
    // The callback is registered; verify no crash
    let _ = SingletonBeanRegistry::singleton_mutex(&container);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. abstract_autowire_capable_bean_factory.rs
// ═══════════════════════════════════════════════════════════════════════════════

// A simple bean definition for use with AbstractAutowireCapableBeanFactory
#[derive(Debug)]
struct TestBeanDef {
    class_name: String,
    scope: Scope,
}

impl TestBeanDef {
    fn new(class_name: &str) -> Self {
        Self {
            class_name: class_name.to_string(),
            scope: Scope::Singleton,
        }
    }
}

impl BeanDefinition for TestBeanDef {
    fn bean_name(&self) -> &ComponentKey {
        static FALLBACK: std::sync::LazyLock<ComponentKey> =
            std::sync::LazyLock::new(|| ComponentKey::of::<String>());
        &FALLBACK
    }

    fn bean_class_name(&self) -> &str {
        &self.class_name
    }

    fn scope(&self) -> Scope {
        self.scope
    }

    fn is_lazy_init(&self) -> bool {
        false
    }

    fn is_primary(&self) -> bool {
        false
    }
}

struct MyService;

#[test]
fn abstract_autowire_capable_bean_factory_default_impl() {
    // Test Default trait
    let factory: vernal_beans::AbstractAutowireCapableBeanFactory = Default::default();
    assert_eq!(factory.bean_post_processor_count(), 0);
}

#[test]
fn abstract_autowire_capable_bean_factory_debug() {
    let factory = vernal_beans::AbstractAutowireCapableBeanFactory::new();
    let debug_str = format!("{:?}", factory);
    assert!(debug_str.contains("AbstractAutowireCapableBeanFactory"));
    assert!(debug_str.contains("instantiation_strategy"));
}

#[test]
fn abstract_autowire_capable_create_bean_with_constructor() {
    let factory = vernal_beans::AbstractAutowireCapableBeanFactory::new();
    // Register a constructor
    factory
        .instantiation_strategy()
        .register_constructor::<MyService>(|_args| {
            Ok(Arc::new(MyService) as Arc<dyn Any + Send + Sync>)
        });
    // Note: SimpleInstantiationStrategy.instantiate cannot map from
    // string class_name back to TypeId, so create_bean always returns
    // an error when the definition provides no factory method.
    // We verify the constructor was registered successfully.
    assert_eq!(factory.instantiation_strategy().constructor_count(), 1);
    let bd = TestBeanDef::new("MyService");
    let result = factory.create_bean("myService", &bd);
    // The strategy cannot find the constructor by string class name,
    // so this returns an error about missing constructor
    assert!(result.is_err());
}

#[test]
fn abstract_autowire_capable_create_bean_no_constructor() {
    let factory = vernal_beans::AbstractAutowireCapableBeanFactory::new();
    let bd = TestBeanDef::new("NoConstructorType");
    let result = factory.create_bean("noConstructor", &bd);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("constructor") || err_msg.contains("no constructors"));
}

#[test]
fn abstract_autowire_capable_initialize_bean_with_processors() {
    let factory = vernal_beans::AbstractAutowireCapableBeanFactory::new();

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
        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(bean))
        }
    }

    factory.add_bean_post_processor(Arc::new(InitBpp));
    let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    let result = factory.initialize_bean(bean, "testBean");
    assert!(result.is_ok());
}

#[test]
fn abstract_autowire_capable_initialize_bean_with_replacement() {
    let factory = vernal_beans::AbstractAutowireCapableBeanFactory::new();

    struct ReplacingBpp;
    impl BeanPostProcessor for ReplacingBpp {
        fn post_process_before_initialization(
            &self,
            _bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(
                Arc::new("replaced".to_string()) as Arc<dyn Any + Send + Sync>
            ))
        }
        fn post_process_after_initialization(
            &self,
            _bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(
                Arc::new("replaced2".to_string()) as Arc<dyn Any + Send + Sync>
            ))
        }
    }

    factory.add_bean_post_processor(Arc::new(ReplacingBpp));
    let bean = Arc::new("original".to_string()) as Arc<dyn Any + Send + Sync>;
    let result = factory.initialize_bean(bean, "testBean");
    assert!(result.is_ok());
    let val = result.unwrap();
    let s = val.downcast_ref::<String>();
    assert_eq!(s, Some(&"replaced2".to_string()));
}

#[test]
fn abstract_autowire_capable_apply_before_init_multiple_processors() {
    let factory = vernal_beans::AbstractAutowireCapableBeanFactory::new();

    struct BppA;
    impl BeanPostProcessor for BppA {
        fn post_process_before_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(bean))
        }
    }

    struct BppB;
    impl BeanPostProcessor for BppB {
        fn post_process_before_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(bean))
        }
    }

    factory.add_bean_post_processor(Arc::new(BppA));
    factory.add_bean_post_processor(Arc::new(BppB));
    let bean = Arc::new(99i32) as Arc<dyn Any + Send + Sync>;
    let result = factory.apply_bean_post_processors_before_initialization(bean, "multi");
    assert!(result.is_ok());
}

#[test]
fn abstract_autowire_capable_apply_after_init_with_processors() {
    let factory = vernal_beans::AbstractAutowireCapableBeanFactory::new();

    struct AfterBpp;
    impl BeanPostProcessor for AfterBpp {
        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(bean))
        }
    }

    factory.add_bean_post_processor(Arc::new(AfterBpp));
    let bean = Arc::new("after_test".to_string()) as Arc<dyn Any + Send + Sync>;
    let result = factory.apply_bean_post_processors_after_initialization(bean, "after");
    assert!(result.is_ok());
}

#[test]
fn abstract_autowire_capable_add_and_get_processors() {
    let factory = vernal_beans::AbstractAutowireCapableBeanFactory::new();
    assert_eq!(factory.bean_post_processor_count(), 0);

    struct DummyBpp;
    impl BeanPostProcessor for DummyBpp {}

    factory.add_bean_post_processor(Arc::new(DummyBpp));
    assert_eq!(factory.bean_post_processor_count(), 1);
    assert_eq!(factory.get_bean_post_processors().len(), 1);
}

#[test]
fn abstract_autowire_capable_bean_post_processor_count() {
    let factory = vernal_beans::AbstractAutowireCapableBeanFactory::new();
    assert_eq!(factory.bean_post_processor_count(), 0);

    struct CountedBpp;
    impl BeanPostProcessor for CountedBpp {}

    factory.add_bean_post_processor(Arc::new(CountedBpp));
    factory.add_bean_post_processor(Arc::new(CountedBpp));
    assert_eq!(factory.bean_post_processor_count(), 2);
}

#[test]
fn abstract_autowire_capable_instantiation_strategy_access() {
    let mut factory = vernal_beans::AbstractAutowireCapableBeanFactory::new();
    {
        let strategy = factory.instantiation_strategy();
        assert_eq!(strategy.constructor_count(), 0);
    }
    {
        let strategy_mut = factory.instantiation_strategy_mut();
        strategy_mut.register_constructor::<String>(|_args| {
            Ok(Arc::new("test".to_string()) as Arc<dyn Any + Send + Sync>)
        });
    }
    {
        let strategy = factory.instantiation_strategy();
        assert_eq!(strategy.constructor_count(), 1);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. registry_builder.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn registry_builder_register_all_with_definitions_and_check() {
    let mut b = RegistryBuilder::new();
    b.register_all([
        ComponentDefinition::singleton::<String, _>(|_| "a".to_string()),
        ComponentDefinition::singleton::<i32, _>(|_| 42i32),
    ])
    .unwrap();
    let registry = b.build().unwrap();
    assert_eq!(registry.definitions().len(), 2);
}

#[test]
fn registry_builder_bind_and_bind_all_combined() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.register(ComponentDefinition::singleton::<OtherImpl, _>(|_| {
        OtherImpl
    }))
    .unwrap();

    // Use bind() then bind_all()
    b.bind(TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();

    b.bind_all(vec![TraitBinding::new::<dyn MyTrait, OtherImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    })])
    .unwrap();

    let registry = b.build().unwrap();
    assert_eq!(registry.bindings().len(), 2);
}

#[test]
fn registry_builder_register_bundle_with_both() {
    let mut b = RegistryBuilder::new();
    b.register_bundle(
        vec![ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl)],
        vec![TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| {
            s as Arc<dyn MyTrait>
        })],
    )
    .unwrap();
    let registry = b.build().unwrap();
    assert_eq!(registry.definitions().len(), 1);
    assert_eq!(registry.bindings().len(), 1);
}

#[test]
fn registry_builder_duplicate_register_error() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "first".to_string()
    }))
    .unwrap();
    let result = b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "second".to_string()
    }));
    assert!(result.is_err());
}

#[test]
fn registry_builder_remove_nonexistent_by_key() {
    let mut b = RegistryBuilder::new();
    let key = ComponentKey::of::<String>();
    let result = b.remove_by_key(&key);
    assert!(result.is_err());
}

#[test]
fn registry_builder_contains_after_register_remove() {
    let mut b = RegistryBuilder::new();
    assert!(!b.contains::<String>());
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "test".to_string()
    }))
    .unwrap();
    assert!(b.contains::<String>());

    let key = ComponentKey::of::<String>();
    let _ = b.remove_by_key(&key);
    assert!(!b.contains::<String>());
}

#[test]
fn registry_builder_is_empty_and_len_full() {
    let mut b = RegistryBuilder::new();
    assert!(b.is_empty());
    assert_eq!(b.len(), 0);

    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "test".to_string()
    }))
    .unwrap();
    assert!(!b.is_empty());
    assert_eq!(b.len(), 1);

    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    assert_eq!(b.len(), 2);
}

#[test]
fn registry_builder_debug() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "test".to_string()
    }))
    .unwrap();
    let debug = format!("{:?}", b);
    assert!(debug.contains("RegistryBuilder"));
    assert!(debug.contains("definitions"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. scope_context.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn scope_context_open_state_and_close() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "test".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());

    let scope = container.open_scope::<RequestScopeTag>();
    assert_eq!(scope.state(), ScopeState::Open);

    let result = scope.close().await;
    assert!(result.is_ok());
    assert_eq!(scope.state(), ScopeState::Closed);
}

#[tokio::test]
async fn scope_context_child_from_parent() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "test".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());

    let parent = container.open_scope::<RequestScopeTag>();
    let child = parent.child::<SubScopeTag>();
    assert_eq!(child.state(), ScopeState::Open);
    assert_eq!(child.key(), ScopeKey::of::<SubScopeTag>());
    assert!(child.parent().is_some());

    // Close parent (should also notify child via cancellation)
    let _ = parent.close().await;
}

#[tokio::test]
async fn scope_context_open_with_cancellation() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "test".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());

    let token = tokio_util::sync::CancellationToken::new();
    let scope = container.open_scope_with_cancellation::<RequestScopeTag>(token);
    assert_eq!(scope.state(), ScopeState::Open);

    let result = scope.close().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn scope_context_get_or_insert_with_value() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "test".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());

    let scope = container.open_scope::<RequestScopeTag>();
    let val = scope
        .get_or_insert_with::<String, _>(|| "scope_value".to_string())
        .unwrap();
    assert_eq!(*val, "scope_value".to_string());
    scope.close().await.unwrap();
}

#[tokio::test]
async fn scope_context_multiple_independent_scopes() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "test".to_string()
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());

    let s1 = container.open_scope::<RequestScopeTag>();
    let s2 = container.open_scope::<SubScopeTag>();
    assert_eq!(s1.state(), ScopeState::Open);
    assert_eq!(s2.state(), ScopeState::Open);
    assert!(s1.parent().is_none());
    assert!(s2.parent().is_none());

    // Close both
    s1.close().await.unwrap();
    s2.close().await.unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. resolver.rs — tested inside ComponentDefinition factory closures
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn resolver_resolve_all_traits_within_factory() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<i32, _>(move |resolver: &Resolver| {
            // resolve_all_traits
            let all = resolver.resolve_all_traits::<dyn MyTrait>().unwrap();
            assert_eq!(all.len(), 2);

            // provider
            let _prov = resolver.provider::<String>().unwrap();

            42i32
        })
        .depends_on::<String>()
        .depends_on_all_traits::<dyn MyTrait>()
        .depends_on_provider::<String>(),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.register(ComponentDefinition::singleton::<OtherImpl, _>(|_| {
        OtherImpl
    }))
    .unwrap();
    // Two bindings (all_traits can handle multiple)
    b.bind(TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, OtherImpl, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let result = container.resolve::<i32>();
    assert!(result.is_ok());
}

#[test]
fn resolver_resolve_trait_with_primary_within_factory() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<i32, _>(move |resolver: &Resolver| {
            // Single trait resolution with a primary binding
            let _t = resolver.resolve_trait::<dyn MyTrait>().unwrap();
            42i32
        })
        .depends_on_trait::<dyn MyTrait>(),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    // One binding with primary
    b.bind(TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| s as Arc<dyn MyTrait>).primary())
        .unwrap();
    let container = Container::new(b.build().unwrap());
    let result = container.resolve::<i32>();
    assert!(result.is_ok());
}

#[test]
fn resolver_resolve_optional_trait_present_within_factory() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<i32, _>(move |resolver: &Resolver| {
            let opt = resolver.resolve_optional_trait::<dyn MyTrait>().unwrap();
            assert!(opt.is_some());
            42i32
        })
        .depends_on_optional_trait::<dyn MyTrait>(),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| s as Arc<dyn MyTrait>).primary())
        .unwrap();
    let container = Container::new(b.build().unwrap());
    let result = container.resolve::<i32>();
    assert!(result.is_ok());
}

#[test]
fn resolver_resolve_optional_trait_absent_within_factory() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<i32, _>(move |resolver: &Resolver| {
            let opt = resolver.resolve_optional_trait::<dyn MyTrait>().unwrap();
            assert!(opt.is_none());
            42i32
        })
        .depends_on_optional_trait::<dyn MyTrait>(),
    )
    .unwrap();
    // Register MyImpl but NO binding for dyn MyTrait
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    let container = Container::new(b.build().unwrap());
    let result = container.resolve::<i32>();
    assert!(result.is_ok());
}

#[test]
fn resolver_resolve_qualified_trait_within_factory() {
    let q = Qualifier::new("myq").unwrap();
    let q_for_factory = q.clone();
    let q_for_dep = q.clone();
    let q_for_binding = q.clone();
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<i32, _>(move |resolver: &Resolver| {
            let _qt = resolver
                .resolve_qualified_trait::<dyn MyTrait>(&q_for_factory)
                .unwrap();
            42i32
        })
        .depends_on_qualified_trait::<dyn MyTrait>(q_for_dep),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.bind(
        TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| s as Arc<dyn MyTrait>)
            .qualified(q_for_binding),
    )
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let result = container.resolve::<i32>();
    assert!(result.is_ok());
}

#[test]
fn resolver_provider_and_trait_provider_within_factory() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<String, _>(move |resolver: &Resolver| {
            // provider::<T>()
            let _prov = resolver.provider::<i32>().unwrap();
            // trait_provider::<T>()
            let _tp = resolver.trait_provider::<dyn MyTrait>().unwrap();
            "ok".to_string()
        })
        .depends_on_provider::<i32>()
        .depends_on_trait_provider::<dyn MyTrait>(),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    b.register(ComponentDefinition::singleton::<MyImpl, _>(|_| MyImpl))
        .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, MyImpl, _>(|s| s as Arc<dyn MyTrait>).primary())
        .unwrap();
    let container = Container::new(b.build().unwrap());
    let result = container.resolve::<String>();
    assert!(result.is_ok());
}
