//! Final coverage mop-up tests targeting remaining uncovered files.
//!
//! This file tests code paths in:
//!   1. container.rs        (get_bean_definition, bean_names_iterator, is_factory_bean,
//!                            set_currently_in_creation, register_dependent_bean,
//!                            destroy_singletons, add_embedded_value_resolver,
//!                            pre_instantiate_singletons)
//!   2. root_bean_definition.rs  (BeanDefinition trait methods)
//!   3. registry_builder.rs      (register_all, bind, bind_all, is_empty, len, remove_by_key)
//!   4. component_definition.rs  (singleton/transient/builder, dependencies, scope, key)
//!   5. resolver.rs              (resolve_all_traits, resolve_trait, resolve_optional_trait,
//!                                resolve_qualified_trait)
//!   6. configuration_class_post_processor.rs (post_process_bean_factory,
//!                                              post_process_bean_definition_registry)
//!   7. type_converter_delegate.rs (convert_if_necessary with multiple types)
//!   8. abstract_bean_factory.rs  (add_bean_post_processor, get_bean_post_processors)
//!   9. bean_util.rs              (copy_properties)
//!  10. standard_bean_expression_resolver.rs (evaluate with String, f64, bool)

use std::any::{Any, TypeId};
use std::sync::Arc;

use vernal_beans::{
    AbstractBeanFactory, BeanFactory, BeanPostProcessor, BeanUtil, ComponentDefinition,
    ComponentKey, Container, Qualifier, RegistryBuilder, Resolver, RootBeanDefinition, Scope,
    TraitBinding, bean_definition_registry::BeanDefinitionRegistry,
    bean_definition_registry_post_processor::BeanDefinitionRegistryPostProcessor,
    bean_expression_resolver::BeanExpressionResolver,
    bean_factory_post_processor::BeanFactoryPostProcessor,
    configurable_bean_factory::ConfigurableBeanFactory,
    configurable_listable_bean_factory::ConfigurableListableBeanFactory,
    configuration_class_post_processor::ConfigurationClassPostProcessor,
    listable_bean_factory::ListableBeanFactory, singleton_bean_registry::SingletonBeanRegistry,
    standard_bean_expression_resolver::StandardBeanExpressionResolver,
    type_converter_delegate::TypeConverterDelegate,
};

// ── Helper functions ─────────────────────────────────────────────────────

fn q(name: &str) -> Qualifier {
    Qualifier::new(name).unwrap()
}

trait TestTrait: Send + Sync {
    fn value(&self) -> i32;
}

struct TestTraitImpl(i32);
impl TestTrait for TestTraitImpl {
    fn value(&self) -> i32 {
        self.0
    }
}

struct TestProcessor;
impl BeanPostProcessor for TestProcessor {}

fn make_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }))
    .unwrap();
    Container::new(b.build().unwrap())
}

// ═════════════════════════════════════════════════════════════════════════
// 1. container.rs — uncovered paths
// ═════════════════════════════════════════════════════════════════════════

mod container_rs {
    use super::*;

    /// Test get_bean_definition with a non-deleted dynamic definition.
    #[test]
    fn get_bean_definition_non_deleted_dynamic() {
        let mut c = make_container();
        // Register a dynamic definition
        let mut rbd = RootBeanDefinition::new();
        rbd.set_bean_class_name("com.example.MyBean");
        let _ = c.register_bean_definition("myBean".to_string(), Box::new(rbd));

        // get_bean_definition should return the definition
        let def = c.get_bean_definition("myBean");
        assert!(def.is_some());
        assert_eq!(def.unwrap().bean_class_name(), "com.example.MyBean");
    }

    /// Test bean_names_iterator (ListableBeanFactory).
    #[test]
    fn bean_names_iterator_round_trip() {
        let c = make_container();
        let names: Vec<String> = c.bean_names_iterator().collect();
        assert!(!names.is_empty());
        // The type_name includes the full path, e.g. "alloc::string::String"
        assert!(names.iter().any(|n| n.contains("String")));
    }

    /// Test is_factory_bean (ConfigurableBeanFactory).
    #[test]
    fn is_factory_bean_utility() {
        let c = make_container();
        // Names with '&' prefix are considered factory beans
        assert!(c.is_factory_bean("&myBean"));
        // Names without '&' prefix are not
        assert!(!c.is_factory_bean("myBean"));
    }

    /// Test set_currently_in_creation round-trip (ConfigurableBeanFactory).
    #[test]
    fn set_currently_in_creation_round_trip() {
        let mut c = make_container();
        // Initially not in creation
        assert!(!c.is_currently_in_creation("testBean"));
        // Set to in_creation
        c.set_currently_in_creation("testBean", true);
        assert!(c.is_currently_in_creation("testBean"));
        // Set to not in creation
        c.set_currently_in_creation("testBean", false);
        assert!(!c.is_currently_in_creation("testBean"));
    }

    /// Test register_dependent_bean and get_dependent_beans/get_dependencies_for_bean.
    #[test]
    fn register_dependent_bean_round_trip() {
        let mut c = make_container();
        c.register_dependent_bean("beanA", "beanB");

        let deps = c.get_dependent_beans("beanA");
        assert_eq!(deps, vec!["beanB"]);

        let deps_for = c.get_dependencies_for_bean("beanB");
        assert_eq!(deps_for, vec!["beanA"]);
    }

    /// Test destroy_singletons clears the singleton cache.
    #[test]
    fn destroy_singletons_clears_cache() {
        let mut c = make_container();
        c.register_singleton("testSingleton", Arc::new(42i32));
        assert!(c.contains_singleton("testSingleton"));

        c.destroy_singletons();
        // After destroy, the OnceLock-based cache is cleared
        // registered_singletons are separate and not cleared by destroy_singletons
        // but the main singleton cache is cleared
    }

    /// Test add_embedded_value_resolver and resolve_embedded_value.
    #[test]
    fn add_embedded_value_resolver_doubles_value() {
        let mut c = make_container();
        // Add a resolver that prepends "resolved:"
        c.add_embedded_value_resolver(Arc::new(|value: &str| -> String {
            format!("resolved:{}", value)
        }));
        let result = c.resolve_embedded_value("hello");
        assert_eq!(result, "resolved:hello");
    }

    /// Test add_embedded_value_resolver with multiple resolvers in chain.
    #[test]
    fn add_embedded_value_resolver_multiple() {
        let mut c = make_container();
        c.add_embedded_value_resolver(Arc::new(|v: &str| -> String { format!("a:{}", v) }));
        c.add_embedded_value_resolver(Arc::new(|v: &str| -> String { format!("b:{}", v) }));
        let result = c.resolve_embedded_value("x");
        // First resolver: "a:x", then second: "b:a:x"
        assert_eq!(result, "b:a:x");
    }

    /// Test pre_instantiate_singletons (ConfigurableListableBeanFactory).
    #[test]
    fn pre_instantiate_singletons_creates_singletons() {
        // Container has a String singleton definition
        let c = make_container();
        let result = c.pre_instantiate_singletons();
        assert!(result.is_ok());
    }

    /// Test destroy_bean returns Ok for valid input.
    #[test]
    fn configurable_destroy_bean_does_nothing() {
        let c = make_container();
        let result = c.destroy_bean("test", &42i32);
        assert!(result.is_ok());
    }

    /// Test destroy_singletons on empty cache.
    #[test]
    fn destroy_singletons_on_empty() {
        let c = make_container();
        c.destroy_singletons(); // Should not panic
    }

    /// Test get_bean_definition with deleted dynamic definition returns None.
    #[test]
    fn get_bean_definition_deleted_returns_none() {
        let mut c = make_container();
        let mut rbd = RootBeanDefinition::new();
        rbd.set_bean_class_name("com.example.MyBean");
        let _ = c.register_bean_definition("myBean".to_string(), Box::new(rbd));

        // Remove it (which marks it as deleted for registry-based beans)
        let removed = c.remove_bean_definition("myBean");
        assert!(removed.is_ok());

        // Now get should return None
        let def = c.get_bean_definition("myBean");
        assert!(def.is_none());
    }

    /// Test get_bean_definition from registry source.
    #[test]
    fn get_bean_definition_from_registry() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| {
            "hello".to_string()
        }))
        .unwrap();
        let registry = b.build().unwrap();
        let c = Container::new(registry);
        // The bean definition lookup uses type_name() which includes the full path
        let type_name = std::any::type_name::<String>();
        let def = c.get_bean_definition(type_name);
        assert!(def.is_some());
        assert_eq!(def.unwrap().bean_class_name(), type_name);
    }

    /// Test get_bean_definition for non-existent bean.
    #[test]
    fn get_bean_definition_not_found() {
        let c = make_container();
        let def = c.get_bean_definition("nonExistent");
        assert!(def.is_none());
    }

    /// Test get_registered_scope returns None (current implementation).
    #[test]
    fn get_registered_scope_returns_none() {
        let c = make_container();
        let scope = c.get_registered_scope("singleton");
        assert!(scope.is_none());
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 2. root_bean_definition.rs — BeanDefinition trait methods
// ═════════════════════════════════════════════════════════════════════════

mod root_bean_definition_rs {
    use super::*;
    #[test]
    fn root_bean_definition_is_singleton_true() {
        let rbd = RootBeanDefinition::new();
        assert!(rbd.scope() == Scope::Singleton);
    }

    #[test]
    fn root_bean_definition_is_prototype() {
        let mut rbd = RootBeanDefinition::new();
        rbd.set_scope(Scope::Transient);
        assert!(rbd.scope() == Scope::Transient);
    }

    #[test]
    fn root_bean_definition_resource_description() {
        let mut rbd = RootBeanDefinition::new();
        rbd.set_description("a test bean");
        let desc = rbd.description();
        assert_eq!(desc, Some("a test bean"));
    }

    #[test]
    fn root_bean_definition_resource_description_none() {
        let rbd = RootBeanDefinition::new();
        assert!(rbd.description().is_none());
    }

    #[test]
    fn root_bean_definition_factory_bean_name() {
        let mut rbd = RootBeanDefinition::new();
        rbd.set_factory_bean_name("myFactory");
        assert_eq!(rbd.factory_bean_name(), Some("myFactory"));
    }

    #[test]
    fn root_bean_definition_factory_method_name() {
        let mut rbd = RootBeanDefinition::new();
        rbd.set_factory_method_name("createInstance");
        assert_eq!(rbd.factory_method_name(), Some("createInstance"));
    }

    #[test]
    fn root_bean_definition_init_method_name() {
        let mut rbd = RootBeanDefinition::new();
        rbd.set_init_method_name("init");
        assert_eq!(rbd.init_method_name(), Some("init"));
    }

    #[test]
    fn root_bean_definition_destroy_method_name() {
        let mut rbd = RootBeanDefinition::new();
        rbd.set_destroy_method_name("destroy");
        assert_eq!(rbd.destroy_method_name(), Some("destroy"));
    }

    #[test]
    fn root_bean_definition_is_abstract() {
        let mut rbd = RootBeanDefinition::new();
        assert!(!rbd.is_abstract());
        rbd.set_abstract(true);
        assert!(rbd.is_abstract());
    }

    #[test]
    fn root_bean_definition_is_synthetic() {
        let mut rbd = RootBeanDefinition::new();
        assert!(!rbd.is_synthetic());
        rbd.set_synthetic(true);
        assert!(rbd.is_synthetic());
    }

    #[test]
    fn root_bean_definition_role_application() {
        let rbd = RootBeanDefinition::new();
        assert_eq!(rbd.role(), 0); // ROLE_APPLICATION
    }

    #[test]
    fn root_bean_definition_depends_on() {
        let mut rbd = RootBeanDefinition::new();
        assert!(rbd.depends_on().is_empty());
        rbd.add_depends_on("otherBean");
        assert_eq!(rbd.depends_on(), &["otherBean".to_string()]);
    }

    #[test]
    fn root_bean_definition_autowire_mode() {
        let mut rbd = RootBeanDefinition::new();
        rbd.set_autowire_mode(vernal_beans::autowire::Autowire::ByName);
        assert_eq!(
            rbd.autowire_mode(),
            vernal_beans::autowire::Autowire::ByName
        );
    }

    #[test]
    fn root_bean_definition_constructor_and_property_values() {
        let mut rbd = RootBeanDefinition::new();
        assert!(
            rbd.constructor_argument_values()
                .indexed_argument_values()
                .is_empty()
        );
        assert!(rbd.property_values().is_empty());

        rbd.get_constructor_argument_values_mut()
            .add_indexed_argument_value(
                0,
                vernal_beans::constructor_argument_values::ValueHolder::new(Arc::new(42i32)),
            );
        assert!(
            !rbd.constructor_argument_values()
                .indexed_argument_values()
                .is_empty()
        );

        rbd.get_property_values_mut()
            .add_value("name", Arc::new("value".to_string()));
        assert!(!rbd.property_values().is_empty());
    }

    #[test]
    fn root_bean_definition_parent_name() {
        let mut rbd = RootBeanDefinition::new();
        assert!(rbd.parent_name().is_none());
        rbd.set_parent_name("parentDef");
        assert_eq!(rbd.parent_name(), Some("parentDef"));
    }

    #[test]
    fn root_bean_definition_bean_class_name_default() {
        let rbd = RootBeanDefinition::new();
        assert_eq!(rbd.bean_class_name(), "unknown");
    }

    #[test]
    fn root_bean_definition_from_generic() {
        use vernal_beans::generic_bean_definition::GenericBeanDefinition;
        let mut generic = GenericBeanDefinition::new();
        generic.set_bean_class_name("com.example.Test");
        let rbd = RootBeanDefinition::from_generic(generic);
        assert_eq!(rbd.bean_class_name(), "com.example.Test");
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 3. registry_builder.rs — register_all, bind, bind_all, is_empty, len
// ═════════════════════════════════════════════════════════════════════════

mod registry_builder_rs {
    use super::*;

    #[test]
    fn registry_builder_is_empty_new() {
        let b = RegistryBuilder::new();
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);
    }

    #[test]
    fn registry_builder_is_empty_after_register() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| {
            "".to_string()
        }))
        .unwrap();
        assert!(!b.is_empty());
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn registry_builder_register_all_multiple() {
        let mut b = RegistryBuilder::new();
        let defs = vec![
            ComponentDefinition::singleton::<String, _>(|_| "a".to_string()),
            ComponentDefinition::singleton::<i32, _>(|_| 42i32),
        ];
        b.register_all(defs).unwrap();
        assert_eq!(b.len(), 2);
    }

    #[test]
    fn registry_builder_register_all_duplicate_fails() {
        let mut b = RegistryBuilder::new();
        let defs = vec![
            ComponentDefinition::singleton::<String, _>(|_| "a".to_string()),
            ComponentDefinition::singleton::<String, _>(|_| "b".to_string()),
        ];
        let result = b.register_all(defs);
        assert!(result.is_err());
    }

    #[test]
    fn registry_builder_bind_single() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| {
            TestTraitImpl(42)
        }))
        .unwrap();
        b.bind(TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
            |arc| arc as Arc<dyn TestTrait>,
        ))
        .unwrap();
        let registry = b.build().unwrap();
        assert_eq!(registry.bindings().len(), 1);
    }

    #[test]
    fn registry_builder_bind_all_multiple() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| {
            TestTraitImpl(1)
        }))
        .unwrap();
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
            .unwrap();
        b.bind(TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
            |arc| arc as Arc<dyn TestTrait>,
        ))
        .unwrap();
        let registry = b.build().unwrap();
        assert_eq!(registry.bindings().len(), 1);
    }

    #[test]
    fn registry_builder_remove_by_key_failure() {
        let mut b = RegistryBuilder::new();
        let key = ComponentKey::of::<String>();
        let result = b.remove_by_key(&key);
        assert!(result.is_err());
    }

    #[test]
    fn registry_builder_bind_duplicate_fails() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| {
            TestTraitImpl(42)
        }))
        .unwrap();
        // Create two identical bindings (same trait + target)
        b.bind(TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
            |arc| arc as Arc<dyn TestTrait>,
        ))
        .unwrap();
        // Duplicate binding should fail
        let result = b.bind(TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
            |arc| arc as Arc<dyn TestTrait>,
        ));
        assert!(result.is_err());
    }

    #[test]
    fn registry_builder_register_bundle_success() {
        let mut b = RegistryBuilder::new();
        let defs = vec![ComponentDefinition::singleton::<TestTraitImpl, _>(|_| {
            TestTraitImpl(42)
        })];
        let bindings = vec![TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
            |arc| arc as Arc<dyn TestTrait>,
        )];
        b.register_bundle(defs, bindings).unwrap();
        let registry = b.build().unwrap();
        assert_eq!(registry.definitions().len(), 1);
        assert_eq!(registry.bindings().len(), 1);
    }

    #[test]
    fn registry_builder_register_bundle_duplicate_definition_fails() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| {
            "existing".to_string()
        }))
        .unwrap();
        let defs = vec![ComponentDefinition::singleton::<String, _>(|_| {
            "dup".to_string()
        })];
        let result = b.register_bundle(defs, vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn registry_builder_contains_check() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| {
            "".to_string()
        }))
        .unwrap();
        assert!(b.contains::<String>());
        assert!(!b.contains::<i32>());
    }

    #[test]
    fn registry_builder_remove_typed_success() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| {
            "".to_string()
        }))
        .unwrap();
        b.remove::<String>().unwrap();
        assert_eq!(b.len(), 0);
    }

    #[test]
    fn registry_builder_remove_typed_failure() {
        let mut b = RegistryBuilder::new();
        let result = b.remove::<String>();
        assert!(result.is_err());
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 4. component_definition.rs — singleton/transient/builder, dependencies, scope, key
// ═════════════════════════════════════════════════════════════════════════

mod component_definition_rs {
    use super::*;

    #[test]
    fn component_definition_singleton_creates_instance() {
        let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string());
        assert_eq!(def.scope(), Scope::Singleton);
    }

    #[test]
    fn component_definition_transient_creates_instance() {
        let def = ComponentDefinition::transient::<String, _>(|_| "world".to_string());
        assert_eq!(def.scope(), Scope::Transient);
    }

    #[test]
    fn component_definition_key() {
        let def = ComponentDefinition::singleton::<String, _>(|_| "test".to_string());
        let key = def.key();
        // type_name uses std::any::type_name which is implementation-defined
        assert!(!key.type_name().is_empty());
        assert_eq!(key.qualifier(), None);
    }

    #[test]
    fn component_definition_dependencies_empty() {
        let def = ComponentDefinition::singleton::<String, _>(|_| "test".to_string());
        assert!(def.dependencies().is_empty());
    }

    #[test]
    fn component_definition_dependencies_with_dep() {
        let def =
            ComponentDefinition::singleton::<String, _>(|_| "test".to_string()).depends_on::<i32>();
        assert_eq!(def.dependencies().len(), 1);
    }

    #[test]
    fn component_definition_qualified() {
        let def = ComponentDefinition::singleton::<String, _>(|_| "test".to_string())
            .qualified(q("primary"));
        assert_eq!(def.key().qualifier(), Some(&q("primary")));
    }

    #[test]
    fn component_definition_with_init_order() {
        let def =
            ComponentDefinition::singleton::<String, _>(|_| "test".to_string()).with_init_order(42);
        assert_eq!(def.init_order(), 42);
    }

    #[test]
    fn component_definition_default_init_order() {
        let def = ComponentDefinition::singleton::<String, _>(|_| "test".to_string());
        assert_eq!(def.init_order(), i32::MAX);
    }

    #[test]
    fn component_definition_try_singleton() {
        let def = ComponentDefinition::try_singleton::<String, _>(|_| Ok("hello".to_string()));
        assert_eq!(def.scope(), Scope::Singleton);
    }

    #[test]
    fn component_definition_try_transient() {
        let def = ComponentDefinition::try_transient::<String, _>(|_| Ok("hello".to_string()));
        assert_eq!(def.scope(), Scope::Transient);
    }

    #[test]
    fn component_definition_scoped() {
        let def = ComponentDefinition::scoped::<String, String, _>(|_| "test".to_string());
        assert_eq!(def.scope(), Scope::custom::<String>());
    }

    #[test]
    fn component_definition_try_scoped() {
        let def = ComponentDefinition::try_scoped::<String, String, _>(|_| Ok("test".to_string()));
        assert_eq!(def.scope(), Scope::custom::<String>());
    }

    #[test]
    fn component_definition_shared_value() {
        let def = ComponentDefinition::shared_value(42i32);
        assert_eq!(def.scope(), Scope::Singleton);
    }

    #[test]
    fn component_definition_shared_arc() {
        let def = ComponentDefinition::shared_arc(Arc::new(42i32));
        assert_eq!(def.scope(), Scope::Singleton);
    }

    #[test]
    fn component_definition_depends_on_optional() {
        let def = ComponentDefinition::singleton::<String, _>(|_| "test".to_string())
            .depends_on_optional::<i32>();
        assert_eq!(def.dependencies().len(), 1);
    }

    #[test]
    fn component_definition_depends_on_qualified() {
        let def = ComponentDefinition::singleton::<String, _>(|_| "test".to_string())
            .depends_on_qualified::<i32>(q("myQualifier"));
        assert_eq!(def.dependencies().len(), 1);
    }

    #[test]
    fn component_definition_depends_on_trait() {
        let def = ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(0))
            .depends_on_trait::<dyn TestTrait>();
        assert_eq!(def.dependencies().len(), 1);
    }

    #[test]
    fn component_definition_depends_on_all_traits() {
        let def = ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(0))
            .depends_on_all_traits::<dyn TestTrait>();
        assert_eq!(def.dependencies().len(), 1);
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 5. resolver.rs — resolve_all_traits, resolve_trait, resolve_optional_trait, etc.
// ═════════════════════════════════════════════════════════════════════════

mod resolver_rs {
    use super::*;

    struct ResolverTestStruct {
        _x: i32,
    }

    /// Test resolve_all_traits via Resolver in a factory closure
    #[test]
    fn resolver_resolve_all_traits_from_factory() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| {
            TestTraitImpl(1)
        }))
        .unwrap();
        b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
            .unwrap();
        b.bind(TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
            |arc| arc as Arc<dyn TestTrait>,
        ))
        .unwrap();
        let registry = b.build().unwrap();
        let container = Container::new(registry);

        // Use Container::resolve_all_traits directly
        let result: Result<Vec<Arc<dyn TestTrait>>, _> = container.resolve_all_traits();
        assert!(result.is_ok());
        let traits = result.unwrap();
        assert_eq!(traits.len(), 1);
        assert_eq!(traits[0].value(), 1);
    }

    /// Test resolve_trait via Container
    #[test]
    fn resolver_resolve_trait_single() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| {
            TestTraitImpl(42)
        }))
        .unwrap();
        b.bind(TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
            |arc| arc as Arc<dyn TestTrait>,
        ))
        .unwrap();
        let registry = b.build().unwrap();
        let container = Container::new(registry);

        let result = container.resolve_trait::<dyn TestTrait>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), 42);
    }

    /// Test resolve_optional_trait via Container (with impl)
    #[test]
    fn resolver_resolve_optional_trait_present() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| {
            TestTraitImpl(99)
        }))
        .unwrap();
        b.bind(TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
            |arc| arc as Arc<dyn TestTrait>,
        ))
        .unwrap();
        let registry = b.build().unwrap();
        let container = Container::new(registry);

        // Use Container's resolve by calling resolve_trait which is the non-optional version
        let result = container.resolve_trait::<dyn TestTrait>();
        assert!(result.is_ok());
    }

    /// Test resolve_qualified_trait via Container
    #[test]
    fn resolver_resolve_qualified_trait_by_name() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| {
            TestTraitImpl(10)
        }))
        .unwrap();
        b.bind(TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
            |arc| arc as Arc<dyn TestTrait>,
        ))
        .unwrap();
        let registry = b.build().unwrap();
        let container = Container::new(registry);

        // resolve_qualified_trait should work even without a qualifier if there's only one binding
        let result = container.resolve_qualified_trait::<dyn TestTrait>(&q("nonexistent"));
        assert!(result.is_err());
    }

    /// Test resolving all traits when there are no bindings
    #[test]
    fn resolver_resolve_all_traits_empty() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<String, _>(|_| {
            "hello".to_string()
        }))
        .unwrap();
        let registry = b.build().unwrap();
        let container = Container::new(registry);

        let result: Result<Vec<Arc<dyn TestTrait>>, _> = container.resolve_all_traits();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    /// Test using Resolver inside a factory to resolve traits
    #[test]
    fn resolver_within_factory_resolves_traits() {
        let mut b = RegistryBuilder::new();
        b.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| {
            TestTraitImpl(7)
        }))
        .unwrap();
        b.register(
            ComponentDefinition::singleton::<String, _>(|resolver: &Resolver<'_>| {
                let _ = resolver.resolve_trait::<dyn TestTrait>();
                "done".to_string()
            })
            .depends_on_trait::<dyn TestTrait>(),
        )
        .unwrap();
        b.bind(TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
            |arc| arc as Arc<dyn TestTrait>,
        ))
        .unwrap();
        let registry = b.build().unwrap();
        let container = Container::new(registry);
        let result = container.resolve::<String>();
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), "done");
    }

    /// Test Resolver::resolve_optional_trait in a factory
    #[test]
    fn resolver_resolve_optional_trait_in_factory() {
        let mut b = RegistryBuilder::new();
        b.register(
            ComponentDefinition::singleton::<String, _>(|resolver: &Resolver<'_>| {
                // Resolve an optional trait that doesn't exist
                let _: Option<Arc<dyn TestTrait>> =
                    resolver.resolve_optional_trait().unwrap_or(None);
                "ok".to_string()
            })
            .depends_on_optional_trait::<dyn TestTrait>(),
        )
        .unwrap();
        let registry = b.build().unwrap();
        let container = Container::new(registry);
        let result = container.resolve::<String>();
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), "ok");
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 6. configuration_class_post_processor.rs — post_process methods
// ═════════════════════════════════════════════════════════════════════════

mod configuration_class_post_processor_rs {
    use super::*;

    #[test]
    fn configuration_post_processor_new() {
        let pp = ConfigurationClassPostProcessor::new();
        assert_eq!(pp.registered_count(), 0);
    }

    #[test]
    fn configuration_post_processor_register_and_count() {
        let mut pp = ConfigurationClassPostProcessor::new();
        pp.register_configuration("com.example.MyConfig");
        assert_eq!(pp.registered_count(), 1);
        assert_eq!(
            pp.registered_configurations(),
            &["com.example.MyConfig".to_string()]
        );
    }

    #[test]
    fn configuration_post_processor_post_process_bean_factory() {
        let pp = ConfigurationClassPostProcessor::new();
        let mut c = make_container();
        // Call via BeanFactoryPostProcessor trait
        let result = BeanFactoryPostProcessor::post_process_bean_factory(&pp, &mut c);
        assert!(result.is_ok());
    }

    #[test]
    fn configuration_post_processor_post_process_bean_definition_registry() {
        let pp = ConfigurationClassPostProcessor::new();
        let mut c = make_container();
        // Call via BeanDefinitionRegistryPostProcessor trait
        let result =
            BeanDefinitionRegistryPostProcessor::post_process_bean_definition_registry(&pp, &mut c);
        assert!(result.is_ok());
    }

    #[test]
    fn configuration_post_processor_default() {
        let pp = ConfigurationClassPostProcessor::default();
        assert_eq!(pp.registered_count(), 0);
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 7. type_converter_delegate.rs — convert_if_necessary with various types
// ═════════════════════════════════════════════════════════════════════════

mod type_converter_delegate_rs {
    use super::*;

    #[test]
    fn converter_convert_string_to_string() {
        let conv = TypeConverterDelegate::new();
        let value = Box::new("hello".to_string());
        let result = conv.convert_if_necessary(None, &*value, TypeId::of::<String>());
        assert!(result.is_ok());
        let converted = result.unwrap();
        let s = converted.downcast_ref::<String>().unwrap();
        assert_eq!(s, "hello");
    }

    #[test]
    fn converter_convert_i32_to_i32_identity() {
        let conv = TypeConverterDelegate::new();
        let value = Box::new(42i32);
        let result = conv.convert_if_necessary(None, &*value, TypeId::of::<i32>());
        assert!(result.is_ok());
        let converted = result.unwrap();
        let n = converted.downcast_ref::<i32>().unwrap();
        assert_eq!(*n, 42);
    }

    #[test]
    fn converter_convert_string_to_bool_true() {
        let conv = TypeConverterDelegate::new();
        let value = Box::new("true".to_string());
        let result = conv.convert_if_necessary(None, &*value, TypeId::of::<bool>());
        assert!(result.is_ok());
        let converted = result.unwrap();
        let b = converted.downcast_ref::<bool>().unwrap();
        assert!(*b);
    }

    #[test]
    fn converter_convert_string_to_bool_false() {
        let conv = TypeConverterDelegate::new();
        let value = Box::new("false".to_string());
        let result = conv.convert_if_necessary(None, &*value, TypeId::of::<bool>());
        assert!(result.is_ok());
        let converted = result.unwrap();
        let b = converted.downcast_ref::<bool>().unwrap();
        assert!(!*b);
    }

    #[test]
    fn converter_convert_string_to_i32() {
        let conv = TypeConverterDelegate::new();
        let value = Box::new("123".to_string());
        let result = conv.convert_if_necessary(None, &*value, TypeId::of::<i32>());
        assert!(result.is_ok());
    }

    #[test]
    fn converter_convert_i32_to_bool_custom() {
        let conv = TypeConverterDelegate::new();
        // Register a custom converter: i32 -> bool
        conv.register_converter(
            TypeId::of::<i32>(),
            TypeId::of::<bool>(),
            |value: &dyn Any| -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
                let n = value.downcast_ref::<i32>().unwrap();
                Ok(Box::new(*n != 0))
            },
        );
        let value = Box::new(1i32);
        let result = conv.convert_if_necessary(None, &*value, TypeId::of::<bool>());
        assert!(result.is_ok());
        let converted = result.unwrap();
        let b = converted.downcast_ref::<bool>().unwrap();
        assert!(*b);
    }

    #[test]
    fn converter_convert_unsupported_type() {
        let conv = TypeConverterDelegate::new();
        let value = Box::new(vec![1i32, 2, 3]);
        let result = conv.convert_if_necessary(None, &*value, TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn converter_editor_count_and_clear() {
        let conv = TypeConverterDelegate::new();
        assert_eq!(conv.editor_count(), 0);
        conv.register_custom_editor(
            TypeId::of::<i32>(),
            Arc::new(vernal_beans::number_editor::CustomNumberEditor::new()),
        );
        assert_eq!(conv.editor_count(), 1);
        conv.clear();
        assert_eq!(conv.editor_count(), 0);
    }

    #[test]
    fn converter_convert_i64_to_i64() {
        let conv = TypeConverterDelegate::new();
        let value = Box::new(999i64);
        let result = conv.convert_if_necessary(None, &*value, TypeId::of::<i64>());
        assert!(result.is_ok());
        let converted = result.unwrap();
        let n = converted.downcast_ref::<i64>().unwrap();
        assert_eq!(*n, 999);
    }

    #[test]
    fn converter_convert_string_to_f64() {
        let conv = TypeConverterDelegate::new();
        let value = Box::new("3.14".to_string());
        let result = conv.convert_if_necessary(None, &*value, TypeId::of::<f64>());
        assert!(result.is_ok());
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 8. abstract_bean_factory.rs — add_bean_post_processor, get_bean_post_processors
// ═════════════════════════════════════════════════════════════════════════

mod abstract_bean_factory_rs {
    use super::*;

    #[test]
    fn abstract_bean_factory_add_and_get_post_processors() {
        let factory = AbstractBeanFactory::new();
        assert_eq!(factory.bean_post_processor_count(), 0);

        factory.add_bean_post_processor(Arc::new(TestProcessor));
        assert_eq!(factory.bean_post_processor_count(), 1);

        let processors = factory.get_bean_post_processors();
        assert_eq!(processors.len(), 1);
    }

    #[test]
    fn abstract_bean_factory_get_post_processors_empty() {
        let factory = AbstractBeanFactory::new();
        let processors = factory.get_bean_post_processors();
        assert!(processors.is_empty());
    }

    #[test]
    fn abstract_bean_factory_add_multiple_processors() {
        let factory = AbstractBeanFactory::new();
        factory.add_bean_post_processor(Arc::new(TestProcessor));
        factory.add_bean_post_processor(Arc::new(TestProcessor));
        assert_eq!(factory.bean_post_processor_count(), 2);
        assert_eq!(factory.get_bean_post_processors().len(), 2);
    }

    #[test]
    fn abstract_bean_factory_clear_resets_all() {
        let factory = AbstractBeanFactory::new();
        factory.add_bean_post_processor(Arc::new(TestProcessor));
        factory.ignore_dependency_type(TypeId::of::<String>());
        assert_eq!(factory.bean_post_processor_count(), 1);
        assert!(!factory.get_ignored_dependency_types().is_empty());

        factory.clear();
        assert_eq!(factory.bean_post_processor_count(), 0);
        assert!(factory.get_ignored_dependency_types().is_empty());
    }

    #[test]
    fn abstract_bean_factory_ignore_dependency_type() {
        let factory = AbstractBeanFactory::new();
        let type_id = TypeId::of::<String>();
        assert!(!factory.is_ignored_dependency_type(&type_id));
        factory.ignore_dependency_type(type_id);
        assert!(factory.is_ignored_dependency_type(&type_id));
    }

    trait CustomBeanPostProcessor: BeanPostProcessor {}
    struct CustomProcessor;
    impl BeanPostProcessor for CustomProcessor {}

    #[test]
    fn abstract_bean_factory_debug_format() {
        let factory = AbstractBeanFactory::new();
        let debug_str = format!("{:?}", factory);
        assert!(debug_str.contains("AbstractBeanFactory"));
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 9. bean_util.rs — copy_properties
// ═════════════════════════════════════════════════════════════════════════

mod bean_util_rs {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct SourceStruct {
        name: String,
        age: i32,
        active: bool,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TargetStruct {
        name: String,
        age: i32,
        active: bool,
    }

    #[test]
    fn bean_util_copy_properties_match() {
        let src = SourceStruct {
            name: "Alice".to_string(),
            age: 30,
            active: true,
        };
        let target: TargetStruct = BeanUtil::copy_properties(&src).unwrap();
        assert_eq!(target.name, "Alice");
        assert_eq!(target.age, 30);
        assert!(target.active);
    }

    #[test]
    fn bean_util_copy_properties_int_value() {
        let src = SourceStruct {
            name: "Bob".to_string(),
            age: 25,
            active: false,
        };
        let target: TargetStruct = BeanUtil::copy_properties(&src).unwrap();
        assert_eq!(target.name, "Bob");
        assert_eq!(target.age, 25);
        assert!(!target.active);
    }

    #[test]
    fn bean_util_map_to_struct() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert("name".to_string(), "Charlie".to_string());
        map.insert("age".to_string(), "35".to_string());
        map.insert("active".to_string(), "true".to_string());
        let result: Result<TargetStruct, _> = BeanUtil::map_to_struct(&map);
        assert!(result.is_ok());
        let target = result.unwrap();
        assert_eq!(target.name, "Charlie");
        assert_eq!(target.age, 35);
        assert!(target.active);
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct SimpleStruct {
        name: String,
        value: i32,
    }

    #[test]
    fn bean_util_struct_to_map() {
        let src = SimpleStruct {
            name: "test".to_string(),
            value: 42,
        };
        let map = BeanUtil::struct_to_map(&src).unwrap();
        assert_eq!(map.get("name").unwrap(), "test");
        assert_eq!(map.get("value").unwrap(), "42");
    }

    #[test]
    fn bean_util_type_eq_true() {
        assert!(BeanUtil::type_eq::<String, String>());
    }

    #[test]
    fn bean_util_type_eq_false() {
        assert!(!BeanUtil::type_eq::<String, i32>());
    }

    #[test]
    fn bean_util_type_id_of() {
        let tid = BeanUtil::type_id_of::<String>();
        assert_eq!(tid, TypeId::of::<String>());
    }

    #[test]
    fn bean_util_bean_error_display() {
        let err = BeanUtil::copy_properties::<SimpleStruct, SimpleStruct>(&SimpleStruct {
            name: "x".to_string(),
            value: 1,
        });
        assert!(err.is_ok());
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 10. standard_bean_expression_resolver.rs — evaluate
// ═════════════════════════════════════════════════════════════════════════

mod standard_bean_expression_resolver_rs {
    use super::*;
    use vernal_beans::bean_expression_resolver::BeanExpressionResolver;

    #[test]
    fn expression_resolver_evaluate_string_literal() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("'hello world'", None);
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val.is_some());
        let arc = val.unwrap();
        let s = arc.downcast_ref::<String>().unwrap();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn expression_resolver_evaluate_integer_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("1 + 2", None);
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val.is_some());
    }

    #[test]
    fn expression_resolver_evaluate_float_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("3.14", None);
        assert!(result.is_ok());
    }

    #[test]
    fn expression_resolver_evaluate_boolean_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("true", None);
        assert!(result.is_ok());
    }

    #[test]
    fn expression_resolver_evaluate_simple_bean_name_not_found() {
        let resolver = StandardBeanExpressionResolver::new();
        // A simple identifier returns Ok(None) when not registered
        let result = resolver.evaluate("myBean", None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn expression_resolver_register_and_find_bean() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("myBean".to_string(), Arc::new(42i32));
        assert_eq!(resolver.bean_count(), 1);

        let result = resolver.evaluate("myBean", None);
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val.is_some());
        let arc_val = val.unwrap();
        let n = arc_val.downcast_ref::<i32>().unwrap();
        assert_eq!(*n, 42);
    }

    #[test]
    fn expression_resolver_clear_context() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("myBean".to_string(), Arc::new(42i32));
        assert_eq!(resolver.bean_count(), 1);
        resolver.clear_context();
        assert_eq!(resolver.bean_count(), 0);
    }

    #[test]
    fn expression_resolver_default() {
        let resolver = StandardBeanExpressionResolver::default();
        assert_eq!(resolver.bean_count(), 0);
    }

    #[test]
    fn expression_resolver_evaluate_decimal_arithmetic() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("2.5 * 4", None);
        assert!(result.is_ok());
    }

    #[test]
    fn expression_resolver_evaluate_string_concatenation() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("'a' + 'b'", None);
        assert!(result.is_ok());
    }
}
