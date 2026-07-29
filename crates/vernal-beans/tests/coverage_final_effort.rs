//! Coverage test for high-priority uncovered code paths.
//!
//! Targets:
//! - `container.rs` — circular dep detection, is_type_match, ContainerObjectProvider,
//!   bean_names_for_type_id flags, get_registered_scope, singleton_count/names,
//!   register_resolvable_dependency, ignore_dependency_type/interface, autowire modes,
//!   autowire_bean_properties modes, is_factory_bean
//! - `abstract_autowire_capable_bean_factory.rs` — create_bean, initialize_bean,
//!   BeanPostProcessor chains, instantiation_strategy_mut, Default, Debug
//! - `registry_builder.rs` — register_all, bind, bind_all, register_bundle,
//!   remove_by_key, is_empty, len, contains
//! - `scope_context.rs` — lifecycle, child scope, get_or_insert_with

use std::any::{Any, TypeId};
use std::sync::Arc;
use vernal_beans::*;

// ─────────────────────────────────────────────────────────────────────────────
// Helper types
// ─────────────────────────────────────────────────────────────────────────────

/// A minimal struct for BeanDefinition and constructor tests.
#[derive(Debug)]
struct MinimalBeanDef;
impl bean_definition::BeanDefinition for MinimalBeanDef {
    fn bean_name(&self) -> &ComponentKey {
        use std::sync::LazyLock;
        static KEY: LazyLock<ComponentKey> = LazyLock::new(|| ComponentKey::of::<MinimalBeanDef>());
        &KEY
    }
    fn bean_class_name(&self) -> &str {
        "MinimalBeanDef"
    }
    fn scope(&self) -> Scope {
        Scope::Singleton
    }
    fn is_lazy_init(&self) -> bool {
        false
    }
    fn is_primary(&self) -> bool {
        false
    }
}

struct TestReplaceProcessor;
impl bean_post_processor::BeanPostProcessor for TestReplaceProcessor {
    fn post_process_before_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(Arc::new(777i32) as Arc<dyn Any + Send + Sync>))
    }

    fn post_process_after_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(Arc::new(888i32) as Arc<dyn Any + Send + Sync>))
    }
}

/// Minimal BeanScope implementation for testing get_registered_scope.
struct TestScope;
impl bean_scope::BeanScope for TestScope {
    fn get(
        &self,
        _name: &str,
        _object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Box::new("test"))
    }

    fn remove(
        &self,
        _name: &str,
    ) -> Result<Option<Box<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    fn register_destruction_callback(
        &self,
        _name: &str,
        _callback: Box<dyn FnOnce() + Send + Sync>,
    ) {
    }

    fn resolve_contextual_object(&self, _key: &str) -> Option<Box<dyn Any>> {
        None
    }

    fn conversation_id(&self) -> Option<&str> {
        None
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. container.rs
// ─────────────────────────────────────────────────────────────────────────────

/// (a) Circular dependency detection — graph planner catches a self-referencing
///     definition at build time.
#[test]
fn test_graph_planner_detects_self_cycle() {
    // A Transient definition with a declared self-dependency.
    // The graph planner's DFS visits the definition and during dependency traversal
    // encounters the same node in STATE_VISITING, returning GraphError::Cycle.
    let def = ComponentDefinition::transient::<String, _>(|_: &Resolver| "x".to_string())
        .depends_on::<String>();

    let mut builder = RegistryBuilder::new();
    builder.register(def).unwrap();
    let result = builder.build();
    assert!(result.is_err(), "expected build to reject self-cycle");

    match result {
        Err(GraphError::Cycle { path }) => {
            assert!(!path.is_empty(), "cycle path should not be empty");
            assert!(
                path.iter().any(|s| s.contains("String")),
                "cycle path should mention String, got: {:?}",
                path
            );
        }
        Err(other) => panic!("expected GraphError::Cycle, got: {:?}", other),
        Ok(_) => panic!("expected error, got Ok"),
    }
}

/// (b) `is_type_match` with matching and non-matching types.
#[test]
fn test_is_type_match() {
    use bean_factory::BeanFactory;

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = registry.container();

    // Matching type
    let key = ComponentKey::of::<i32>();
    assert!(container.is_type_match(&key, TypeId::of::<i32>()));

    // Non-matching type
    assert!(!container.is_type_match(&key, TypeId::of::<String>()));

    // Non-existent key
    let missing_key = ComponentKey::of::<String>();
    assert!(!container.is_type_match(&missing_key, TypeId::of::<String>()));
}

/// (c) ContainerObjectProvider::if_available — no cached singleton vs cached.
#[test]
fn test_container_object_provider_if_available() {
    use bean_factory::BeanFactory;

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = registry.container();

    // Create a provider — initially no cached singleton
    let provider = container
        .get_bean_provider_by_type_id(TypeId::of::<i32>())
        .unwrap();

    // if_available should be None because no singleton has been resolved yet
    // (the singleton cell exists but the OnceLock is uninitialized)
    assert!(provider.if_available().is_none());

    // Resolve the bean to populate the singleton cache
    let _resolved = container.resolve::<i32>().unwrap();

    // Now if_available should return the cached value
    let available = provider.if_available();
    assert!(
        available.is_some(),
        "expected Some after singleton resolution"
    );
    let val = available.unwrap();
    assert_eq!(*val.downcast_ref::<i32>().unwrap(), 42);
}

/// (d) bean_names_for_type_id with include_non_singletons / allow_eager_init flags.
#[test]
fn test_bean_names_for_type_id_with_flags() {
    use listable_bean_factory::ListableBeanFactory;

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    builder
        .register(ComponentDefinition::shared_value("hello".to_string()))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = registry.container();

    // include_non_singletons = true, allow_eager_init = true
    let names_true = container.bean_names_for_type_id(TypeId::of::<i32>(), true, true);
    assert_eq!(names_true.len(), 1, "should find one i32 definition");

    // include_non_singletons = false, allow_eager_init = false
    let names_false = container.bean_names_for_type_id(TypeId::of::<i32>(), false, false);
    // The current implementation ignores both flags, so the result is the same
    assert_eq!(names_false.len(), 1);

    // Non-existing type
    let names_missing = container.bean_names_for_type_id(TypeId::of::<f64>(), true, true);
    assert!(names_missing.is_empty());
}

/// (e) get_registered_scope — always returns None (by design).
#[test]
fn test_get_registered_scope() {
    use configurable_bean_factory::ConfigurableBeanFactory;

    let registry = Registry::empty();
    let mut container = registry.container();

    // Before registering any scope — both existing and non-existing names return None
    assert!(container.get_registered_scope("request").is_none());
    assert!(container.get_registered_scope("nonexistent").is_none());

    // Register a scope and try again
    container.register_scope("test_scope", Box::new(TestScope));
    assert!(
        container.get_registered_scope("test_scope").is_none(),
        "get_registered_scope always returns None due to Mutex borrowing constraint"
    );
    assert!(container.get_registered_scope("request").is_none());

    // registered_scope_names should contain the registered name
    let names = container.registered_scope_names();
    assert!(names.contains(&"test_scope".to_string()));
}

/// (f) singleton_count and singleton_names with both registered and OnceLock singletons.
#[test]
fn test_singleton_count_and_names() {
    use singleton_bean_registry::SingletonBeanRegistry;

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    builder
        .register(ComponentDefinition::shared_value("hello".to_string()))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = registry.container();

    // Initially: no registered singletons and no resolved OnceLock singletons
    assert_eq!(container.singleton_count(), 0);
    assert!(container.singleton_names().is_empty());

    // Register singletons directly (via SingletonBeanRegistry)
    container.register_singleton(
        "my_registered",
        Arc::new(100i64) as Arc<dyn Any + Send + Sync>,
    );
    container.register_singleton("my_other", Arc::new(200i64) as Arc<dyn Any + Send + Sync>);

    // Resolve beans to populate the OnceLock cache
    let _ = container.resolve::<i32>().unwrap();
    let _ = container.resolve::<String>().unwrap();

    // Now: 2 registered + 2 OnceLock = 4
    assert_eq!(
        container.singleton_count(),
        4,
        "expected 4 (2 registered + 2 OnceLock), got {}",
        container.singleton_count()
    );

    let names = container.singleton_names();
    assert!(names.contains(&"my_registered".to_string()));
    assert!(names.contains(&"my_other".to_string()));
    // The OnceLock names come from ComponentKey::type_name()
    assert!(
        names.iter().any(|n| n.contains("i32")),
        "i32 type name should be present: {:?}",
        names
    );

    // contains_singleton checks both stores
    assert!(container.contains_singleton("my_registered"));
    assert!(container.contains_singleton("my_other"));
}

/// (g) ConfigurableListableBeanFactory methods.
#[test]
fn test_configurable_listable_bean_factory_methods() {
    use configurable_listable_bean_factory::ConfigurableListableBeanFactory;

    let registry = Registry::empty();
    let mut container = registry.container();

    // ignore_dependency_type
    container.ignore_dependency_type(TypeId::of::<i32>());
    // ignore_dependency_interface (same storage)
    container.ignore_dependency_interface(TypeId::of::<String>());

    // register_resolvable_dependency
    container.register_resolvable_dependency(
        TypeId::of::<i64>(),
        Arc::new(42i64) as Arc<dyn Any + Send + Sync>,
    );

    // freeze_configuration / is_configuration_frozen
    assert!(!container.is_configuration_frozen());
    container.freeze_configuration();
    assert!(container.is_configuration_frozen());

    // is_autowire_candidate for a non-existent bean
    assert!(!container.is_autowire_candidate("nonexistent_bean"));
}

/// (h) autowire with valid modes (0-3) and invalid mode 99.
#[test]
fn test_autowire_modes() {
    use autowire_capable_bean_factory::AutowireCapableBeanFactory;

    struct AwBeanA;
    struct AwBeanB;

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::transient::<AwBeanA, _>(|_| AwBeanA))
        .unwrap();
    builder
        .register(ComponentDefinition::transient::<AwBeanB, _>(|_| AwBeanB))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = registry.container();

    let name_a = ComponentKey::of::<AwBeanA>().type_name();
    let name_b = ComponentKey::of::<AwBeanB>().type_name();

    // Mode 0 (AUTOWIRE_NO): create bean only
    let r0 = container.autowire(name_a, 0, false);
    assert!(r0.is_ok(), "autowire mode 0 should succeed: {:?}", r0.err());

    // Mode 1 (AUTOWIRE_BY_NAME): create + autowire properties
    let r1 = container.autowire(name_a, 1, false);
    assert!(r1.is_ok(), "autowire mode 1 should succeed: {:?}", r1.err());

    // Mode 2 (AUTOWIRE_BY_TYPE): create + autowire properties
    let r2 = container.autowire(name_a, 2, false);
    assert!(r2.is_ok(), "autowire mode 2 should succeed: {:?}", r2.err());

    // Mode 3 (AUTOWIRE_CONSTRUCTOR): create bean only
    let r3 = container.autowire(name_b, 3, false);
    assert!(r3.is_ok(), "autowire mode 3 should succeed: {:?}", r3.err());

    // Invalid mode 99
    let r_invalid = container.autowire("anything", 99, false);
    assert!(r_invalid.is_err(), "autowire mode 99 should fail");
    let err_msg = r_invalid.unwrap_err().to_string();
    assert!(
        err_msg.contains("Invalid autowire mode"),
        "error should mention invalid mode, got: {}",
        err_msg
    );

    // Non-existent bean_class_name with mode 0
    let r_missing = container.autowire("nonexistent.Type", 0, false);
    assert!(r_missing.is_err());
}

/// (i) autowire_bean_properties with all modes: 0, 1, 2, 999.
#[test]
fn test_autowire_bean_properties_modes() {
    use autowire_capable_bean_factory::AutowireCapableBeanFactory;

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = registry.container();

    let bean = Arc::new(999i32) as Arc<dyn Any + Send + Sync>;

    // Mode 0 (AUTOWIRE_NO): no-op, returns bean as-is
    let r0 = container.autowire_bean_properties(bean.clone(), 0, false);
    assert!(r0.is_ok());
    assert!(Arc::ptr_eq(&bean, &r0.unwrap()));

    // Mode 1 (AUTOWIRE_BY_NAME): loops over definitions, returns bean as-is
    let r1 = container.autowire_bean_properties(bean.clone(), 1, false);
    assert!(r1.is_ok());

    // Mode 2 (AUTOWIRE_BY_TYPE): same as mode 1
    let r2 = container.autowire_bean_properties(bean.clone(), 2, false);
    assert!(r2.is_ok());

    // Mode 999 (fallback branch): returns bean as-is
    let r999 = container.autowire_bean_properties(bean.clone(), 999, false);
    assert!(r999.is_ok());
    assert!(Arc::ptr_eq(&bean, &r999.unwrap()));
}

/// (j) is_factory_bean with & prefix, without &, and empty string.
#[test]
fn test_is_factory_bean() {
    use configurable_bean_factory::ConfigurableBeanFactory;

    let registry = Registry::empty();
    let container = registry.container();

    // With & prefix — is a factory bean reference
    assert!(container.is_factory_bean("&myBean"));

    // Without & prefix — not a factory bean
    assert!(!container.is_factory_bean("myBean"));

    // Empty string
    assert!(!container.is_factory_bean(""));

    // Just the prefix character
    assert!(container.is_factory_bean("&"));

    // With multiple & characters
    assert!(container.is_factory_bean("&&myBean"));
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. abstract_autowire_capable_bean_factory.rs
// ─────────────────────────────────────────────────────────────────────────────

/// create_bean with a registered constructor — still fails because
/// SimpleInstantiationStrategy cannot match a class_name string to a TypeId,
/// so it falls through to the "cannot instantiate" error.
#[test]
fn test_abstract_create_bean_with_registered_constructor() {
    use abstract_autowire_capable_bean_factory::AbstractAutowireCapableBeanFactory;

    let factory = AbstractAutowireCapableBeanFactory::new();
    factory
        .instantiation_strategy()
        .register_constructor::<MinimalBeanDef>(|_args| {
            Ok(Arc::new(MinimalBeanDef) as Arc<dyn Any + Send + Sync>)
        });

    let bd = MinimalBeanDef;
    let result = factory.create_bean("testBean", &bd);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("cannot instantiate") || err.contains("no constructors registered"),
        "unexpected error: {}",
        err
    );
}

/// create_bean with NO matching constructor: error path.
#[test]
fn test_abstract_create_bean_no_constructor() {
    use abstract_autowire_capable_bean_factory::AbstractAutowireCapableBeanFactory;

    let factory = AbstractAutowireCapableBeanFactory::new();
    let bd = MinimalBeanDef;
    let result = factory.create_bean("testBean", &bd);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("no constructors registered"),
        "expected 'no constructors registered', got: {}",
        err
    );
}

/// initialize_bean round-trips the bean through the post-processor chain.
#[test]
fn test_abstract_initialize_bean() {
    use abstract_autowire_capable_bean_factory::AbstractAutowireCapableBeanFactory;

    let factory = AbstractAutowireCapableBeanFactory::new();
    let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    let result = factory.initialize_bean(bean.clone(), "testBean");
    assert!(result.is_ok());
    let returned = result.unwrap();
    // Without any processors, the same bean should be returned
    assert!(Arc::ptr_eq(&bean, &returned));
}

/// apply_bean_post_processors_before_initialization with a processor returning Some(replacement).
#[test]
fn test_abstract_processor_before_with_replacement() {
    use abstract_autowire_capable_bean_factory::AbstractAutowireCapableBeanFactory;

    let factory = AbstractAutowireCapableBeanFactory::new();
    factory.add_bean_post_processor(Arc::new(TestReplaceProcessor));

    let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    let result = factory.apply_bean_post_processors_before_initialization(bean, "testBean");
    assert!(result.is_ok());
    let output = result.unwrap();
    let val = *output.downcast_ref::<i32>().unwrap();
    assert_eq!(val, 777, "processor should replace bean with 777");
}

/// apply_bean_post_processors_after_initialization with a processor.
#[test]
fn test_abstract_processor_after_with_replacement() {
    use abstract_autowire_capable_bean_factory::AbstractAutowireCapableBeanFactory;

    let factory = AbstractAutowireCapableBeanFactory::new();
    factory.add_bean_post_processor(Arc::new(TestReplaceProcessor));

    let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    let result = factory.apply_bean_post_processors_after_initialization(bean, "testBean");
    assert!(result.is_ok());
    let output = result.unwrap();
    let val = *output.downcast_ref::<i32>().unwrap();
    assert_eq!(val, 888, "processor should replace bean with 888");
}

/// Multiple post-processors in before-initialization chain.
#[test]
fn test_abstract_multiple_processors_before() {
    use abstract_autowire_capable_bean_factory::AbstractAutowireCapableBeanFactory;

    struct Doubler;
    impl bean_post_processor::BeanPostProcessor for Doubler {
        fn post_process_before_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            if let Some(v) = bean.downcast_ref::<i32>() {
                Ok(Some(Arc::new(v * 2) as Arc<dyn Any + Send + Sync>))
            } else {
                Ok(Some(bean))
            }
        }
    }

    struct Tripler;
    impl bean_post_processor::BeanPostProcessor for Tripler {
        fn post_process_before_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            if let Some(v) = bean.downcast_ref::<i32>() {
                Ok(Some(Arc::new(v * 3) as Arc<dyn Any + Send + Sync>))
            } else {
                Ok(Some(bean))
            }
        }
    }

    let factory = AbstractAutowireCapableBeanFactory::new();
    factory.add_bean_post_processor(Arc::new(Doubler));
    factory.add_bean_post_processor(Arc::new(Tripler));

    let bean = Arc::new(5i32) as Arc<dyn Any + Send + Sync>;
    let result = factory.apply_bean_post_processors_before_initialization(bean, "test");
    assert!(result.is_ok());
    let output = result.unwrap();
    let val = *output.downcast_ref::<i32>().unwrap();
    // Doubler: 5 -> 10, then Tripler: 10 -> 30
    assert_eq!(val, 30, "multiple processors should compose: 5*2*3 = 30");
}

/// instantiation_strategy_mut returns a mutable reference.
#[test]
fn test_abstract_instantiation_strategy_mut() {
    use abstract_autowire_capable_bean_factory::AbstractAutowireCapableBeanFactory;

    let mut factory = AbstractAutowireCapableBeanFactory::new();
    {
        let strategy = factory.instantiation_strategy_mut();
        strategy
            .register_constructor::<i32>(|_args| Ok(Arc::new(42) as Arc<dyn Any + Send + Sync>));
        assert_eq!(strategy.constructor_count(), 1);
    }
    // Verify persistence
    assert_eq!(factory.instantiation_strategy().constructor_count(), 1);
}

/// Default and Debug implementations.
#[test]
fn test_abstract_default_and_debug() {
    use abstract_autowire_capable_bean_factory::AbstractAutowireCapableBeanFactory;

    let factory = AbstractAutowireCapableBeanFactory::default();
    let debug_str = format!("{:?}", factory);
    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("AbstractAutowireCapableBeanFactory"));

    // Adding a processor changes Debug output
    factory.add_bean_post_processor(Arc::new(TestReplaceProcessor));
    let debug_str2 = format!("{:?}", factory);
    assert!(debug_str2.contains("bean_post_processor_count"));
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. registry_builder.rs
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_registry_builder_register_all() {
    struct RbA;
    struct RbB;

    let mut builder = RegistryBuilder::new();
    let defs = vec![
        ComponentDefinition::shared_value(RbA),
        ComponentDefinition::shared_value(RbB),
    ];
    builder.register_all(defs).unwrap();
    assert_eq!(builder.len(), 2);
}

#[test]
fn test_registry_builder_bind() {
    trait MyBindTrait: Send + Sync + 'static {}
    impl MyBindTrait for i32 {}

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(42i32))
        .unwrap();

    let binding =
        TraitBinding::new::<dyn MyBindTrait, i32, _>(|arc: Arc<i32>| arc as Arc<dyn MyBindTrait>);
    builder.bind(binding).unwrap();

    // Verify the binding was stored by checking build succeeds
    let registry = builder.build().unwrap();
    assert!(!registry.bindings().is_empty());
}

#[test]
fn test_registry_builder_bind_all() {
    trait MyBindTrait2: Send + Sync + 'static {}
    impl MyBindTrait2 for i32 {}
    impl MyBindTrait2 for String {}

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    builder
        .register(ComponentDefinition::shared_value("hello".to_string()))
        .unwrap();

    let bindings = vec![
        TraitBinding::new::<dyn MyBindTrait2, i32, _>(|arc: Arc<i32>| arc as Arc<dyn MyBindTrait2>),
        TraitBinding::new::<dyn MyBindTrait2, String, _>(|arc: Arc<String>| {
            arc as Arc<dyn MyBindTrait2>
        }),
    ];
    builder.bind_all(bindings).unwrap();

    let registry = builder.build().unwrap();
    assert_eq!(registry.bindings().len(), 2);
}

#[test]
fn test_registry_builder_register_bundle() {
    trait BundleTrait: Send + Sync + 'static {}
    impl BundleTrait for i32 {}

    let mut builder = RegistryBuilder::new();
    let definitions = vec![ComponentDefinition::shared_value(42i32)];
    let bindings = vec![TraitBinding::new::<dyn BundleTrait, i32, _>(
        |arc: Arc<i32>| arc as Arc<dyn BundleTrait>,
    )];

    builder.register_bundle(definitions, bindings).unwrap();
    assert_eq!(builder.len(), 1);

    let registry = builder.build().unwrap();
    assert_eq!(registry.bindings().len(), 1);
}

#[test]
fn test_registry_builder_remove_by_key() {
    struct RbRemove1;
    struct RbRemove2;

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(RbRemove1))
        .unwrap();
    builder
        .register(ComponentDefinition::shared_value(RbRemove2))
        .unwrap();
    assert_eq!(builder.len(), 2);

    // Remove by key — success
    let key1 = ComponentKey::of::<RbRemove1>();
    builder.remove_by_key(&key1).unwrap();
    assert_eq!(builder.len(), 1);
    assert!(!builder.contains::<RbRemove1>());
    assert!(builder.contains::<RbRemove2>());

    // Remove same key again — failure (already gone)
    let result = builder.remove_by_key(&key1);
    assert!(result.is_err());

    // Remove remaining key
    let key2 = ComponentKey::of::<RbRemove2>();
    builder.remove_by_key(&key2).unwrap();
    assert!(builder.is_empty());
}

#[test]
fn test_registry_builder_is_empty_and_len() {
    let builder = RegistryBuilder::new();
    assert!(builder.is_empty());
    assert_eq!(builder.len(), 0);

    let mut builder2 = RegistryBuilder::new();
    builder2
        .register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    assert!(!builder2.is_empty());
    assert_eq!(builder2.len(), 1);
}

#[test]
fn test_registry_builder_contains() {
    struct RbContA;
    struct RbContB;

    let mut builder = RegistryBuilder::new();

    // Before register — false
    assert!(!builder.contains::<RbContA>());
    assert!(!builder.contains::<RbContB>());

    // Register A
    builder
        .register(ComponentDefinition::shared_value(RbContA))
        .unwrap();
    assert!(builder.contains::<RbContA>());
    assert!(!builder.contains::<RbContB>());

    // After remove — false again
    let key_a = ComponentKey::of::<RbContA>();
    builder.remove_by_key(&key_a).unwrap();
    assert!(!builder.contains::<RbContA>());
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. scope_context.rs
// ─────────────────────────────────────────────────────────────────────────────

/// Open a scope, verify it's Open, close it, verify it becomes Closed.
#[tokio::test]
async fn test_scope_context_lifecycle() {
    let registry = Registry::empty();
    let container = registry.container();

    let scope = container.open_scope::<String>();
    assert_eq!(scope.state(), ScopeState::Open);

    // Close the scope
    scope.close().await.unwrap();
    assert_eq!(scope.state(), ScopeState::Closed);
}

/// Child scope creation and independent lifecycle.
#[tokio::test]
async fn test_scope_context_child() {
    let registry = Registry::empty();
    let container = registry.container();

    struct ParentScope;
    struct ChildScope;

    let parent = container.open_scope::<ParentScope>();
    let child = parent.child::<ChildScope>();

    assert_eq!(child.state(), ScopeState::Open);
    assert_eq!(parent.state(), ScopeState::Open);

    // Close the parent scope; child should still be closable
    parent.close().await.unwrap();
    assert_eq!(parent.state(), ScopeState::Closed);

    // Child can be closed independently
    child.close().await.unwrap();
    assert_eq!(child.state(), ScopeState::Closed);
}

/// get_or_insert_with on a scope — returns the factory result and caches it.
#[tokio::test]
async fn test_scope_context_get_or_insert_with() {
    let registry = Registry::empty();
    let container = registry.container();

    struct MyScope;
    let scope = container.open_scope::<MyScope>();

    // First insertion
    let value: Arc<i32> = scope.get_or_insert_with(|| 42i32).unwrap();
    assert_eq!(*value, 42);

    // Second insertion with a different factory — should return cached first value
    let value2: Arc<i32> = scope.get_or_insert_with(|| 100i32).unwrap();
    assert_eq!(
        *value2, 42,
        "should return cached value, not re-execute factory"
    );

    // get_or_insert_with with a String type
    let s: Arc<String> = scope.get_or_insert_with(|| "cached".to_string()).unwrap();
    assert_eq!(*s, "cached");

    // Verify close still works
    scope.close().await.unwrap();
    assert_eq!(scope.state(), ScopeState::Closed);

    // After close, get_or_insert_with should fail
    let after_close = scope.get_or_insert_with::<i32, _>(|| 99i32);
    assert!(
        after_close.is_err(),
        "get_or_insert_with after close should fail"
    );
}
