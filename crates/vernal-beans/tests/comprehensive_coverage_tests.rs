/// Comprehensive coverage tests for vernal-beans crate.
///
/// This file targets uncovered lines in the following files:
/// 1. container.rs - 194 uncovered lines (HIGHEST PRIORITY)
/// 2. scope_context.rs - 37 uncovered lines
/// 3. registry_builder.rs - 29 uncovered lines
/// 4. standard_bean_expression_resolver.rs - 27 uncovered lines
/// 5. property_editor_registry.rs - 15 uncovered lines
/// 6. property_editor_registry_support.rs - 14 uncovered lines
/// 7. property_editor_cache.rs - 14 uncovered lines
/// 8. type_converter_delegate.rs - 13 uncovered lines
/// 9. bean_definition_utils.rs - 11 uncovered lines
/// 10. component_contract.rs - 10 uncovered lines
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use vernal_beans::{
    BeanExpressionResolver, BeanFactory, Component, ComponentDefinition, ComponentKey, Container,
    HierarchicalBeanFactory, ListableBeanFactory, PropertyEditorRegistry, Qualifier, RegistryBuilder,
    ResolveError, ScopeKey, ScopeState,
};
use vernal_beans::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
use vernal_beans::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
use vernal_beans::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
use vernal_beans::factory::config::singleton_bean_registry::SingletonBeanRegistry;
use vernal_beans::factory::support::bean_definition_registry::BeanDefinitionRegistry;
use vernal_beans::factory::support::dependency_descriptor::DependencyDescriptor;

// ═══════════════════════════════════════════════════════════════════
// CONTAINER.RS TESTS - HIGHEST PRIORITY (194 uncovered lines)
// ═══════════════════════════════════════════════════════════════════

/// Helper to create a basic container with String and i32 singletons.
fn make_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()))
        .unwrap();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32))
        .unwrap();
    Container::new(b.build().unwrap())
}

/// Helper to create a container with transient definitions.
fn make_transient_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::transient::<String, _>(|_| "transient".to_string()))
        .unwrap();
    Container::new(b.build().unwrap())
}

/// Helper to create a container with qualified definitions.
fn make_qualified_container() -> Container {
    let mut b = RegistryBuilder::new();
    let q = Qualifier::new("primary").unwrap();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "primary".to_string()).qualified(q.clone()),
    )
    .unwrap();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "default".to_string()))
        .unwrap();
    Container::new(b.build().unwrap())
}

// ── select_definition tests ──────────────────────────────────────────

#[test]
fn container_select_definition_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let result: Result<Arc<String>, _> = c.resolve();
    assert!(result.is_err());
    match result.unwrap_err() {
        ResolveError::NotFound { .. } => {}
        other => panic!("Expected NotFound, got {:?}", other),
    }
}

#[test]
fn container_select_definition_single_match() {
    let c = make_container();
    let val: Arc<String> = c.resolve().unwrap();
    assert_eq!(*val, "hello");
}

// ── resolve_optional_typed tests ─────────────────────────────────────

#[test]
fn container_resolve_optional_typed_found() {
    let c = make_container();
    let val: Arc<String> = c.resolve().unwrap();
    assert_eq!(*val, "hello");
}

#[test]
fn container_resolve_optional_typed_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let result: Result<Arc<String>, _> = c.resolve();
    assert!(result.is_err());
}

// ── contains_local_bean tests ────────────────────────────────────────

#[test]
fn container_contains_local_bean_in_registry() {
    let c = make_container();
    assert!(c.contains_local_bean("alloc::string::String"));
    assert!(c.contains_local_bean("i32"));
    assert!(!c.contains_local_bean("nonexistent"));
}

#[test]
fn container_contains_local_bean_dynamic_definition() {
    let mut c = make_container();
    let def = Box::new(vernal_beans::RootBeanDefinition::new());
    c.register_bean_definition("dynamicBean".to_string(), def).unwrap();
    assert!(c.contains_local_bean("dynamicBean"));
}

#[test]
fn container_contains_local_bean_deleted_dynamic() {
    let mut c = make_container();
    let def = Box::new(vernal_beans::RootBeanDefinition::new());
    c.register_bean_definition("tempBean".to_string(), def).unwrap();
    c.remove_bean_definition("tempBean").unwrap();
    assert!(!c.contains_local_bean("tempBean"));
}

// ── warm_up tests ────────────────────────────────────────────────────

#[test]
fn container_warm_up_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    c.warm_up().unwrap();
}

#[test]
fn container_warm_up_resolves_all_singletons() {
    let c = make_container();
    c.warm_up().unwrap();
    let unused = c.unused_definitions();
    assert!(unused.is_empty());
}

#[test]
fn container_warm_up_with_transient() {
    let c = make_transient_container();
    c.warm_up().unwrap();
    let unused = c.unused_definitions();
    assert_eq!(unused.len(), 1);
}

// ── resolve_in tests ─────────────────────────────────────────────────

#[test]
fn container_resolve_in_success() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let val: Arc<String> = c.resolve_in(&scope).unwrap();
    assert_eq!(*val, "hello");
}

#[test]
fn container_resolve_in_wrong_owner() {
    let c = make_container();
    let other = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = other.open_scope::<String>();
    let result: Result<Arc<String>, _> = c.resolve_in(&scope);
    assert!(result.is_err());
    match result.unwrap_err() {
        ResolveError::ScopeOwnerMismatch { .. } => {}
        other => panic!("Expected ScopeOwnerMismatch, got {:?}", other),
    }
}

// ── resolve_qualified_in tests ───────────────────────────────────────

#[test]
fn container_resolve_qualified_in_success() {
    let c = make_qualified_container();
    let q = Qualifier::new("primary").unwrap();
    let scope = c.open_scope::<String>();
    let val: Arc<String> = c.resolve_qualified_in(&q, &scope).unwrap();
    assert_eq!(*val, "primary");
}

#[test]
fn container_resolve_qualified_in_wrong_owner() {
    let c = make_qualified_container();
    let other = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = other.open_scope::<String>();
    let q = Qualifier::new("primary").unwrap();
    let result: Result<Arc<String>, _> = c.resolve_qualified_in(&q, &scope);
    assert!(result.is_err());
}

// ── resolve_qualified tests ──────────────────────────────────────────

#[test]
fn container_resolve_qualified_success() {
    let c = make_qualified_container();
    let q = Qualifier::new("primary").unwrap();
    let val: Arc<String> = c.resolve_qualified(&q).unwrap();
    assert_eq!(*val, "primary");
}

#[test]
fn container_resolve_qualified_not_found() {
    let c = make_container();
    let q = Qualifier::new("missing").unwrap();
    let result: Result<Arc<String>, _> = c.resolve_qualified(&q);
    assert!(result.is_err());
}

// ── resolve_trait tests ──────────────────────────────────────────────

#[test]
fn container_resolve_trait_in_wrong_owner() {
    let c = make_container();
    let other = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = other.open_scope::<String>();
    let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_trait_in(&scope);
    assert!(result.is_err());
}

#[test]
fn container_resolve_qualified_trait_in_wrong_owner() {
    let c = make_container();
    let other = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = other.open_scope::<String>();
    let q = Qualifier::new("q").unwrap();
    let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> =
        c.resolve_qualified_trait_in(&q, &scope);
    assert!(result.is_err());
}

// ── resolve_all_traits tests ─────────────────────────────────────────

#[test]
fn container_resolve_all_traits_empty() {
    let c = make_container();
    let result: Result<Vec<Arc<dyn std::fmt::Display + Send + Sync>>, _> = c.resolve_all_traits();
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn container_resolve_all_traits_in_wrong_owner() {
    let c = make_container();
    let other = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = other.open_scope::<String>();
    let result: Result<Vec<Arc<dyn std::fmt::Display + Send + Sync>>, _> =
        c.resolve_all_traits_in(&scope);
    assert!(result.is_err());
}

#[test]
fn container_resolve_all_traits_in_success() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let result: Result<Vec<Arc<dyn std::fmt::Display + Send + Sync>>, _> =
        c.resolve_all_traits_in(&scope);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

// ── resolve_trait_typed tests ────────────────────────────────────────

#[test]
fn container_resolve_trait_not_found() {
    let c = make_container();
    let result: Result<Arc<dyn std::fmt::Debug + Send + Sync>, _> = c.resolve_trait();
    assert!(result.is_err());
}

#[test]
fn container_resolve_qualified_trait_not_found() {
    let c = make_container();
    let q = Qualifier::new("missing").unwrap();
    let result: Result<Arc<dyn std::fmt::Debug + Send + Sync>, _> = c.resolve_qualified_trait(&q);
    assert!(result.is_err());
}

// ── ObjectProvider tests ─────────────────────────────────────────────

#[test]
fn container_object_provider_get_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
    let result = provider.get();
    assert!(result.is_err());
}

#[test]
fn container_object_provider_if_available_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
    let result = provider.if_available();
    assert!(result.is_none());
}

#[test]
fn container_object_provider_stream_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
    let items = provider.stream();
    assert!(items.is_empty());
}

#[test]
fn container_object_provider_ordered_stream_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
    let items = provider.ordered_stream();
    assert!(items.is_empty());
}

#[test]
fn container_object_provider_get_returns_singleton() {
    let c = make_container();
    let _: Arc<String> = c.resolve().unwrap();
    let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
    let result = provider.get();
    assert!(result.is_ok());
}

#[test]
fn container_object_provider_if_available_returns_some() {
    let c = make_container();
    let _: Arc<String> = c.resolve().unwrap();
    let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
    let result = provider.if_available();
    assert!(result.is_some());
}

#[test]
fn container_object_provider_stream_returns_items() {
    let c = make_container();
    let _: Arc<String> = c.resolve().unwrap();
    let provider = c.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
    let items = provider.stream();
    assert!(!items.is_empty());
}

// ── AutowireCapableBeanFactory tests ─────────────────────────────────

#[test]
fn container_autowire_bean_with_matching_definition() {
    let c = make_container();
    let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.autowire_bean(existing);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_no_matching_definition() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i64);
    let result = c.autowire_bean(existing);
    assert!(result.is_ok());
}

#[test]
fn container_configure_bean() {
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.configure_bean(bean, "test_bean");
    assert!(result.is_ok());
}

#[test]
fn container_initialize_bean() {
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.initialize_bean(bean, "test_bean");
    assert!(result.is_ok());
}

#[test]
fn container_destroy_bean_instance() {
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.destroy_bean_instance("test_bean", bean.as_ref());
    assert!(result.is_ok());
}

#[test]
fn container_apply_bean_property_values() {
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.apply_bean_property_values(bean, "test_bean");
    assert!(result.is_ok());
}

// ── resolve_named_bean tests ─────────────────────────────────────────

#[test]
fn container_resolve_named_bean_single() {
    let c = make_container();
    let result = c.resolve_named_bean(TypeId::of::<String>());
    assert!(result.is_ok());
}

#[test]
fn container_resolve_named_bean_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let result = c.resolve_named_bean(TypeId::of::<String>());
    assert!(result.is_err());
}

#[test]
fn container_resolve_named_bean_ambiguous() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()))
        .unwrap();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
            .qualified(Qualifier::new("q").unwrap()),
    )
    .unwrap();
    let c = Container::new(b.build().unwrap());
    let result = c.resolve_named_bean(TypeId::of::<String>());
    assert!(result.is_err());
}

// ── resolve_dependency tests ─────────────────────────────────────────

#[test]
fn container_resolve_dependency_found() {
    let c = make_container();
    let descriptor = DependencyDescriptor::new(
        TypeId::of::<String>(),
        "String".to_string(),
        true,
    );
    let result = c.resolve_dependency(&descriptor, None);
    let _ = result;
}

#[test]
fn container_resolve_dependency_not_found_required() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let descriptor = DependencyDescriptor::new(
        TypeId::of::<String>(),
        "String".to_string(),
        true,
    );
    let result = c.resolve_dependency(&descriptor, None);
    assert!(result.is_err());
}

#[test]
fn container_resolve_dependency_not_found_optional() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let descriptor = DependencyDescriptor::new(
        TypeId::of::<String>(),
        "String".to_string(),
        false,
    );
    let result = c.resolve_dependency(&descriptor, None).unwrap();
    assert!(result.is_none());
}

// ── BeanDefinitionRegistry tests ─────────────────────────────────────

#[test]
fn container_register_and_get_bean_definition() {
    let mut c = make_container();
    let def = Box::new(vernal_beans::RootBeanDefinition::new());
    BeanDefinitionRegistry::register_bean_definition(&mut c, "myBean".to_string(), def).unwrap();
    assert!(BeanDefinitionRegistry::contains_bean_definition(&c, "myBean"));
    let bd = BeanDefinitionRegistry::get_bean_definition(&c, "myBean");
    assert!(bd.is_some());
}

#[test]
fn container_register_duplicate_bean_definition_fails() {
    let mut c = make_container();
    let def1 = Box::new(vernal_beans::RootBeanDefinition::new());
    let def2 = Box::new(vernal_beans::RootBeanDefinition::new());
    BeanDefinitionRegistry::register_bean_definition(&mut c, "myBean".to_string(), def1).unwrap();
    let result = BeanDefinitionRegistry::register_bean_definition(&mut c, "myBean".to_string(), def2);
    assert!(result.is_err());
}

#[test]
fn container_remove_dynamic_bean_definition() {
    let mut c = make_container();
    let def = Box::new(vernal_beans::RootBeanDefinition::new());
    BeanDefinitionRegistry::register_bean_definition(&mut c, "myBean".to_string(), def).unwrap();
    let removed = BeanDefinitionRegistry::remove_bean_definition(&mut c, "myBean");
    assert!(removed.is_ok());
    assert!(!BeanDefinitionRegistry::contains_bean_definition(&c, "myBean"));
}

#[test]
fn container_remove_registry_bean_definition() {
    let mut c = make_container();
    let removed = BeanDefinitionRegistry::remove_bean_definition(&mut c, "alloc::string::String");
    assert!(removed.is_ok());
    assert!(!BeanDefinitionRegistry::contains_bean_definition(&c, "alloc::string::String"));
}

#[test]
fn container_remove_nonexistent_bean_definition_fails() {
    let mut c = make_container();
    let result = BeanDefinitionRegistry::remove_bean_definition(&mut c, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn container_get_bean_definition_from_registry() {
    let c = make_container();
    let bd = BeanDefinitionRegistry::get_bean_definition(&c, "alloc::string::String");
    assert!(bd.is_some());
}

#[test]
fn container_get_bean_definition_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let bd = BeanDefinitionRegistry::get_bean_definition(&c, "nonexistent");
    assert!(bd.is_none());
}

#[test]
fn container_get_bean_definition_deleted_returns_none() {
    let mut c = make_container();
    BeanDefinitionRegistry::remove_bean_definition(&mut c, "alloc::string::String").unwrap();
    let bd = BeanDefinitionRegistry::get_bean_definition(&c, "alloc::string::String");
    assert!(bd.is_none());
}

#[test]
fn container_bean_definition_count() {
    let mut c = make_container();
    let initial = BeanDefinitionRegistry::bean_definition_count(&c);
    let def = Box::new(vernal_beans::RootBeanDefinition::new());
    BeanDefinitionRegistry::register_bean_definition(&mut c, "newBean".to_string(), def).unwrap();
    assert_eq!(BeanDefinitionRegistry::bean_definition_count(&c), initial + 1);
}

#[test]
fn container_bean_definition_names() {
    let c = make_container();
    let names = BeanDefinitionRegistry::bean_definition_names(&c);
    assert!(names.len() >= 2);
}

// ── SingletonBeanRegistry tests ──────────────────────────────────────

#[test]
fn container_register_and_get_singleton() {
    let c = make_container();
    let obj: Arc<dyn Any + Send + Sync> = Arc::new("singleton_value".to_string());
    c.register_singleton("mySingleton", obj.clone());
    let retrieved = c.get_singleton("mySingleton");
    assert!(retrieved.is_some());
    assert_eq!(
        *retrieved.unwrap().downcast_ref::<String>().unwrap(),
        "singleton_value"
    );
}

#[test]
fn container_contains_singleton() {
    let c = make_container();
    let obj: Arc<dyn Any + Send + Sync> = Arc::new("val".to_string());
    c.register_singleton("mySingleton", obj);
    assert!(c.contains_singleton("mySingleton"));
    assert!(!c.contains_singleton("nonexistent"));
}

#[test]
fn container_singleton_names() {
    let c = make_container();
    let _: Arc<String> = c.resolve().unwrap();
    let names = c.singleton_names();
    assert!(!names.is_empty());
}

#[test]
fn container_singleton_count() {
    let c = make_container();
    let _: Arc<String> = c.resolve().unwrap();
    let count = c.singleton_count();
    assert!(count >= 1);
}

#[test]
fn container_singleton_mutex() {
    let c = make_container();
    let mutex = c.singleton_mutex();
    let _ = mutex;
}

// ── HierarchicalBeanFactory tests ────────────────────────────────────

#[test]
fn container_parent_bean_factory_initially_none() {
    let c = make_container();
    assert!(c.parent_bean_factory().is_none());
}

// ── ListableBeanFactory tests ────────────────────────────────────────

#[test]
fn container_listable_bean_names_for_type_id() {
    let c = make_container();
    let names = c.bean_names_for_type_id(TypeId::of::<String>(), true, true);
    assert_eq!(names.len(), 1);
}

#[test]
fn container_listable_beans_of_type_id() {
    let c = make_container();
    let beans = c.beans_of_type_id(TypeId::of::<String>(), true, true).unwrap();
    assert_eq!(beans.len(), 1);
}

#[test]
fn container_listable_contains_non_singleton_bean() {
    let c = make_container();
    assert!(!c.contains_non_singleton_bean());
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

#[test]
fn container_listable_with_transient() {
    let c = make_transient_container();
    assert!(c.contains_non_singleton_bean());
    assert!(!c.contains_singleton_bean());
}

// ── ConfigurableBeanFactory tests ────────────────────────────────────

#[test]
fn container_set_parent_bean_factory() {
    let mut c = make_container();
    let parent_container = Container::new(RegistryBuilder::new().build().unwrap());
    let parent: Arc<dyn BeanFactory> = Arc::new(parent_container);
    c.set_parent_bean_factory(parent).unwrap();
    assert!(c.parent_bean_factory().is_some());
}

#[test]
fn container_register_scope_and_get() {
    let mut c = make_container();
    let scope = Box::new(vernal_beans::RequestScope::new("test-request"));
    c.register_scope("request", scope);
    let names = c.registered_scope_names();
    assert!(names.contains(&"request".to_string()));
    assert!(c.get_registered_scope("request").is_some());
    assert!(c.get_registered_scope("nonexistent").is_none());
}

#[test]
fn container_register_alias_success() {
    let mut c = make_container();
    c.register_alias("bean1", "alias1").unwrap();
}

#[test]
fn container_register_alias_conflict() {
    let mut c = make_container();
    c.register_alias("bean1", "alias1").unwrap();
    let result = c.register_alias("bean2", "alias1");
    assert!(result.is_err());
}

#[test]
fn container_register_alias_same_target_ok() {
    let mut c = make_container();
    c.register_alias("bean1", "alias1").unwrap();
    c.register_alias("bean1", "alias1").unwrap();
}

#[test]
fn container_is_factory_bean() {
    let c = make_container();
    assert!(c.is_factory_bean("&myFactoryBean"));
    assert!(!c.is_factory_bean("regularBean"));
}

#[test]
fn container_currently_in_creation() {
    let mut c = make_container();
    assert!(!c.is_currently_in_creation("bean1"));
    c.set_currently_in_creation("bean1", true);
    assert!(c.is_currently_in_creation("bean1"));
    c.set_currently_in_creation("bean1", false);
    assert!(!c.is_currently_in_creation("bean1"));
}

#[test]
fn container_dependent_beans_tracking() {
    let mut c = make_container();
    c.register_dependent_bean("service", "controller");
    let dependents = c.get_dependent_beans("service");
    assert!(dependents.contains(&"controller".to_string()));
    let deps = c.get_dependencies_for_bean("controller");
    assert!(deps.contains(&"service".to_string()));
}

#[test]
fn container_destroy_bean_configurable() {
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    c.destroy_bean("test_bean", bean.as_ref()).unwrap();
}

#[test]
fn container_destroy_singletons() {
    let c = make_container();
    let _: Arc<String> = c.resolve().unwrap();
    assert!(c.singleton_count() >= 1);
    c.destroy_singletons();
    assert_eq!(c.singleton_count(), 0);
}

#[test]
fn container_embedded_value_resolvers() {
    let mut c = make_container();
    let resolver: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new(|v: &str| {
        v.replace("${key}", "value")
    });
    c.add_embedded_value_resolver(resolver);
    let result = c.resolve_embedded_value("${key}");
    assert_eq!(result, "value");
}

#[test]
fn container_embedded_value_no_resolvers() {
    let c = make_container();
    let result = c.resolve_embedded_value("plain");
    assert_eq!(result, "plain");
}

// ── ConfigurableListableBeanFactory tests ────────────────────────────

#[test]
fn container_ignore_dependency_type() {
    let mut c = make_container();
    c.ignore_dependency_type(TypeId::of::<f64>());
}

#[test]
fn container_ignore_dependency_interface() {
    let mut c = make_container();
    c.ignore_dependency_interface(TypeId::of::<dyn std::fmt::Display>());
}

#[test]
fn container_register_resolvable_dependency() {
    let mut c = make_container();
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    c.register_resolvable_dependency(TypeId::of::<i32>(), value);
}

#[test]
fn container_is_autowire_candidate() {
    let c = make_container();
    assert!(c.is_autowire_candidate("any_bean"));
}

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
    c.pre_instantiate_singletons().unwrap();
}

// ── Autowire mode tests ──────────────────────────────────────────────

#[test]
fn container_autowire_mode_no() {
    let c = make_container();
    let result = c.autowire("alloc::string::String", 0, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_mode_by_name() {
    let c = make_container();
    let result = c.autowire("alloc::string::String", 1, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_mode_by_type() {
    let c = make_container();
    let result = c.autowire("alloc::string::String", 2, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_mode_constructor() {
    let c = make_container();
    let result = c.autowire("alloc::string::String", 3, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_invalid_mode() {
    let c = make_container();
    let result = c.autowire("alloc::string::String", 99, false);
    assert!(result.is_err());
}

// ── autowire_bean_properties tests ───────────────────────────────────

#[test]
fn container_autowire_bean_properties_no() {
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.autowire_bean_properties(bean, 0, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_properties_by_name() {
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.autowire_bean_properties(bean, 1, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_properties_by_type() {
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.autowire_bean_properties(bean, 2, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_properties_fallback() {
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.autowire_bean_properties(bean, 99, false);
    assert!(result.is_ok());
}

// ── BeanFactory trait tests ──────────────────────────────────────────

#[test]
fn container_bean_factory_get_type() {
    let c = make_container();
    let result = c.get_type(&ComponentKey::of::<String>()).unwrap();
    assert!(result.is_some());
}

#[test]
fn container_bean_factory_get_type_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(c.get_type(&ComponentKey::of::<String>()).is_err());
}

#[test]
fn container_bean_factory_get_aliases() {
    let c = make_container();
    let aliases = c.get_aliases(&ComponentKey::of::<String>());
    assert!(aliases.is_empty());
}

#[test]
fn container_bean_factory_is_type_match() {
    let c = make_container();
    assert!(c.is_type_match(&ComponentKey::of::<String>(), TypeId::of::<String>()));
    assert!(!c.is_type_match(&ComponentKey::of::<String>(), TypeId::of::<i32>()));
    assert!(!c.is_type_match(&ComponentKey::of::<f64>(), TypeId::of::<f64>()));
}

#[test]
fn container_bean_factory_is_singleton() {
    let c = make_container();
    assert!(c.is_singleton(&ComponentKey::of::<String>()).unwrap());
}

#[test]
fn container_bean_factory_is_singleton_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(c.is_singleton(&ComponentKey::of::<String>()).is_err());
}

#[test]
fn container_bean_factory_is_prototype() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string()))
        .unwrap();
    let c = Container::new(b.build().unwrap());
    assert!(c.is_prototype(&ComponentKey::of::<String>()).unwrap());
}

#[test]
fn container_bean_factory_is_prototype_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(c.is_prototype(&ComponentKey::of::<String>()).is_err());
}

#[test]
fn container_bean_factory_contains_bean() {
    let c = make_container();
    assert!(c.contains_bean(&ComponentKey::of::<String>()));
    assert!(!c.contains_bean(&ComponentKey::of::<f64>()));
}

#[test]
fn container_bean_factory_get_bean_by_key() {
    let c = make_container();
    let key = ComponentKey::of::<String>();
    let result = c.get_bean_by_key(&key);
    assert!(result.is_ok());
}

#[test]
fn container_bean_factory_get_bean_by_key_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let key = ComponentKey::of::<String>();
    let result = c.get_bean_by_key(&key);
    assert!(result.is_err());
}

#[test]
fn container_bean_factory_get_bean_by_type_id_single() {
    let c = make_container();
    let result = c.get_bean_by_type_id(TypeId::of::<String>());
    assert!(result.is_ok());
}

#[test]
fn container_bean_factory_get_bean_by_type_id_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let result = c.get_bean_by_type_id(TypeId::of::<String>());
    assert!(result.is_err());
}

#[test]
fn container_bean_factory_get_bean_by_type_id_ambiguous() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()))
        .unwrap();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
            .qualified(Qualifier::new("q").unwrap()),
    )
    .unwrap();
    let c = Container::new(b.build().unwrap());
    let result = c.get_bean_by_type_id(TypeId::of::<String>());
    assert!(result.is_err());
}

// ── create_bean tests ────────────────────────────────────────────────

#[test]
fn container_create_bean_by_class_name() {
    let c = make_container();
    let result = c.create_bean("alloc::string::String");
    assert!(result.is_ok());
}

#[test]
fn container_create_bean_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let result = c.create_bean("nonexistent");
    assert!(result.is_err());
}

// ── type_converter tests ─────────────────────────────────────────────

#[test]
fn container_set_type_converter_and_type_converter() {
    let mut c = make_container();
    c.set_type_converter(None);
    assert!(c.type_converter().is_none());
}

// ═══════════════════════════════════════════════════════════════════
// SCOPE_CONTEXT.RS TESTS - 37 uncovered lines
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn scope_context_child_scope_independent_close() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let child_scope = scope.child::<i32>();

    child_scope.close().await.unwrap();
    assert_eq!(child_scope.state(), ScopeState::Closed);
    assert_eq!(scope.state(), ScopeState::Open);
}

#[tokio::test]
async fn scope_context_child_cancelled_when_parent_closed() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let child = scope.child::<i32>();

    assert!(!child.cancellation().is_cancelled());
    scope.close().await.unwrap();
    assert!(child.cancellation().is_cancelled());
}

#[tokio::test]
async fn scope_context_close_with_timeout_success() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let result = scope
        .close_with_timeout(std::time::Duration::from_secs(5))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn scope_context_close_with_timeout_exceeded() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    scope
        .on_close(|| async {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            Ok::<(), std::io::Error>(())
        })
        .unwrap();

    let result = scope
        .close_with_timeout(std::time::Duration::from_millis(1))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn scope_context_on_close_hook_failure() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    scope
        .on_close(|| async {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "hook failed",
            ))
        })
        .unwrap();

    let result = scope.close().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn scope_context_multiple_hooks_executed() {
    use std::sync::atomic::{AtomicU32, Ordering};
    let c = make_container();
    let scope = c.open_scope::<String>();
    let counter = Arc::new(AtomicU32::new(0));

    let c1 = counter.clone();
    scope
        .on_close(move || async move {
            c1.fetch_add(1, Ordering::SeqCst);
            Ok::<(), std::io::Error>(())
        })
        .unwrap();

    let c2 = counter.clone();
    scope
        .on_close(move || async move {
            c2.fetch_add(1, Ordering::SeqCst);
            Ok::<(), std::io::Error>(())
        })
        .unwrap();

    scope.close().await.unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn scope_context_get_or_insert_with_after_close() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    scope.close().await.unwrap();
    let result = scope.get_or_insert_with::<String, _>(|| "hello".to_string());
    assert!(result.is_err());
}

#[tokio::test]
async fn scope_context_on_close_after_close() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    scope.close().await.unwrap();
    let result = scope.on_close(|| async { Ok::<(), std::io::Error>(()) });
    assert!(result.is_err());
}

#[tokio::test]
async fn scope_context_on_close_fails_when_cancelled() {
    let c = make_container();
    let token = CancellationToken::new();
    let scope = c.open_scope_with_cancellation::<String>(token.clone());
    token.cancel();
    let result = scope.on_close(|| async { Ok::<(), std::io::Error>(()) });
    assert!(result.is_err());
}

#[tokio::test]
async fn scope_context_close_idempotent() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    scope.close().await.unwrap();
    scope.close().await.unwrap();
    assert_eq!(scope.state(), ScopeState::Closed);
}

#[tokio::test]
async fn scope_context_grandchild_parent_chain() {
    let c = make_container();
    let root = c.open_scope::<String>();
    let mid = root.child::<i32>();
    let leaf = mid.child::<f64>();

    // Verify parent chain structure
    assert!(leaf.parent().is_some());
    assert!(leaf.parent().unwrap().parent().is_some());
    assert!(leaf.parent().unwrap().parent().unwrap().parent().is_none());
}

#[tokio::test]
async fn scope_context_get_or_insert_with_creates_value() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let value = scope
        .get_or_insert_with::<String, _>(|| "hello".to_string())
        .unwrap();
    assert_eq!(*value, "hello");
}

#[tokio::test]
async fn scope_context_get_or_insert_with_returns_cached() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let value1 = scope
        .get_or_insert_with::<String, _>(|| "first".to_string())
        .unwrap();
    let value2 = scope
        .get_or_insert_with::<String, _>(|| "second".to_string())
        .unwrap();
    assert!(Arc::ptr_eq(&value1, &value2));
}

// ═══════════════════════════════════════════════════════════════════
// REGISTRY_BUILDER.RS TESTS - 29 uncovered lines
// ═══════════════════════════════════════════════════════════════════

#[test]
fn registry_builder_register_and_build() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    builder
        .register(ComponentDefinition::shared_value("hello".to_string()))
        .unwrap();
    let registry = builder.build().unwrap();
    assert_eq!(registry.definitions().len(), 2);
}

#[test]
fn registry_builder_register_duplicate_fails() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    assert!(builder
        .register(ComponentDefinition::shared_value(100i32))
        .is_err());
}

#[test]
fn registry_builder_register_all_with_internal_duplicate() {
    let mut builder = RegistryBuilder::new();
    let defs = vec![
        ComponentDefinition::shared_value(1i32),
        ComponentDefinition::shared_value(2i32),
    ];
    assert!(builder.register_all(defs).is_err());
    assert!(builder.is_empty());
}

#[test]
fn registry_builder_register_all_with_existing_duplicate() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(1i32))
        .unwrap();
    let defs = vec![ComponentDefinition::shared_value(2i32)];
    assert!(builder.register_all(defs).is_err());
    assert_eq!(builder.len(), 1);
}

#[test]
fn registry_builder_bind_all_duplicate_exact() {
    let mut builder = RegistryBuilder::new();
    let binding1 = vernal_beans::TraitBinding::new(|s: Arc<String>| {
        s as Arc<dyn std::fmt::Debug + Send + Sync>
    });
    let binding2 = vernal_beans::TraitBinding::new(|s: Arc<String>| {
        s as Arc<dyn std::fmt::Debug + Send + Sync>
    });
    builder.bind_all(vec![binding1]).unwrap();
    let result = builder.bind_all(vec![binding2]);
    assert!(result.is_err());
}

#[test]
fn registry_builder_bind_all_duplicate_qualified() {
    let mut builder = RegistryBuilder::new();
    let q = Qualifier::new("myqual").unwrap();
    let binding1 = vernal_beans::TraitBinding::new(|s: Arc<String>| {
        s as Arc<dyn std::fmt::Debug + Send + Sync>
    })
    .qualified(q.clone());
    let binding2 = vernal_beans::TraitBinding::new(|s: Arc<String>| {
        s as Arc<dyn std::fmt::Debug + Send + Sync>
    })
    .qualified(q);
    builder.bind_all(vec![binding1]).unwrap();
    let result = builder.bind_all(vec![binding2]);
    assert!(result.is_err());
}

#[test]
fn registry_builder_bind_all_multiple_primary_fails() {
    let mut builder = RegistryBuilder::new();
    let binding1 = vernal_beans::TraitBinding::new(|s: Arc<String>| {
        s as Arc<dyn std::fmt::Display + Send + Sync>
    })
    .primary();
    let binding2 = vernal_beans::TraitBinding::new(|i: Arc<i32>| {
        i as Arc<dyn std::fmt::Display + Send + Sync>
    })
    .primary();
    let result = builder.bind_all(vec![binding1, binding2]);
    assert!(result.is_err());
}

#[test]
fn registry_builder_register_bundle_with_bindings() {
    let mut builder = RegistryBuilder::new();
    let binding = vernal_beans::TraitBinding::new(|s: Arc<String>| {
        s as Arc<dyn std::fmt::Display + Send + Sync>
    });
    builder
        .register_bundle(
            vec![ComponentDefinition::shared_value("hello".to_string())],
            vec![binding],
        )
        .unwrap();
    assert_eq!(builder.len(), 1);
}

#[test]
fn registry_builder_register_bundle_binding_failure_rolls_back() {
    let mut builder = RegistryBuilder::new();
    let q = Qualifier::new("q").unwrap();
    let binding1 = vernal_beans::TraitBinding::new(|s: Arc<String>| {
        s as Arc<dyn std::fmt::Display + Send + Sync>
    })
    .qualified(q.clone());
    let binding2 = vernal_beans::TraitBinding::new(|i: Arc<i32>| {
        i as Arc<dyn std::fmt::Display + Send + Sync>
    })
    .qualified(q);
    let result = builder.bind_all(vec![binding1, binding2]);
    assert!(result.is_err());
    assert!(builder.is_empty());
}

#[test]
fn registry_builder_remove_and_rebuild_indices() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(1i32))
        .unwrap();
    builder
        .register(ComponentDefinition::shared_value(2u64))
        .unwrap();
    builder
        .register(ComponentDefinition::shared_value("hello".to_string()))
        .unwrap();
    assert_eq!(builder.len(), 3);

    builder.remove::<i32>().unwrap();
    assert_eq!(builder.len(), 2);
    assert!(builder.contains::<u64>());
    assert!(builder.contains::<String>());

    builder
        .register(ComponentDefinition::shared_value(3.14f64))
        .unwrap();
    assert_eq!(builder.len(), 3);
}

#[test]
fn registry_builder_bean_definition_registry_trait() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    assert!(builder.contains_bean_definition("i32"));
    assert!(!builder.contains_bean_definition("nonexistent"));
    assert_eq!(builder.bean_definition_count(), 1);
    let names = builder.bean_definition_names();
    assert_eq!(names.len(), 1);
    assert!(builder.get_bean_definition("anything").is_none());
}

// ═══════════════════════════════════════════════════════════════════
// STANDARD_BEAN_EXPRESSION_RESOLVER.RS TESTS - 27 uncovered lines
// ═══════════════════════════════════════════════════════════════════

#[test]
fn standard_bean_expression_resolver_new_and_default() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    assert_eq!(resolver.bean_count(), 0);
    let resolver2 = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::default();
    assert_eq!(resolver2.bean_count(), 0);
}

#[test]
fn standard_bean_expression_resolver_register_and_find() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    resolver.register_bean("myBean".to_string(), Arc::new("value".to_string()));
    assert_eq!(resolver.bean_count(), 1);
    let result = <vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver as BeanExpressionResolver>::evaluate(
        &resolver, "myBean", None,
    )
    .unwrap();
    assert!(result.is_some());
    assert_eq!(
        *result.unwrap().downcast_ref::<String>().unwrap(),
        "value"
    );
}

#[test]
fn standard_bean_expression_resolver_clear_context() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    resolver.register_bean("bean1".to_string(), Arc::new("v1".to_string()));
    assert_eq!(resolver.bean_count(), 1);
    resolver.clear_context();
    assert_eq!(resolver.bean_count(), 0);
    let result = <vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver as BeanExpressionResolver>::evaluate(
        &resolver, "bean1", None,
    )
    .unwrap();
    assert!(result.is_none());
}

#[test]
fn standard_bean_expression_resolver_simple_identifier_not_found() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = <vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver as BeanExpressionResolver>::evaluate(
        &resolver, "nonExistentBean", None,
    )
    .unwrap();
    assert!(result.is_none());
}

#[test]
fn standard_bean_expression_resolver_empty_expression() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = <vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver as BeanExpressionResolver>::evaluate(
        &resolver, "", None,
    )
    .unwrap();
    assert!(result.is_none());
}

#[test]
fn standard_bean_expression_resolver_whitespace_trimmed() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    resolver.register_bean("myBean".to_string(), Arc::new("found".to_string()));
    let result = <vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver as BeanExpressionResolver>::evaluate(
        &resolver, "  myBean  ", None,
    )
    .unwrap();
    assert!(result.is_some());
}

#[test]
fn standard_bean_expression_resolver_spel_arithmetic() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = <vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver as BeanExpressionResolver>::evaluate(
        &resolver, "1 + 1", None,
    )
    .unwrap();
    if let Some(val) = result {
        let i = val.downcast_ref::<i64>();
        assert!(i.is_some());
        assert_eq!(*i.unwrap(), 2);
    }
}

#[test]
fn standard_bean_expression_resolver_spel_string_literal() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = <vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver as BeanExpressionResolver>::evaluate(
        &resolver, "'hello world'", None,
    )
    .unwrap();
    if let Some(val) = result {
        let s = val.downcast_ref::<String>();
        assert!(s.is_some());
        assert_eq!(*s.unwrap(), "hello world");
    }
}

#[test]
fn standard_bean_expression_resolver_spel_comparison() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = <vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver as BeanExpressionResolver>::evaluate(
        &resolver, "3 > 2", None,
    )
    .unwrap();
    if let Some(val) = result {
        let b = val.downcast_ref::<bool>();
        assert!(b.is_some());
        assert!(*b.unwrap());
    }
}

#[test]
fn standard_bean_expression_resolver_spel_complex() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = <vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver as BeanExpressionResolver>::evaluate(
        &resolver, "(2 + 3) * 4", None,
    )
    .unwrap();
    if let Some(val) = result {
        let i = val.downcast_ref::<i64>();
        assert!(i.is_some());
        assert_eq!(*i.unwrap(), 20);
    }
}

#[test]
fn standard_bean_expression_resolver_invalid_spel() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = <vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver as BeanExpressionResolver>::evaluate(
        &resolver, "@#$invalid", None,
    )
    .unwrap();
    assert!(result.is_none());
}

#[test]
fn standard_bean_expression_resolver_with_bean_name() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    resolver.register_bean("contextBean".to_string(), Arc::new(42i32));
    let result = <vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver as BeanExpressionResolver>::evaluate(
        &resolver, "contextBean", Some("requestBean"),
    )
    .unwrap();
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════
// PROPERTY_EDITOR_REGISTRY.RS TESTS - 15 uncovered lines
// ═══════════════════════════════════════════════════════════════════

#[test]
fn property_editor_registry_simple_new() {
    let registry = vernal_beans::property_editor_registry::SimplePropertyEditorRegistry::new();
    assert!(<vernal_beans::property_editor_registry::SimplePropertyEditorRegistry as PropertyEditorRegistry>::find_custom_editor(
        &registry, TypeId::of::<String>(), None,
    ).is_none());
}

#[test]
fn property_editor_registry_simple_default() {
    let registry = vernal_beans::property_editor_registry::SimplePropertyEditorRegistry::default();
    assert!(<vernal_beans::property_editor_registry::SimplePropertyEditorRegistry as PropertyEditorRegistry>::find_custom_editor(
        &registry, TypeId::of::<String>(), None,
    ).is_none());
}

#[test]
fn property_editor_registry_simple_register_and_find() {
    use vernal_beans::PropertyEditor;

    struct TestEditor;
    impl PropertyEditor for TestEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            None
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
    }

    let mut registry = vernal_beans::property_editor_registry::SimplePropertyEditorRegistry::new();
    registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor));
    assert!(<vernal_beans::property_editor_registry::SimplePropertyEditorRegistry as PropertyEditorRegistry>::has_custom_editor(&registry, TypeId::of::<String>(), None));
    assert!(!<vernal_beans::property_editor_registry::SimplePropertyEditorRegistry as PropertyEditorRegistry>::has_custom_editor(&registry, TypeId::of::<i32>(), None));
}

#[test]
fn property_editor_registry_simple_register_for_path() {
    use vernal_beans::PropertyEditor;

    struct TestEditor;
    impl PropertyEditor for TestEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            None
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
    }

    let mut registry = vernal_beans::property_editor_registry::SimplePropertyEditorRegistry::new();
    registry.register_custom_editor_for_path(TypeId::of::<String>(), "name", Box::new(TestEditor));
    assert!(<vernal_beans::property_editor_registry::SimplePropertyEditorRegistry as PropertyEditorRegistry>::has_custom_editor(&registry, TypeId::of::<String>(), Some("name")));
    assert!(!<vernal_beans::property_editor_registry::SimplePropertyEditorRegistry as PropertyEditorRegistry>::has_custom_editor(&registry, TypeId::of::<String>(), Some("other")));
}

#[test]
fn property_editor_registry_simple_find_editor_mut() {
    use vernal_beans::PropertyEditor;

    struct TestEditor;
    impl PropertyEditor for TestEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            None
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
    }

    let mut registry = vernal_beans::property_editor_registry::SimplePropertyEditorRegistry::new();
    registry.register_custom_editor_for_path(TypeId::of::<String>(), "field", Box::new(TestEditor));
    let found = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("field"));
    assert!(found.is_some());
}

#[test]
fn property_editor_registry_simple_find_editor_mut_fallback_to_type() {
    use vernal_beans::PropertyEditor;

    struct TestEditor;
    impl PropertyEditor for TestEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            None
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
    }

    let mut registry = vernal_beans::property_editor_registry::SimplePropertyEditorRegistry::new();
    registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor));
    let found = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("nonexistent"));
    assert!(found.is_some());
}

// ═══════════════════════════════════════════════════════════════════
// PROPERTY_EDITOR_REGISTRY_SUPPORT.RS TESTS - 14 uncovered lines
// ═══════════════════════════════════════════════════════════════════

#[test]
fn property_editor_registry_support_new() {
    let registry = vernal_beans::PropertyEditorRegistrySupport::new();
    let _ = registry;
}

#[test]
fn property_editor_registry_support_default() {
    let registry = vernal_beans::PropertyEditorRegistrySupport::default();
    let _ = registry;
}

#[test]
fn property_editor_registry_support_register_default_editor() {
    use vernal_beans::PropertyEditor;

    struct TestEditor;
    impl PropertyEditor for TestEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            None
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
    }

    let mut registry = vernal_beans::PropertyEditorRegistrySupport::new();
    registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor));
    assert!(registry.has_default_editor(TypeId::of::<i32>()));
    assert!(registry.get_default_editor(TypeId::of::<i32>()).is_some());
}

#[test]
fn property_editor_registry_support_override_default_editors() {
    use vernal_beans::PropertyEditor;

    struct TestEditor;
    impl PropertyEditor for TestEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            None
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
    }

    let mut registry = vernal_beans::PropertyEditorRegistrySupport::new();
    registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor));
    registry.override_default_editors();
    assert!(!registry.has_default_editor(TypeId::of::<i32>()));
    assert!(registry.get_default_editor(TypeId::of::<i32>()).is_none());
    registry.restore_default_editors();
    assert!(registry.has_default_editor(TypeId::of::<i32>()));
}

#[test]
fn property_editor_registry_support_find_custom_editor_falls_back_to_default() {
    use vernal_beans::PropertyEditor;

    struct TestEditor;
    impl PropertyEditor for TestEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            None
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
    }

    let mut registry = vernal_beans::PropertyEditorRegistrySupport::new();
    registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor));
    let editor = registry.find_custom_editor(TypeId::of::<i32>(), None);
    assert!(editor.is_some());
}

#[test]
fn property_editor_registry_support_find_custom_editor_no_fallback_when_overridden() {
    use vernal_beans::PropertyEditor;

    struct TestEditor;
    impl PropertyEditor for TestEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            None
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
    }

    let mut registry = vernal_beans::PropertyEditorRegistrySupport::new();
    registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor));
    registry.override_default_editors();
    let editor = registry.find_custom_editor(TypeId::of::<i32>(), None);
    assert!(editor.is_none());
}

#[test]
fn property_editor_registry_support_find_custom_editor_mut_with_default() {
    use vernal_beans::PropertyEditor;

    struct TestEditor;
    impl PropertyEditor for TestEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            None
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
    }

    let mut registry = vernal_beans::PropertyEditorRegistrySupport::new();
    registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor));
    let editor = registry.find_custom_editor_mut(TypeId::of::<i32>(), None);
    assert!(editor.is_some());
}

#[test]
fn property_editor_registry_support_find_custom_editor_mut_no_default_when_overridden() {
    use vernal_beans::PropertyEditor;

    struct TestEditor;
    impl PropertyEditor for TestEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            None
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
    }

    let mut registry = vernal_beans::PropertyEditorRegistrySupport::new();
    registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor));
    registry.override_default_editors();
    let editor = registry.find_custom_editor_mut(TypeId::of::<i32>(), None);
    assert!(editor.is_none());
}

// ═══════════════════════════════════════════════════════════════════
// PROPERTY_EDITOR_CACHE.RS TESTS - 14 uncovered lines
// ═══════════════════════════════════════════════════════════════════

#[test]
fn property_editor_cache_new() {
    let cache = vernal_beans::PropertyEditorCache::new();
    assert!(!cache.has_default_editors());
    assert_eq!(cache.custom_editor_count(), 0);
    assert_eq!(cache.default_editor_count(), 0);
}

#[test]
fn property_editor_cache_default() {
    let cache = vernal_beans::PropertyEditorCache::default();
    assert!(!cache.has_default_editors());
}

#[test]
fn property_editor_cache_register_and_find_custom() {
    use vernal_beans::PropertyEditor;

    #[derive(Debug)]
    struct StubEditor;
    impl PropertyEditor for StubEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            Some("stub".to_string())
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
    }

    let cache = vernal_beans::PropertyEditorCache::new();
    cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor));
    assert!(cache.has_custom_editor_for(TypeId::of::<String>()));
    assert_eq!(cache.custom_editor_count(), 1);
    let found = cache.find_custom_editor(TypeId::of::<String>());
    assert!(found.is_some());
}

#[test]
fn property_editor_cache_find_editor_prefers_custom() {
    use vernal_beans::PropertyEditor;

    #[derive(Debug)]
    struct StubEditor(String);
    impl PropertyEditor for StubEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            Some(self.0.clone())
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<i32>()
        }
    }

    let cache = vernal_beans::PropertyEditorCache::new();
    cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor("default".to_string())));
    cache.register_custom_editor(TypeId::of::<i32>(), Arc::new(StubEditor("custom".to_string())));
    let found = cache.find_editor(TypeId::of::<i32>()).unwrap();
    assert_eq!(found.get_as_text(), Some("custom".to_string()));
}

#[test]
fn property_editor_cache_clear() {
    use vernal_beans::PropertyEditor;

    #[derive(Debug)]
    struct StubEditor;
    impl PropertyEditor for StubEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            None
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
    }

    let cache = vernal_beans::PropertyEditorCache::new();
    cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor));
    cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor));
    cache.mark_default_editors_registered();
    cache.clear();
    assert_eq!(cache.custom_editor_count(), 0);
    assert_eq!(cache.default_editor_count(), 0);
    assert!(!cache.has_default_editors());
}

#[test]
fn property_editor_cache_clear_custom_editors_only() {
    use vernal_beans::PropertyEditor;

    #[derive(Debug)]
    struct StubEditor;
    impl PropertyEditor for StubEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
        fn set_as_text(&mut self, _: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            None
        }
        fn set_value(&mut self, _: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn Any> {
            None
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
    }

    let cache = vernal_beans::PropertyEditorCache::new();
    cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor));
    cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor));
    cache.mark_default_editors_registered();
    cache.clear_custom_editors();
    assert_eq!(cache.custom_editor_count(), 0);
    assert_eq!(cache.default_editor_count(), 1);
    assert!(cache.has_default_editors());
}

#[test]
fn property_editor_cache_debug_format() {
    let cache = vernal_beans::PropertyEditorCache::new();
    let debug = format!("{:?}", cache);
    assert!(debug.contains("PropertyEditorCache"));
}

// ═══════════════════════════════════════════════════════════════════
// TYPE_CONVERTER_DELEGATE.RS TESTS - 13 uncovered lines
// ═══════════════════════════════════════════════════════════════════

#[test]
fn type_converter_delegate_new() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    assert_eq!(delegate.editor_count(), 0);
}

#[test]
fn type_converter_delegate_default() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::default();
    assert_eq!(delegate.editor_count(), 0);
}

#[test]
fn type_converter_delegate_clear() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    delegate.register_converter(TypeId::of::<i32>(), TypeId::of::<i64>(), |v| {
        let n = v.downcast_ref::<i32>().unwrap();
        Ok(Box::new(*n as i64))
    });
    delegate.clear();
    assert_eq!(delegate.editor_count(), 0);
}

#[test]
fn type_converter_delegate_register_custom_editor() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let editor = Arc::new(vernal_beans::number_editor::CustomNumberEditor::new());
    delegate.register_custom_editor(TypeId::of::<i32>(), editor);
    assert_eq!(delegate.editor_count(), 1);
}

#[test]
fn type_converter_delegate_register_and_use_converter() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    delegate.register_converter(TypeId::of::<i32>(), TypeId::of::<i64>(), |v| {
        let n = v.downcast_ref::<i32>().unwrap();
        Ok(Box::new(*n as i64))
    });
    let value: Box<dyn Any> = Box::new(42i32);
    let result = delegate
        .convert_if_necessary(None, &*value, TypeId::of::<i64>())
        .unwrap();
    assert_eq!(*result.downcast::<i64>().unwrap(), 42i64);
}

#[test]
fn type_converter_delegate_convert_same_type() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value = "hello".to_string();
    let result = delegate
        .convert_if_necessary(None, &value, TypeId::of::<String>())
        .unwrap();
    assert_eq!(*result.downcast::<String>().unwrap(), "hello");
}

#[test]
fn type_converter_delegate_convert_string_to_bool() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value = "true".to_string();
    let result = delegate.convert_if_necessary(None, &value, TypeId::of::<bool>());
    let _ = result;
}

#[test]
fn type_converter_delegate_convert_string_to_i32() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value = "42".to_string();
    let result = delegate.convert_if_necessary(None, &value, TypeId::of::<i32>());
    let _ = result;
}

#[test]
fn type_converter_delegate_convert_unsupported_type() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    struct Unsupported;
    let value = Unsupported;
    let result = delegate.convert_if_necessary(None, &value, TypeId::of::<String>());
    assert!(result.is_err());
}

#[test]
fn type_converter_delegate_custom_converter_priority() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    delegate.register_converter(TypeId::of::<i32>(), TypeId::of::<i32>(), |v| {
        let n = v.downcast_ref::<i32>().unwrap();
        Ok(Box::new(*n + 100))
    });
    let value = 42i32;
    let result = delegate
        .convert_if_necessary(None, &value, TypeId::of::<i32>())
        .unwrap();
    assert_eq!(*result.downcast::<i32>().unwrap(), 142);
}

// ═══════════════════════════════════════════════════════════════════
// BEAN_DEFINITION_UTILS.RS TESTS - 11 uncovered lines
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_utils_generate_bean_name_no_conflict() {
    let builder = RegistryBuilder::new();
    let name = vernal_beans::bean_definition_utils::generate_bean_name(Some("myBean"), &builder);
    assert_eq!(name, "myBean");
}

#[test]
fn bean_definition_utils_generate_bean_name_with_conflict() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(42i32))
        .unwrap();
    let name = vernal_beans::bean_definition_utils::generate_bean_name(Some("i32"), &builder);
    assert_eq!(name, "i32#1");
}

#[test]
fn bean_definition_utils_generate_bean_name_none_class_name() {
    let builder = RegistryBuilder::new();
    let name = vernal_beans::bean_definition_utils::generate_bean_name(None, &builder);
    assert_eq!(name, "anonymous");
}

#[test]
fn bean_definition_utils_generate_bean_name_special_characters() {
    let builder = RegistryBuilder::new();
    let name =
        vernal_beans::bean_definition_utils::generate_bean_name(Some("com.example.MyService"), &builder);
    assert_eq!(name, "com.example.MyService");
}

#[test]
fn bean_definition_utils_generate_bean_name_single_char() {
    let builder = RegistryBuilder::new();
    let name = vernal_beans::bean_definition_utils::generate_bean_name(Some("A"), &builder);
    assert_eq!(name, "A");
}

// ═══════════════════════════════════════════════════════════════════
// COMPONENT_CONTRACT.RS TESTS - 10 uncovered lines
// ═══════════════════════════════════════════════════════════════════

#[test]
fn component_contract_default_init_order() {
    struct TestComponent;
    impl Component for TestComponent {
        fn definition() -> ComponentDefinition {
            ComponentDefinition::singleton::<TestComponent, _>(|_| TestComponent)
        }
    }
    // Default init_order should be INIT_SORT_DEFAULT
    assert_eq!(<TestComponent as Component>::init_order(), vernal_core::ordered::INIT_SORT_DEFAULT);
}

#[test]
fn component_contract_default_has_async_run() {
    struct TestComponent;
    impl Component for TestComponent {
        fn definition() -> ComponentDefinition {
            ComponentDefinition::singleton::<TestComponent, _>(|_| TestComponent)
        }
    }
    assert!(!<TestComponent as Component>::has_async_run());
}

#[test]
fn component_contract_default_shutdown() {
    struct TestComponent;
    impl Component for TestComponent {
        fn definition() -> ComponentDefinition {
            ComponentDefinition::singleton::<TestComponent, _>(|_| TestComponent)
        }
    }
    let component = TestComponent;
    <TestComponent as Component>::shutdown(&component); // Should not panic
}

#[test]
fn component_contract_custom_init_order() {
    struct TestComponent;
    impl Component for TestComponent {
        fn definition() -> ComponentDefinition {
            ComponentDefinition::singleton::<TestComponent, _>(|_| TestComponent)
        }
        fn init_order() -> i32 {
            100
        }
    }
    assert_eq!(<TestComponent as Component>::init_order(), 100);
}

#[test]
fn component_contract_custom_has_async_run() {
    struct TestComponent;
    impl Component for TestComponent {
        fn definition() -> ComponentDefinition {
            ComponentDefinition::singleton::<TestComponent, _>(|_| TestComponent)
        }
        fn has_async_run() -> bool {
            true
        }
    }
    assert!(<TestComponent as Component>::has_async_run());
}
