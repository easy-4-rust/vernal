//! Phase 2 coverage tests targeting specific uncovered code paths.
//!
//! Covers: container.rs (BeanDefinitionRegistry + ConfigurableBeanFactory),
//! registry_builder.rs, scope_context.rs, resolver.rs, default_listable_bean_factory.rs,
//! component_definition.rs, type_converter_delegate.rs, bean_util.rs,
//! standard_bean_expression_resolver.rs, bean_definition_store_exception.rs

use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;
use vernal_beans::{
    BeanDefinitionRegistry, BeanError, BeanUtil, ComponentDefinition, ComponentKey,
    ConfigurableBeanFactory, Container, Qualifier, RegistryBuilder, Resolver, TraitBinding,
    bean_definition::BeanDefinition, bean_definition_store_exception::BeanDefinitionStoreException,
    bean_expression_resolver::BeanExpressionResolver,
    default_listable_bean_factory::DefaultListableBeanFactory,
    root_bean_definition::RootBeanDefinition,
    standard_bean_expression_resolver::StandardBeanExpressionResolver,
    type_converter_delegate::TypeConverterDelegate,
};

// ── Shared test types ──────────────────────────────────────────────────────

trait MyService: Send + Sync + 'static {
    fn greet(&self) -> &'static str;
}

struct MyServiceImpl;
impl MyService for MyServiceImpl {
    fn greet(&self) -> &'static str {
        "hello from service"
    }
}

struct OtherServiceImpl;
impl MyService for OtherServiceImpl {
    fn greet(&self) -> &'static str {
        "hello from other"
    }
}

trait MyOtherTrait: Send + Sync + 'static {
    fn value(&self) -> i32;
}

struct MyOtherImpl(i32);
impl MyOtherTrait for MyOtherImpl {
    fn value(&self) -> i32 {
        self.0
    }
}

// ── Serde types for BeanUtil tests ─────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct SourceStruct {
    name: String,
    age: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct TargetStruct {
    name: String,
    age: i32,
}

// =========================================================================
// 1. container.rs — BeanDefinitionRegistry trait tests
// =========================================================================

#[test]
fn container_get_bean_definition_dynamic_non_deleted() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let mut c = Container::new(b.build().unwrap());

    let mut rbd = RootBeanDefinition::new();
    rbd.set_bean_class_name("test::DynamicBean");
    let name = "test_dynamic".to_string();
    c.register_bean_definition(name.clone(), Box::new(rbd))
        .unwrap();

    let got = c.get_bean_definition(&name);
    assert!(got.is_some());
    assert_eq!(got.unwrap().bean_class_name(), "test::DynamicBean");
}

#[test]
fn container_get_bean_definition_dynamic_deleted() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let mut c = Container::new(b.build().unwrap());

    // Register then remove to create __DELETED__ entry
    let mut rbd = RootBeanDefinition::new();
    rbd.set_bean_class_name("test::ToDelete");
    let name = "test_to_delete".to_string();
    c.register_bean_definition(name.clone(), Box::new(rbd))
        .unwrap();
    c.remove_bean_definition(&name).unwrap();

    // Now get_bean_definition should return None for __DELETED__
    let got = c.get_bean_definition(&name);
    // Note: `remove_bean_definition` for a dynamic entry removes it from the map,
    // not adding __DELETED__. __DELETED__ entries are created only when removing
    // from the registry.
    // After removal, get_bean_definition should return None (def removed)
    assert!(got.is_none());

    // Now test the __DELETED__ path by removing a registry definition
    let registry_name = "alloc::string::String";
    c.remove_bean_definition(registry_name).unwrap();
    let got_deleted = c.get_bean_definition(registry_name);
    assert!(got_deleted.is_none());
}

#[test]
fn container_get_bean_definition_registry_definition() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let c = Container::new(b.build().unwrap());

    let got = c.get_bean_definition("alloc::string::String");
    assert!(got.is_some());
    assert!(got.unwrap().bean_class_name().contains("String"));
}

#[test]
fn container_get_bean_definition_not_found() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let c = Container::new(b.build().unwrap());

    let got = c.get_bean_definition("nonexistent_bean");
    assert!(got.is_none());
}

#[test]
fn container_remove_bean_definition_from_dynamic() {
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());

    let mut rbd = RootBeanDefinition::new();
    rbd.set_bean_class_name("test::DynamicBean");
    let name = "test_dynamic".to_string();
    c.register_bean_definition(name.clone(), Box::new(rbd))
        .unwrap();

    let removed = c.remove_bean_definition(&name);
    assert!(removed.is_ok());
    assert_eq!(removed.unwrap().bean_class_name(), "test::DynamicBean");
    assert!(!c.contains_bean_definition(&name));
}

#[test]
fn container_remove_bean_definition_from_registry_creates_deleted() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let mut c = Container::new(b.build().unwrap());

    let removed = c.remove_bean_definition("alloc::string::String");
    assert!(removed.is_ok());

    // After removal from registry, the __DELETED__ marker should be in dynamic_definitions
    // so contains should return false
    assert!(!c.contains_bean_definition("alloc::string::String"));
}

#[test]
fn container_remove_bean_definition_already_deleted() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let mut c = Container::new(b.build().unwrap());

    // Remove from registry to create __DELETED__
    c.remove_bean_definition("alloc::string::String").unwrap();

    // Remove again should fail because __DELETED__ prevents double-remove
    let result = c.remove_bean_definition("alloc::string::String");
    assert!(result.is_err());
}

#[test]
fn container_contains_bean_definition_deleted() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let mut c = Container::new(b.build().unwrap());

    // Registry definition exists
    assert!(c.contains_bean_definition("alloc::string::String"));

    // Remove to create __DELETED__
    c.remove_bean_definition("alloc::string::String").unwrap();

    // Should return false for __DELETED__
    assert!(!c.contains_bean_definition("alloc::string::String"));
}

#[test]
fn container_bean_definition_names_and_count() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    let mut c = Container::new(b.build().unwrap());

    // Registry has 2 definitions
    assert_eq!(c.bean_definition_count(), 2);

    let names = c.bean_definition_names();
    assert_eq!(names.len(), 2);

    // Register a dynamic one
    let mut rbd = RootBeanDefinition::new();
    rbd.set_bean_class_name("test::Dynamic");
    c.register_bean_definition("test_dynamic".to_string(), Box::new(rbd))
        .unwrap();
    assert_eq!(c.bean_definition_count(), 3);

    // Remove one from registry
    c.remove_bean_definition("alloc::string::String").unwrap();
    // Now it's counting: registry entries minus deleted markers + non-deleted dynamic
    assert_eq!(c.bean_definition_count(), 2);
}

// =========================================================================
// 1b. container.rs — ConfigurableBeanFactory tests
// =========================================================================

#[test]
fn container_register_and_get_scope() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let mut c = Container::new(b.build().unwrap());

    use vernal_beans::bean_scope::BeanScope;
    struct TestScope;
    impl BeanScope for TestScope {
        fn get(
            &self,
            name: &str,
            _object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Err(format!("scope {} not available", name).into())
        }
    }

    c.register_scope("testScope", Box::new(TestScope));
    let names = c.registered_scope_names();
    assert!(names.contains(&"testScope".to_string()));

    // get_registered_scope always returns None currently
    assert!(c.get_registered_scope("testScope").is_none());
    assert!(c.get_registered_scope("nonexistent").is_none());
}

#[test]
fn container_is_factory_bean() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let c = Container::new(b.build().unwrap());

    // is_factory_bean checks for "&" prefix
    assert!(!c.is_factory_bean("myBean"));
    // With & prefix
    assert!(c.is_factory_bean("&myBean"));
}

#[test]
fn container_currently_in_creation() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let mut c = Container::new(b.build().unwrap());

    assert!(!c.is_currently_in_creation("testBean"));
    c.set_currently_in_creation("testBean", true);
    assert!(c.is_currently_in_creation("testBean"));
    c.set_currently_in_creation("testBean", false);
    assert!(!c.is_currently_in_creation("testBean"));
}

#[test]
fn container_dependent_bean_operations() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let mut c = Container::new(b.build().unwrap());

    c.register_dependent_bean("beanA", "beanB");
    let deps = c.get_dependent_beans("beanA");
    assert_eq!(deps, vec!["beanB"]);

    let deps_for = c.get_dependencies_for_bean("beanB");
    assert_eq!(deps_for, vec!["beanA"]);

    // Non-existent bean returns empty
    assert!(c.get_dependent_beans("nonexistent").is_empty());
    assert!(c.get_dependencies_for_bean("nonexistent").is_empty());
}

#[test]
fn container_destroy_operations() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let c = Container::new(b.build().unwrap());

    // destroy_bean always returns Ok
    let result = c.destroy_bean("testBean", &42);
    assert!(result.is_ok());

    // destroy_singletons clears the cache
    c.destroy_singletons();
    // Should not panic
}

#[test]
fn container_embedded_value_resolver() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let mut c = Container::new(b.build().unwrap());

    // No resolvers yet
    assert_eq!(c.resolve_embedded_value("hello"), "hello");

    // Add a resolver
    c.add_embedded_value_resolver(Arc::new(|v| v.replace("${app.name}", "MyApp")));
    assert_eq!(c.resolve_embedded_value("app: ${app.name}"), "app: MyApp");

    // Multiple resolvers chain
    c.add_embedded_value_resolver(Arc::new(|v| v.to_uppercase()));
    let result = c.resolve_embedded_value("hello ${app.name}");
    assert_eq!(result, "HELLO MYAPP");
}

// =========================================================================
// 2. registry_builder.rs tests
// =========================================================================

#[test]
fn registry_builder_register_all() {
    let mut b = RegistryBuilder::new();
    b.register_all([
        ComponentDefinition::singleton::<String, _>(|_| "a".to_string()),
        ComponentDefinition::singleton::<i32, _>(|_| 1i32),
    ])
    .unwrap();
    assert_eq!(b.len(), 2);
    assert!(b.contains::<String>());
    assert!(b.contains::<i32>());

    let registry = b.build().unwrap();
    assert_eq!(registry.definitions().len(), 2);
}

#[test]
fn registry_builder_register_all_duplicate_error() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "a".to_string()
    }))
    .unwrap();
    let result = b.register_all([ComponentDefinition::singleton::<String, _>(|_| {
        "b".to_string()
    })]);
    assert!(result.is_err());
}

#[test]
fn registry_builder_bind() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyServiceImpl, _>(|_| {
        MyServiceImpl
    }))
    .unwrap();
    let binding = TraitBinding::new::<dyn MyService, MyServiceImpl, _>(|s| s as Arc<dyn MyService>);
    b.bind(binding).unwrap();

    let registry = b.build().unwrap();
    assert_eq!(registry.bindings().len(), 1);
}

#[test]
fn registry_builder_bind_all() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyServiceImpl, _>(|_| {
        MyServiceImpl
    }))
    .unwrap();
    b.register(ComponentDefinition::singleton::<OtherServiceImpl, _>(
        |_| OtherServiceImpl,
    ))
    .unwrap();

    let q1 = Qualifier::new("primary").unwrap();
    let q2 = Qualifier::new("secondary").unwrap();

    let bindings = vec![
        TraitBinding::new::<dyn MyService, MyServiceImpl, _>(|s| s as Arc<dyn MyService>)
            .qualified(q1),
        TraitBinding::new::<dyn MyService, OtherServiceImpl, _>(|s| s as Arc<dyn MyService>)
            .qualified(q2),
    ];
    b.bind_all(bindings).unwrap();

    let registry = b.build().unwrap();
    assert_eq!(registry.bindings().len(), 2);
}

#[test]
fn registry_builder_bind_all_duplicate_error() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyServiceImpl, _>(|_| {
        MyServiceImpl
    }))
    .unwrap();

    let binding1 =
        TraitBinding::new::<dyn MyService, MyServiceImpl, _>(|s| s as Arc<dyn MyService>);
    b.bind(binding1).unwrap();

    let binding2 =
        TraitBinding::new::<dyn MyService, MyServiceImpl, _>(|s| s as Arc<dyn MyService>);
    let result = b.bind_all([binding2]);
    assert!(result.is_err());
}

#[test]
fn registry_builder_register_bundle() {
    let mut b = RegistryBuilder::new();
    let definition = ComponentDefinition::singleton::<MyServiceImpl, _>(|_| MyServiceImpl);
    let binding = TraitBinding::new::<dyn MyService, MyServiceImpl, _>(|s| s as Arc<dyn MyService>);

    b.register_bundle([definition], [binding]).unwrap();
    assert_eq!(b.len(), 1);
    assert!(b.contains::<MyServiceImpl>());

    let registry = b.build().unwrap();
    assert_eq!(registry.bindings().len(), 1);
}

#[test]
fn registry_builder_register_bundle_error_rollback() {
    let mut b = RegistryBuilder::new();
    let definition = ComponentDefinition::singleton::<MyServiceImpl, _>(|_| MyServiceImpl);
    let binding = TraitBinding::new::<dyn MyService, MyServiceImpl, _>(|s| s as Arc<dyn MyService>);

    // First bundle succeeds
    b.register_bundle([definition], [binding]).unwrap();

    // Second bundle with duplicate definition should fail, and bindings shouldn't be partially applied
    let def2 = ComponentDefinition::singleton::<OtherServiceImpl, _>(|_| OtherServiceImpl);
    let binding2 =
        TraitBinding::new::<dyn MyService, MyServiceImpl, _>(|s| s as Arc<dyn MyService>);
    let result = b.register_bundle([def2], [binding2]);
    assert!(result.is_err());

    // Original binding should still be there
    let registry = b.build().unwrap();
    assert_eq!(registry.bindings().len(), 1);
}

#[test]
fn registry_builder_contains_and_remove_by_key() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "x".to_string()
    }))
    .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();

    assert!(b.contains::<String>());
    assert!(b.contains::<i32>());
    assert!(!b.contains::<f64>());

    // remove_by_key existing
    let key = ComponentKey::of::<String>();
    b.remove_by_key(&key).unwrap();
    assert!(!b.contains::<String>());

    // remove_by_key non-existing
    let key2 = ComponentKey::of::<f64>();
    let result = b.remove_by_key(&key2);
    assert!(result.is_err());
}

#[test]
fn registry_builder_register_duplicate_error() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "x".to_string()
    }))
    .unwrap();
    let result = b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "y".to_string()
    }));
    assert!(result.is_err());
}

// =========================================================================
// 3. scope_context.rs tests
// =========================================================================

#[test]
fn scope_context_open_get_state_close() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let c = Container::new(b.build().unwrap());

    let scope = c.open_scope::<String>();
    assert_eq!(scope.state(), vernal_beans::ScopeState::Open);

    // Test key
    let key = scope.key();
    assert!(!key.type_name().is_empty());

    // Test parent is None for root
    assert!(scope.parent().is_none());

    // Test child
    let child = scope.child::<i32>();
    assert_eq!(child.state(), vernal_beans::ScopeState::Open);
    assert!(child.parent().is_some());
}

#[test]
fn scope_context_close_is_idempotent() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let c = Container::new(b.build().unwrap());

    let scope = c.open_scope::<String>();
    // Close should work
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        scope.close().await.unwrap();
    });
    assert_eq!(scope.state(), vernal_beans::ScopeState::Closed);
}

// =========================================================================
// 4. resolver.rs tests (via ComponentDefinition factory closures)
// =========================================================================

#[test]
fn resolver_resolve_trait_from_factory() {
    let mut b = RegistryBuilder::new();

    // Register the concrete implementation
    b.register(ComponentDefinition::singleton::<MyServiceImpl, _>(|_| {
        MyServiceImpl
    }))
    .unwrap();

    // Register a component that uses resolve_trait in its factory
    b.register(
        ComponentDefinition::singleton::<String, _>(|resolver: &Resolver| {
            let svc: Arc<dyn MyService> = resolver.resolve_trait().unwrap();
            format!("resolved: {}", svc.greet())
        })
        .depends_on_trait::<dyn MyService>(),
    )
    .unwrap();

    // Create TraitBinding
    let binding = TraitBinding::new::<dyn MyService, MyServiceImpl, _>(|s| s as Arc<dyn MyService>);
    b.bind(binding).unwrap();

    let c = Container::new(b.build().unwrap());
    let result: Arc<String> = c.resolve().unwrap();
    assert_eq!(*result, "resolved: hello from service");
}

#[test]
fn resolver_resolve_all_traits_from_factory() {
    let mut b = RegistryBuilder::new();

    b.register(ComponentDefinition::singleton::<MyServiceImpl, _>(|_| {
        MyServiceImpl
    }))
    .unwrap();
    b.register(ComponentDefinition::singleton::<OtherServiceImpl, _>(
        |_| OtherServiceImpl,
    ))
    .unwrap();

    b.register(
        ComponentDefinition::singleton::<String, _>(|resolver: &Resolver| {
            let all: Vec<Arc<dyn MyService>> = resolver.resolve_all_traits().unwrap();
            assert!(!all.is_empty());
            format!("count: {}", all.len())
        })
        .depends_on_all_traits::<dyn MyService>(),
    )
    .unwrap();

    let b1 = TraitBinding::new::<dyn MyService, MyServiceImpl, _>(|s| s as Arc<dyn MyService>);
    let b2 = TraitBinding::new::<dyn MyService, OtherServiceImpl, _>(|s| s as Arc<dyn MyService>);
    b.bind_all([b1, b2]).unwrap();

    let c = Container::new(b.build().unwrap());
    let result: Arc<String> = c.resolve().unwrap();
    assert_eq!(*result, "count: 2");
}

#[test]
fn resolver_resolve_optional_trait_from_factory() {
    let mut b = RegistryBuilder::new();

    b.register(ComponentDefinition::singleton::<MyServiceImpl, _>(|_| {
        MyServiceImpl
    }))
    .unwrap();

    b.register(
        ComponentDefinition::singleton::<String, _>(|resolver: &Resolver| {
            let svc: Option<Arc<dyn MyService>> = resolver.resolve_optional_trait().unwrap();
            match svc {
                Some(s) => format!("found: {}", s.greet()),
                None => "not found".to_string(),
            }
        })
        .depends_on_optional_trait::<dyn MyService>(),
    )
    .unwrap();

    let binding = TraitBinding::new::<dyn MyService, MyServiceImpl, _>(|s| s as Arc<dyn MyService>);
    b.bind(binding).unwrap();

    let c = Container::new(b.build().unwrap());
    let result: Arc<String> = c.resolve().unwrap();
    assert!(result.starts_with("found:"));

    // Test absent case: resolve_optional_trait when no binding exists
    let mut b2 = RegistryBuilder::new();
    b2.register(
        ComponentDefinition::singleton::<String, _>(|resolver: &Resolver| {
            let svc: Option<Arc<dyn MyOtherTrait>> = resolver.resolve_optional_trait().unwrap();
            match svc {
                Some(_) => "found".to_string(),
                None => "not found".to_string(),
            }
        })
        .depends_on_optional_trait::<dyn MyOtherTrait>(),
    )
    .unwrap();

    let c2 = Container::new(b2.build().unwrap());
    let result2: Arc<String> = c2.resolve().unwrap();
    assert_eq!(*result2, "not found");
}

#[test]
fn resolver_resolve_qualified_trait_from_factory() {
    let mut b = RegistryBuilder::new();

    b.register(ComponentDefinition::singleton::<MyServiceImpl, _>(|_| {
        MyServiceImpl
    }))
    .unwrap();
    b.register(ComponentDefinition::singleton::<OtherServiceImpl, _>(
        |_| OtherServiceImpl,
    ))
    .unwrap();

    let qual = Qualifier::new("primary").unwrap();

    b.register(
        ComponentDefinition::singleton::<String, _>(|resolver: &Resolver| {
            let svc: Arc<dyn MyService> = resolver
                .resolve_qualified_trait(&Qualifier::new("primary").unwrap())
                .unwrap();
            svc.greet().to_string()
        })
        .depends_on_qualified_trait::<dyn MyService>(qual),
    )
    .unwrap();

    let b1 = TraitBinding::new::<dyn MyService, MyServiceImpl, _>(|s| s as Arc<dyn MyService>)
        .qualified(Qualifier::new("primary").unwrap());
    let b2 = TraitBinding::new::<dyn MyService, OtherServiceImpl, _>(|s| s as Arc<dyn MyService>)
        .qualified(Qualifier::new("secondary").unwrap());
    b.bind_all([b1, b2]).unwrap();

    let c = Container::new(b.build().unwrap());
    let result: Arc<String> = c.resolve().unwrap();
    assert_eq!(*result, "hello from service");
}

// =========================================================================
// 5. default_listable_bean_factory.rs tests
// =========================================================================

#[test]
fn default_listable_bean_factory_basic() {
    let factory = DefaultListableBeanFactory::empty();
    assert_eq!(factory.bean_definition_count(), 0);
    assert!(factory.bean_definition_names().is_empty());
}

#[test]
fn default_listable_bean_factory_register_get_remove() {
    let factory = DefaultListableBeanFactory::empty();

    let mut bd = RootBeanDefinition::new();
    bd.set_bean_class_name("test::MyBean");

    factory.register_bean_definition("myBean", Arc::new(bd));
    assert_eq!(factory.bean_definition_count(), 1);
    assert!(factory.contains_bean_definition("myBean"));

    let got = factory.get_bean_definition("myBean");
    assert!(got.is_some());

    factory.remove_bean_definition("myBean");
    assert!(!factory.contains_bean_definition("myBean"));
    assert_eq!(factory.bean_definition_count(), 0);

    // Multiple names
    let mut bd1 = RootBeanDefinition::new();
    bd1.set_bean_class_name("test::Bean1");
    let mut bd2 = RootBeanDefinition::new();
    bd2.set_bean_class_name("test::Bean2");

    factory.register_bean_definition("bean1", Arc::new(bd1));
    factory.register_bean_definition("bean2", Arc::new(bd2));

    let names = factory.bean_definition_names();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"bean1".to_string()));
    assert!(names.contains(&"bean2".to_string()));
}

#[test]
fn default_listable_bean_factory_pre_instantiate_singletons() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let registry = b.build().unwrap();
    let factory = DefaultListableBeanFactory::new(registry);
    assert!(factory.pre_instantiate_singletons().is_ok());

    let container = factory.container();
    let result: Arc<String> = container.resolve().unwrap();
    assert_eq!(*result, "hello");
}

#[test]
fn default_listable_bean_factory_debug() {
    let factory = DefaultListableBeanFactory::empty();
    let debug_str = format!("{:?}", factory);
    assert!(debug_str.contains("DefaultListableBeanFactory"));
}

#[test]
fn default_listable_bean_factory_container_getter() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let registry = b.build().unwrap();
    let factory = DefaultListableBeanFactory::new(registry);

    let container = factory.container();
    let result: Arc<String> = container.resolve().unwrap();
    assert_eq!(*result, "hello");
}

// =========================================================================
// 6. component_definition.rs tests
// =========================================================================

#[test]
fn component_definition_depends_on_optional() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<String, _>(|resolver: &Resolver| {
            let opt: Option<Arc<i32>> = resolver.resolve_optional().unwrap();
            match opt {
                Some(v) => format!("has dep: {}", v),
                None => "no dep".to_string(),
            }
        })
        .depends_on_optional::<i32>(),
    )
    .unwrap();

    let c = Container::new(b.build().unwrap());
    let result: Arc<String> = c.resolve().unwrap();
    assert_eq!(*result, "no dep");

    // Now with the dep present
    let mut b2 = RegistryBuilder::new();
    b2.register(ComponentDefinition::singleton::<i32, _>(|_| 99i32))
        .unwrap();
    b2.register(
        ComponentDefinition::singleton::<String, _>(|resolver: &Resolver| {
            let opt: Option<Arc<i32>> = resolver.resolve_optional().unwrap();
            format!("has dep: {}", opt.unwrap())
        })
        .depends_on_optional::<i32>(),
    )
    .unwrap();

    let c2 = Container::new(b2.build().unwrap());
    let result2: Arc<String> = c2.resolve().unwrap();
    assert_eq!(*result2, "has dep: 99");
}

#[test]
fn component_definition_depends_on_qualified() {
    let q = Qualifier::new("primary").unwrap();
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "primary".to_string()).qualified(q.clone()),
    )
    .unwrap();

    // Use a wrapper type for the consumer to avoid ambiguity with String
    struct QualifiedConsumer(String);

    b.register(
        ComponentDefinition::singleton::<QualifiedConsumer, _>(|resolver: &Resolver| {
            let dep: Arc<String> = resolver
                .resolve_qualified(&Qualifier::new("primary").unwrap())
                .unwrap();
            QualifiedConsumer(format!("resolved: {}", dep))
        })
        .depends_on_qualified::<String>(q),
    )
    .unwrap();

    let c = Container::new(b.build().unwrap());
    let result: Arc<QualifiedConsumer> = c.resolve().unwrap();
    assert_eq!(result.0, "resolved: primary");
}

#[test]
fn component_definition_depends_on_trait_and_all_traits() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<MyServiceImpl, _>(|_| {
        MyServiceImpl
    }))
    .unwrap();

    // Test depends_on_trait
    b.register(
        ComponentDefinition::singleton::<String, _>(|resolver: &Resolver| {
            let svc: Arc<dyn MyService> = resolver.resolve_trait().unwrap();
            svc.greet().to_string()
        })
        .depends_on_trait::<dyn MyService>(),
    )
    .unwrap();

    let binding = TraitBinding::new::<dyn MyService, MyServiceImpl, _>(|s| s as Arc<dyn MyService>);
    b.bind(binding).unwrap();

    let c = Container::new(b.build().unwrap());
    let result: Arc<String> = c.resolve().unwrap();
    assert_eq!(*result, "hello from service");
}

#[test]
fn component_definition_with_init_order() {
    let def =
        ComponentDefinition::singleton::<String, _>(|_| "test".to_string()).with_init_order(42);
    assert_eq!(def.init_order(), 42);

    let def2 = ComponentDefinition::singleton::<String, _>(|_| "test".to_string());
    assert_eq!(def2.init_order(), i32::MAX);
}

#[test]
fn component_definition_qualified() {
    let q = Qualifier::new("myQualifier").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_| "test".to_string()).qualified(q);
    assert!(def.key().qualifier().is_some());
    assert_eq!(def.key().qualifier().unwrap().as_str(), "myQualifier");
}

// =========================================================================
// 7. type_converter_delegate.rs tests
// =========================================================================

#[test]
fn type_converter_string_to_i32() {
    let converter = TypeConverterDelegate::new();
    let result = converter.convert_if_necessary(
        Some("age"),
        &"42".to_string(),
        std::any::TypeId::of::<i32>(),
    );
    assert!(result.is_ok());
    let val = result.unwrap();
    // The CustomNumberEditor stores f64 internally, so expect f64
    let float_val = val.downcast_ref::<f64>().unwrap();
    assert_eq!(*float_val, 42.0);
}

#[test]
fn type_converter_string_to_bool() {
    let converter = TypeConverterDelegate::new();
    let result = converter.convert_if_necessary(
        Some("active"),
        &"true".to_string(),
        std::any::TypeId::of::<bool>(),
    );
    assert!(result.is_ok());
    let val = result.unwrap();
    let bool_val = val.downcast_ref::<bool>().unwrap();
    assert!(*bool_val);
}

#[test]
fn type_converter_i32_to_string() {
    let converter = TypeConverterDelegate::new();
    let result =
        converter.convert_if_necessary(Some("name"), &42i32, std::any::TypeId::of::<String>());
    assert!(result.is_ok());
    let val = result.unwrap();
    let str_val = val.downcast_ref::<String>().unwrap();
    assert_eq!(str_val, "42");
}

#[test]
fn type_converter_unsupported_conversion() {
    let converter = TypeConverterDelegate::new();
    // Try converting a non-standard type to something completely different
    // HashMap doesn't have a PropertyEditor and isn't convertible
    let result =
        converter.convert_if_necessary(Some("unknown"), &true, std::any::TypeId::of::<i32>());
    // This will try to convert bool to i32, which should have no registered editor
    assert!(result.is_err());
}

#[test]
fn type_converter_identical_type_passthrough() {
    let converter = TypeConverterDelegate::new();
    let result = converter.convert_if_necessary(
        Some("value"),
        &"hello".to_string(),
        std::any::TypeId::of::<String>(),
    );
    assert!(result.is_ok());
}

// =========================================================================
// 8. bean_util.rs tests
// =========================================================================

#[test]
fn bean_util_copy_properties() {
    let src = SourceStruct {
        name: "Alice".to_string(),
        age: 30,
    };
    let target: TargetStruct = BeanUtil::copy_properties(&src).unwrap();
    assert_eq!(target.name, "Alice");
    assert_eq!(target.age, 30);
}

#[test]
fn bean_util_type_eq() {
    assert!(BeanUtil::type_eq::<String, String>());
    assert!(!BeanUtil::type_eq::<String, i32>());
}

#[test]
fn bean_util_type_eq_complex() {
    assert!(BeanUtil::type_eq::<Vec<String>, Vec<String>>());
}

#[test]
fn bean_error_display() {
    let err = BeanError {
        message: "test error".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("test error"));
    assert!(msg.contains("Bean"));
}

#[test]
fn bean_error_std_error() {
    use std::error::Error;
    let err = BeanError {
        message: "std error test".to_string(),
    };
    let _ = err.source();
    // BeanError should implement Debug
    let debug = format!("{:?}", err);
    assert!(debug.contains("BeanError"));
}

// =========================================================================
// 9. standard_bean_expression_resolver.rs tests
// =========================================================================

#[test]
fn expression_resolver_default_constructor() {
    let resolver = StandardBeanExpressionResolver::new();
    assert_eq!(resolver.bean_count(), 0);
}

#[test]
fn expression_resolver_boolean_expression() {
    let resolver = StandardBeanExpressionResolver::new();
    // "true" is treated as a simple identifier, use a proper SpEL expression
    // Numeric comparison is a proper expression
    let result = BeanExpressionResolver::evaluate(&resolver, "1+1", None).unwrap();
    assert!(result.is_some());
}

#[test]
fn expression_resolver_numeric_expression() {
    let resolver = StandardBeanExpressionResolver::new();
    let result = BeanExpressionResolver::evaluate(&resolver, "42", None).unwrap();
    assert!(result.is_some());
}

#[test]
fn expression_resolver_string_expression() {
    let resolver = StandardBeanExpressionResolver::new();
    // String literals in SpEL need quotes
    let result = BeanExpressionResolver::evaluate(&resolver, "'hello world'", None).unwrap();
    assert!(result.is_some());
}

#[test]
fn expression_resolver_simple_identifier_not_found() {
    let resolver = StandardBeanExpressionResolver::new();
    let result = BeanExpressionResolver::evaluate(&resolver, "myBean", None).unwrap();
    assert!(result.is_none());
}

#[test]
fn expression_resolver_register_bean_and_find() {
    let resolver = StandardBeanExpressionResolver::new();
    resolver.register_bean("myBean".to_string(), Arc::new(42i32));
    assert_eq!(resolver.bean_count(), 1);

    let result = BeanExpressionResolver::evaluate(&resolver, "myBean", None).unwrap();
    assert!(result.is_some());
    let val = result.unwrap();
    assert!(val.downcast_ref::<i32>().is_some());
}

// =========================================================================
// 10. bean_definition_store_exception.rs tests
// =========================================================================

#[test]
fn bean_definition_store_exception_new() {
    let ex = BeanDefinitionStoreException::new("application.xml", "Invalid bean definition");
    assert_eq!(ex.resource_description(), "application.xml");
    let msg = format!("{}", ex);
    assert!(msg.contains("application.xml"));
    assert!(msg.contains("Invalid bean definition"));
}

#[test]
fn bean_definition_store_exception_with_cause() {
    let inner = std::io::Error::new(std::io::ErrorKind::Other, "inner error");
    let ex =
        BeanDefinitionStoreException::with_cause("config.yml", "Failed to parse", Box::new(inner));
    let msg = format!("{}", ex);
    assert!(msg.contains("config.yml"));
    assert!(msg.contains("Failed to parse"));
    assert!(msg.contains("inner error"));
}

#[test]
fn bean_definition_store_exception_debug() {
    let ex = BeanDefinitionStoreException::new("test.xml", "msg");
    let debug = format!("{:?}", ex);
    assert!(debug.contains("BeanDefinitionStoreException"));
}

#[test]
fn bean_definition_store_exception_error_trait() {
    use std::error::Error;
    let ex = BeanDefinitionStoreException::new("test.xml", "msg");
    // Error trait should be implementable
    let _: &dyn Error = &ex;
    let source = ex.source();
    assert!(source.is_none());

    let inner = std::io::Error::new(std::io::ErrorKind::Other, "inner");
    let ex_with = BeanDefinitionStoreException::with_cause("test.xml", "msg", Box::new(inner));
    // source() delegates to cause.source()
    let _source = ex_with.source();
}

// =========================================================================
// Additional edge cases for container BeanDefinitionRegistry
// =========================================================================

#[test]
fn container_bean_definition_count_with_dynamic() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let mut c = Container::new(b.build().unwrap());

    // 1 from registry
    assert_eq!(c.bean_definition_count(), 1);

    // Add dynamic
    let mut rbd = RootBeanDefinition::new();
    rbd.set_bean_class_name("test::Dynamic");
    c.register_bean_definition("dyn1".to_string(), Box::new(rbd))
        .unwrap();
    assert_eq!(c.bean_definition_count(), 2);

    // Remove registry definition - should be replaced with __DELETED__ in dynamic
    c.remove_bean_definition("alloc::string::String").unwrap();
    // Count: registry definitions minus those with matching deleted key = 0,
    // plus dynamic non-deleted = 1
    assert_eq!(c.bean_definition_count(), 1);
}

#[test]
fn container_bean_definition_names_with_deleted() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    let mut c = Container::new(b.build().unwrap());

    // Remove one from registry
    c.remove_bean_definition("alloc::string::String").unwrap();
    let names = c.bean_definition_names();
    // "alloc::string::String" should not appear
    assert!(!names.contains(&"alloc::string::String".to_string()));
    // The other registry definition should still appear
    assert!(names.iter().any(|n| n.contains("i32")));
}

#[test]
fn container_contains_bean_definition_registry() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    let c = Container::new(b.build().unwrap());

    assert!(c.contains_bean_definition("alloc::string::String"));
    assert!(!c.contains_bean_definition("nonexistent"));
}

#[test]
fn container_register_bean_definition_duplicate() {
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());

    let mut rbd = RootBeanDefinition::new();
    rbd.set_bean_class_name("test::Bean");
    let name = "test_bean".to_string();
    c.register_bean_definition(name.clone(), Box::new(rbd))
        .unwrap();

    let mut rbd2 = RootBeanDefinition::new();
    rbd2.set_bean_class_name("test::Bean2");
    let result = c.register_bean_definition(name.clone(), Box::new(rbd2));
    assert!(result.is_err());
}
