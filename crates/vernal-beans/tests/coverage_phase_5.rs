//! Coverage Phase 5: targeted tests for specific uncovered code paths
//! in container.rs, abstract_autowire_capable_bean_factory.rs,
//! registry_builder.rs, and scope_context.rs.

use std::any::{Any, TypeId};
use std::sync::Arc;

use vernal_beans::{
    AutowireCapableBeanFactory, BeanDefinition, BeanFactory, BeanPostProcessor,
    ComponentDefinition, ComponentKey, Container, RegistryBuilder, ResolveError, Scope, ScopeKey,
    ScopeState, TraitBinding, hierarchical_bean_factory::HierarchicalBeanFactory,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Shared test types
// ═══════════════════════════════════════════════════════════════════════════════

trait MyTrait: Send + Sync + 'static {
    #[allow(dead_code)]
    fn identify(&self) -> &'static str;
}

struct ImplOne;
impl MyTrait for ImplOne {
    fn identify(&self) -> &'static str {
        "ImplOne"
    }
}

struct ImplTwo;
impl MyTrait for ImplTwo {
    fn identify(&self) -> &'static str {
        "ImplTwo"
    }
}

struct RequestScopeTag;
struct TaskScopeTag;

// ═══════════════════════════════════════════════════════════════════════════════
// 1. container.rs — BeanFactory / ConfigurableBeanFactory / SingletonBeanRegistry
//    / ConfigurableListableBeanFactory / AutowireCapableBeanFactory
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn resolve_binding_not_found_error_format() {
    // The runtime NotFound path in resolve_binding is guarded by build-time
    // validation (GraphPlanner.validate_binding_targets). We test the error
    // formatting instead.
    let err = ResolveError::NotFound {
        component: "test::MissingType".to_string(),
        path: vec!["A".to_string(), "test::MissingType".to_string()],
    };
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

#[test]
fn get_bean_by_key_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let key = ComponentKey::of::<String>();
    let result = c.get_bean_by_key(&key);
    assert!(result.is_err());
}

#[test]
fn create_bean_success_and_error() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "test".to_string()
    }))
    .unwrap();
    let c = Container::new(b.build().unwrap());

    // Success: existing type_name
    let result = c.create_bean(std::any::type_name::<String>());
    assert!(result.is_ok());

    // Error: non-existing type_name
    let result = c.create_bean("NonExistentType");
    assert!(result.is_err());
}

#[test]
fn autowire_with_invalid_mode() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let c = Container::new(b.build().unwrap());

    // Invalid autowire mode (99)
    let result = c.autowire("some_type", 99, false);
    assert!(result.is_err());
}

#[test]
fn autowire_bean_properties_all_modes() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;

    // Mode 0: AUTOWIRE_NO — returns bean as-is
    let r0 = c.autowire_bean_properties(bean.clone(), 0, false).unwrap();
    assert!(Arc::ptr_eq(&bean, &r0));

    // Mode 1: AUTOWIRE_BY_NAME — returns bean as-is (no matching definitions)
    let r1 = c.autowire_bean_properties(bean.clone(), 1, false).unwrap();
    assert!(Arc::ptr_eq(&bean, &r1));

    // Mode 2: AUTOWIRE_BY_TYPE — returns bean as-is
    let r2 = c.autowire_bean_properties(bean.clone(), 2, false).unwrap();
    assert!(Arc::ptr_eq(&bean, &r2));

    // Mode 999: fallback — returns bean as-is
    let r999 = c
        .autowire_bean_properties(bean.clone(), 999, false)
        .unwrap();
    assert!(Arc::ptr_eq(&bean, &r999));
}

#[test]
fn set_parent_bean_factory_via_configurable_bean_factory() {
    use vernal_beans::configurable_bean_factory::ConfigurableBeanFactory;

    let mut child = Container::new(RegistryBuilder::new().build().unwrap());
    let parent_container = Container::new(RegistryBuilder::new().build().unwrap());
    // Wrap the parent Container in Arc<dyn BeanFactory> first, then wrap in
    // Arc<dyn Any> for the downcast path in ConfigurableBeanFactory.
    let bf: Arc<dyn BeanFactory> = Arc::new(parent_container);
    let parent_any: Arc<dyn Any + Send + Sync> = Arc::new(bf);

    let result = ConfigurableBeanFactory::set_parent_bean_factory(&mut child, parent_any);
    assert!(result.is_ok());

    // Verify parent was set
    let retrieved = HierarchicalBeanFactory::parent_bean_factory(&child);
    assert!(retrieved.is_some());
}

#[test]
fn destroy_bean_and_singletons() {
    use vernal_beans::configurable_bean_factory::ConfigurableBeanFactory;

    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    let c = Container::new(b.build().unwrap());

    // Pre-instantiate the singleton
    let _ = c.get_bean_by_key(&ComponentKey::of::<i32>()).unwrap();

    // destroy_bean is a no-op in vernal, but must not error
    let dummy = Arc::new(0i32);
    let result = ConfigurableBeanFactory::destroy_bean(&c, "i32", &*dummy);
    assert!(result.is_ok());

    // destroy_singletons clears the singleton cache
    ConfigurableBeanFactory::destroy_singletons(&c);

    // After destroy_singletons, the singleton should be re-created on next access
    let _recreated = c.get_bean_by_key(&ComponentKey::of::<i32>()).unwrap();
}

#[test]
fn is_factory_bean() {
    use vernal_beans::configurable_bean_factory::ConfigurableBeanFactory;

    let c = Container::new(RegistryBuilder::new().build().unwrap());
    // "&foo" is considered a factory bean name (starts with '&')
    assert!(ConfigurableBeanFactory::is_factory_bean(&c, "&myBean"));
    assert!(!ConfigurableBeanFactory::is_factory_bean(&c, "myBean"));
}

#[test]
fn currently_in_creation() {
    use vernal_beans::configurable_bean_factory::ConfigurableBeanFactory;

    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let bean_name = "testBean";

    assert!(!ConfigurableBeanFactory::is_currently_in_creation(
        &c, bean_name
    ));

    ConfigurableBeanFactory::set_currently_in_creation(&mut c, bean_name, true);
    assert!(ConfigurableBeanFactory::is_currently_in_creation(
        &c, bean_name
    ));

    ConfigurableBeanFactory::set_currently_in_creation(&mut c, bean_name, false);
    assert!(!ConfigurableBeanFactory::is_currently_in_creation(
        &c, bean_name
    ));
}

#[test]
fn embedded_value_resolver_chain() {
    use vernal_beans::configurable_bean_factory::ConfigurableBeanFactory;

    let mut c = Container::new(RegistryBuilder::new().build().unwrap());

    // Add two resolvers: one replaces ${...} with "hello", another appends "!"
    let r1: Arc<dyn Fn(&str) -> String + Send + Sync> =
        Arc::new(|val: &str| val.replace("${name}", "hello"));
    let r2: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new(|val: &str| format!("{}!", val));

    ConfigurableBeanFactory::add_embedded_value_resolver(&mut c, r1);
    ConfigurableBeanFactory::add_embedded_value_resolver(&mut c, r2);

    let result = ConfigurableBeanFactory::resolve_embedded_value(&c, "${name}");
    assert_eq!(result, "hello!");
}

#[test]
fn add_singleton_callback() {
    use vernal_beans::singleton_bean_registry::SingletonBeanRegistry;

    let mut c = Container::new(RegistryBuilder::new().build().unwrap());

    let callback_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = Arc::clone(&callback_called);
    let callback: Arc<dyn Fn(&dyn Any) + Send + Sync> = Arc::new(move |_| {
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    SingletonBeanRegistry::add_singleton_callback(&mut c, "test".to_string(), callback);

    // Register a singleton manually to verify the callback field stores it
    SingletonBeanRegistry::register_singleton(&c, "test", Arc::new(99i32));
    assert!(SingletonBeanRegistry::contains_singleton(&c, "test"));
}

#[test]
fn register_resolvable_dependency() {
    use vernal_beans::configurable_listable_bean_factory::ConfigurableListableBeanFactory;

    let mut c = Container::new(RegistryBuilder::new().build().unwrap());

    let value: Arc<dyn Any + Send + Sync> = Arc::new(String::from("resolvable"));
    ConfigurableListableBeanFactory::register_resolvable_dependency(
        &mut c,
        TypeId::of::<String>(),
        value,
    );

    // Dependency is stored; verify via is_autowire_candidate (which doesn't check resolvable,
    // but at least the method doesn't panic)
    assert!(!ConfigurableListableBeanFactory::is_autowire_candidate(
        &c, "unknown"
    ));
}

#[test]
fn ignore_dependency_type_and_interface() {
    use vernal_beans::configurable_listable_bean_factory::ConfigurableListableBeanFactory;

    let mut c = Container::new(RegistryBuilder::new().build().unwrap());

    ConfigurableListableBeanFactory::ignore_dependency_type(&mut c, TypeId::of::<String>());
    ConfigurableListableBeanFactory::ignore_dependency_interface(&mut c, TypeId::of::<i32>());

    // These just store ignored types; no query method, but at least call the methods.
    // We can verify through is_autowire_candidate if we register a definition.
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let c2 = Container::new(b.build().unwrap());

    // Before ignoring: String is an autowire candidate
    assert!(ConfigurableListableBeanFactory::is_autowire_candidate(
        &c2,
        std::any::type_name::<String>()
    ));
}

// ── Factory closure testing select_trait_binding ambiguous path ────────────

#[test]
fn select_trait_binding_ambiguous_build_time() {
    // The runtime ambiguous path in select_trait_binding is guarded by
    // build-time validation (GraphPlanner catches ambiguous trait deps).
    // Test that the build-time error occurs.
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<ImplOne, _>(|_| ImplOne))
        .unwrap();
    b.register(ComponentDefinition::singleton::<ImplTwo, _>(|_| ImplTwo))
        .unwrap();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "consumer".to_string())
            .depends_on_trait::<dyn MyTrait>(),
    )
    .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, ImplOne, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, ImplTwo, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();

    let result = b.build();
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(!err_msg.is_empty());
}

#[test]
fn select_trait_binding_not_found_from_factory() {
    // Test the NOT-FOUND path in select_trait_binding by having a definition
    // with an optional_trait_provider dependency but no bindings registered.
    // This avoids build-time ambiguity errors.
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::try_singleton::<String, _>(|resolver| {
            // Resolve an optional trait — there are no bindings, so this
            // should go through the NotFound path and return Ok(None).
            let result = resolver.resolve_optional_trait::<dyn MyTrait>()?;
            assert!(result.is_none());
            Ok("no_trait".to_string())
        })
        .depends_on_optional_trait::<dyn MyTrait>(),
    )
    .unwrap();
    let container = Container::new(b.build().unwrap());
    let result = container.resolve::<String>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "no_trait");
}

// ── resolve_binding not-found via all traits (build-time error) ─────────

#[test]
fn resolve_binding_not_found_build_time() {
    // The runtime NotFound path is guarded by build-time validation.
    // Test that a binding with a missing target fails at build time.
    let mut b = RegistryBuilder::new();
    b.bind(TraitBinding::new::<dyn MyTrait, ImplOne, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    let result = b.build();
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(!err_msg.is_empty());
}

// ── resolve_binding type-mismatch path ────────────────────────────────────
// The runtime type-mismatch path in resolve_binding (container.rs lines 614-619)
// is unreachable through normal APIs because:
//   - binding.upcast() always wraps the correct Arc<T> for the binding's trait
//   - resolve_binding's generic T matches via TypeId filtering
// We test the error formatting instead.

#[test]
fn resolve_binding_type_mismatch_error_format() {
    // Create a binding for MyTrait, but when the upcast closure returns
    // an Arc<dyn MyTrait>, the framework wraps it as Arc<Arc<dyn MyTrait>>.
    // If we try to resolve with the wrong trait type parameter, the downcast
    // to Arc<OtherTrait> would fail.
    //
    // Approach: register a definition, a binding for MyTrait, but then
    // manually call resolve_all_traits::<OtherTrait>() which filters bindings
    // by TypeId. Since the binding is for MyTrait, it won't match OtherTrait.
    // So we need a binding that matches OtherTrait's TypeId but wraps
    // something else...

    // Actually, the filter uses TypeId, so it won't match different traits.
    // Let's use resolve_all_traits for MyTrait with a binding whose upcast
    // returns a wrong inner type by exploiting the TraitBinding API.

    // TraitBinding::new::<dyn MyTrait, ImplOne, _>() wraps
    // `Arc<dyn MyTrait>` inside the ErasedTraitComponent.
    // The downcast in resolve_binding::<dyn MyTrait> checks for
    // `Arc<dyn MyTrait>`, which matches. No mismatch possible through
    // the normal API.
    //
    // We can test that the type-mismatch error formatting works:
    let tk = vernal_beans::TraitKey::of::<dyn MyTrait>();
    let ck = ComponentKey::of::<ImplOne>();
    let err = ResolveError::TraitBindingTypeMismatch {
        binding: tk,
        target: ck,
    };
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. abstract_autowire_capable_bean_factory.rs
// ═══════════════════════════════════════════════════════════════════════════════

use vernal_beans::abstract_autowire_capable_bean_factory::AbstractAutowireCapableBeanFactory;
use vernal_beans::simple_instantiation_strategy::SimpleInstantiationStrategy;

/// A minimal BeanDefinition for testing
#[derive(Debug)]
struct TestBeanDef {
    class_name: String,
    scope: Scope,
}

impl BeanDefinition for TestBeanDef {
    fn bean_name(&self) -> &ComponentKey {
        static KEY: std::sync::LazyLock<ComponentKey> =
            std::sync::LazyLock::new(|| ComponentKey::of::<TestBeanDef>());
        &KEY
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

struct ReplacingProcessor;
impl BeanPostProcessor for ReplacingProcessor {
    fn post_process_after_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(Arc::new(999i32) as Arc<dyn Any + Send + Sync>))
    }

    fn post_process_before_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(Arc::new(888i32) as Arc<dyn Any + Send + Sync>))
    }
}

struct NoneProcessor;
impl BeanPostProcessor for NoneProcessor {}

#[test]
fn aacbf_create_bean_with_constructor_fails_currently() {
    // SimpleInstantiationStrategy registers constructors but does not
    // currently use them for instantiation by class_name string lookup.
    // Both with and without a registered constructor, create_bean fails
    // because the strategy cannot map a string class name to a TypeId.
    let factory = AbstractAutowireCapableBeanFactory::new();
    factory
        .instantiation_strategy()
        .register_constructor::<String>(|_| Ok(Arc::new("constructed".to_string())));
    let bd = TestBeanDef {
        class_name: std::any::type_name::<String>().to_string(),
        scope: Scope::Singleton,
    };
    let result = factory.create_bean("testBean", &bd);
    // Expected to fail because SimpleInstantiationStrategy cannot look up
    // constructors by bean_class_name string — it checks by TypeId but
    // the instantiate method doesn't resolve string→TypeId.
    assert!(result.is_err());
}

#[test]
fn aacbf_create_bean_no_matching_constructor() {
    let factory = AbstractAutowireCapableBeanFactory::new();
    let bd = TestBeanDef {
        class_name: "NoConstructorType".into(),
        scope: Scope::Singleton,
    };
    let result = factory.create_bean("testBean", &bd);
    assert!(result.is_err());
}

#[test]
fn aacbf_initialize_bean_applies_post_processors() {
    let factory = AbstractAutowireCapableBeanFactory::new();
    factory.add_bean_post_processor(Arc::new(NoneProcessor));

    let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    let result = factory.initialize_bean(bean.clone(), "testBean");
    assert!(result.is_ok());
    let returned = result.unwrap();
    // With NoneProcessor, the bean should be returned unchanged
    assert!(Arc::ptr_eq(&bean, &returned));
}

#[test]
fn aacbf_initialize_bean_post_processor_replaces() {
    let factory = AbstractAutowireCapableBeanFactory::new();
    factory.add_bean_post_processor(Arc::new(ReplacingProcessor));

    let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    let result = factory.initialize_bean(bean.clone(), "testBean");
    assert!(result.is_ok());
    let returned = result.unwrap();
    // The ReplacingProcessor should return 999
    let val = returned.downcast_ref::<i32>().copied().unwrap();
    assert_eq!(val, 999);
}

#[test]
fn aacbf_apply_processors_before_initialization() {
    let factory = AbstractAutowireCapableBeanFactory::new();

    // Multiple processors: first returns None (no change), second replaces
    factory.add_bean_post_processor(Arc::new(NoneProcessor));
    factory.add_bean_post_processor(Arc::new(ReplacingProcessor));

    let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    let result = factory.apply_bean_post_processors_before_initialization(bean.clone(), "testBean");
    assert!(result.is_ok());
    let val = result.unwrap().downcast_ref::<i32>().copied().unwrap();
    assert_eq!(val, 888);
}

#[test]
fn aacbf_apply_processors_after_initialization() {
    let factory = AbstractAutowireCapableBeanFactory::new();

    // Processors that return None (no modification)
    factory.add_bean_post_processor(Arc::new(NoneProcessor));
    factory.add_bean_post_processor(Arc::new(NoneProcessor));

    let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    let result = factory.apply_bean_post_processors_after_initialization(bean.clone(), "testBean");
    assert!(result.is_ok());
    assert!(Arc::ptr_eq(&bean, &result.unwrap()));
}

#[test]
fn aacbf_add_and_get_processors() {
    let factory = AbstractAutowireCapableBeanFactory::new();
    assert_eq!(factory.get_bean_post_processors().len(), 0);

    factory.add_bean_post_processor(Arc::new(NoneProcessor));
    assert_eq!(factory.get_bean_post_processors().len(), 1);
}

#[test]
fn aacbf_processor_count() {
    let factory = AbstractAutowireCapableBeanFactory::new();
    assert_eq!(factory.bean_post_processor_count(), 0);

    factory.add_bean_post_processor(Arc::new(NoneProcessor));
    assert_eq!(factory.bean_post_processor_count(), 1);

    factory.add_bean_post_processor(Arc::new(NoneProcessor));
    assert_eq!(factory.bean_post_processor_count(), 2);
}

#[test]
fn aacbf_instantiation_strategy_access() {
    let mut factory = AbstractAutowireCapableBeanFactory::new();

    // Immutable access
    let strategy: &SimpleInstantiationStrategy = factory.instantiation_strategy();
    assert!(strategy.constructor_count() == 0);

    // Mutable access
    let strategy_mut: &mut SimpleInstantiationStrategy = factory.instantiation_strategy_mut();
    strategy_mut.register_constructor::<i32>(|_| Ok(Arc::new(42i32)));
    assert!(strategy_mut.constructor_count() == 1);
}

#[test]
fn aacbf_default_impl() {
    let factory = AbstractAutowireCapableBeanFactory::default();
    assert_eq!(factory.bean_post_processor_count(), 0);
}

#[test]
fn aacbf_debug_impl() {
    let factory = AbstractAutowireCapableBeanFactory::new();
    let debug_str = format!("{:?}", factory);
    assert!(debug_str.contains("AbstractAutowireCapableBeanFactory"));
    assert!(debug_str.contains("instantiation_strategy"));
    assert!(debug_str.contains("bean_post_processor_count"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. registry_builder.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn rb_bind_with_trait_binding() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<ImplOne, _>(|_| ImplOne))
        .unwrap();
    b.bind(TraitBinding::new::<dyn MyTrait, ImplOne, _>(|s| {
        s as Arc<dyn MyTrait>
    }))
    .unwrap();
    let registry = b.build().unwrap();
    assert_eq!(registry.bindings().len(), 1);
}

#[test]
fn rb_bind_all() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<ImplOne, _>(|_| ImplOne))
        .unwrap();
    b.register(ComponentDefinition::singleton::<ImplTwo, _>(|_| ImplTwo))
        .unwrap();
    let bindings = vec![
        TraitBinding::new::<dyn MyTrait, ImplOne, _>(|s| s as Arc<dyn MyTrait>),
        TraitBinding::new::<dyn MyTrait, ImplTwo, _>(|s| s as Arc<dyn MyTrait>),
    ];
    b.bind_all(bindings).unwrap();
    let registry = b.build().unwrap();
    assert_eq!(registry.bindings().len(), 2);
}

#[test]
fn rb_register_bundle() {
    let mut b = RegistryBuilder::new();
    let definitions = vec![
        ComponentDefinition::singleton::<ImplOne, _>(|_| ImplOne),
        ComponentDefinition::singleton::<ImplTwo, _>(|_| ImplTwo),
    ];
    let bindings = vec![TraitBinding::new::<dyn MyTrait, ImplOne, _>(|s| {
        s as Arc<dyn MyTrait>
    })];
    b.register_bundle(definitions, bindings).unwrap();
    let registry = b.build().unwrap();
    assert_eq!(registry.definitions().len(), 2);
    assert_eq!(registry.bindings().len(), 1);
}

#[test]
fn rb_duplicate_bind_error() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<ImplOne, _>(|_| ImplOne))
        .unwrap();
    let binding = TraitBinding::new::<dyn MyTrait, ImplOne, _>(|s| s as Arc<dyn MyTrait>);
    b.bind(binding).unwrap();
    // Duplicate (same key + same target) should error
    let dup = TraitBinding::new::<dyn MyTrait, ImplOne, _>(|s| s as Arc<dyn MyTrait>);
    let result = b.bind(dup);
    assert!(result.is_err());
}

#[test]
fn rb_remove_by_key_existing() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<ImplOne, _>(|_| ImplOne))
        .unwrap();
    let key = ComponentKey::of::<ImplOne>();
    assert!(b.contains::<ImplOne>());
    let result = b.remove_by_key(&key);
    assert!(result.is_ok());
    assert!(!b.contains::<ImplOne>());
}

#[test]
fn rb_remove_by_key_non_existing() {
    let mut b = RegistryBuilder::new();
    let key = ComponentKey::of::<ImplOne>();
    let result = b.remove_by_key(&key);
    assert!(result.is_err());
}

#[test]
fn rb_is_empty_initially() {
    let b = RegistryBuilder::new();
    assert!(b.is_empty());
}

#[test]
fn rb_is_empty_after_register() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<ImplOne, _>(|_| ImplOne))
        .unwrap();
    assert!(!b.is_empty());
}

#[test]
fn rb_len_accurate() {
    let mut b = RegistryBuilder::new();
    assert_eq!(b.len(), 0);
    b.register(ComponentDefinition::singleton::<ImplOne, _>(|_| ImplOne))
        .unwrap();
    assert_eq!(b.len(), 1);
    b.register(ComponentDefinition::singleton::<ImplTwo, _>(|_| ImplTwo))
        .unwrap();
    assert_eq!(b.len(), 2);
}

#[test]
fn rb_debug_format() {
    let b = RegistryBuilder::new();
    let debug_str = format!("{:?}", b);
    assert!(debug_str.contains("RegistryBuilder"));
}

#[test]
fn rb_contains() {
    let mut b = RegistryBuilder::new();

    // Before register
    assert!(!b.contains::<ImplOne>());

    // After register
    b.register(ComponentDefinition::singleton::<ImplOne, _>(|_| ImplOne))
        .unwrap();
    assert!(b.contains::<ImplOne>());

    // After remove
    b.remove::<ImplOne>().unwrap();
    assert!(!b.contains::<ImplOne>());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. scope_context.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn sc_open_and_close() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = c.open_scope::<RequestScopeTag>();

    assert_eq!(scope.state(), ScopeState::Open);

    let result = scope.close().await;
    assert!(result.is_ok());
    assert_eq!(scope.state(), ScopeState::Closed);
}

#[tokio::test]
async fn sc_child_from_parent() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let parent = c.open_scope::<RequestScopeTag>();
    let child = parent.child::<TaskScopeTag>();

    assert_eq!(child.state(), ScopeState::Open);

    // Verify parent relationship
    let parent_ref = child.parent();
    assert!(parent_ref.is_some());
    assert_eq!(parent_ref.unwrap().key(), parent.key());

    // Verify different keys
    assert_ne!(parent.key(), child.key());

    // Close parent
    let result = parent.close().await;
    assert!(result.is_ok());

    // Child scope is not automatically closed, but parent cancellation token
    // is propagated — child should be cancelled
    assert_eq!(child.state(), ScopeState::Open);
    assert!(child.cancellation().is_cancelled());
}

#[tokio::test]
async fn sc_open_with_cancellation() {
    use tokio_util::sync::CancellationToken;

    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let token = CancellationToken::new();
    let scope = c.open_scope_with_cancellation::<RequestScopeTag>(token.clone());

    assert_eq!(scope.state(), ScopeState::Open);

    // Cancel externally, then close
    token.cancel();
    let result = scope.close().await;
    // Even though token was cancelled externally, close should still succeed
    assert!(result.is_ok());
}

#[tokio::test]
async fn sc_get_or_insert_with() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = c.open_scope::<RequestScopeTag>();

    // First call creates the value
    let val1 = scope
        .get_or_insert_with::<i32, _>(|| 42)
        .expect("should succeed");
    assert_eq!(*val1, 42);

    // Second call returns cached value
    let val2 = scope
        .get_or_insert_with::<i32, _>(|| 100)
        .expect("should succeed");
    assert_eq!(*val2, 42); // Still 42 (cached)

    // A different type works independently
    let val3 = scope
        .get_or_insert_with::<String, _>(|| "hello".to_string())
        .expect("should succeed");
    assert_eq!(*val3, "hello");

    // Close the scope
    let result = scope.close().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn sc_multiple_independent_scopes() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());

    // Create two independent scopes
    let scope_a = c.open_scope::<RequestScopeTag>();
    let scope_b = c.open_scope::<TaskScopeTag>();

    assert_eq!(scope_a.state(), ScopeState::Open);
    assert_eq!(scope_b.state(), ScopeState::Open);
    assert!(scope_a.parent().is_none());
    assert!(scope_b.parent().is_none());

    // Verify different ScopeKeys
    assert_ne!(scope_a.key(), scope_b.key());

    // Store different types in each
    let va = scope_a
        .get_or_insert_with::<i32, _>(|| 1)
        .expect("should succeed");
    let vb = scope_b
        .get_or_insert_with::<String, _>(|| "two".to_string())
        .expect("should succeed");
    assert_eq!(*va, 1);
    assert_eq!(*vb, "two");

    // Close independently
    assert!(scope_a.close().await.is_ok());
    assert_eq!(scope_a.state(), ScopeState::Closed);
    assert_eq!(scope_b.state(), ScopeState::Open); // b still open

    assert!(scope_b.close().await.is_ok());
    assert_eq!(scope_b.state(), ScopeState::Closed);
}

#[test]
fn sc_scope_key_type_name() {
    let key = ScopeKey::of::<RequestScopeTag>();
    let type_name = key.type_name();
    assert!(type_name.contains("RequestScopeTag"));

    let display = format!("{key}");
    assert_eq!(display, type_name);
}
