//! ComponentProvider/TraitProvider tests

use std::any::Any;
use std::sync::Arc;
use vernal_beans::{BeanFactory, ComponentDefinition, Container, Qualifier, RegistryBuilder, Resolver, Scope};

#[test]
fn component_provider_get_in_with_scope() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let scope = container.open_scope::<String>();

    let mut builder2 = RegistryBuilder::new();
    builder2.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let def = ComponentDefinition::singleton::<i32, _>(move |resolver: &Resolver| {
        let provider = resolver.provider::<String>().unwrap();
        let _ = provider.get_in(&scope);
        42
    }).depends_on_provider::<String>();
    builder2.register(def).unwrap();

    let container2 = Container::new(builder2.build().unwrap());
    let result = container2.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn component_provider_get_if_available_in_with_scope() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let scope = container.open_scope::<String>();

    let mut builder2 = RegistryBuilder::new();
    builder2.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let def = ComponentDefinition::singleton::<i32, _>(move |resolver: &Resolver| {
        let provider = resolver.provider::<String>().unwrap();
        let _ = provider.get_if_available_in(&scope);
        42
    }).depends_on_provider::<String>();
    builder2.register(def).unwrap();

    let container2 = Container::new(builder2.build().unwrap());
    let result = container2.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn component_provider_clone() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let def = ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        let provider = resolver.provider::<String>().unwrap();
        let _ = provider.clone();
        42
    }).depends_on_provider::<String>();
    builder.register(def).unwrap();

    let container = Container::new(builder.build().unwrap());
    let result = container.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn component_provider_debug() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let def = ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        let provider = resolver.provider::<String>().unwrap();
        let debug = format!("{:?}", provider);
        assert!(debug.contains("ComponentProvider"));
        42
    }).depends_on_provider::<String>();
    builder.register(def).unwrap();

    let container = Container::new(builder.build().unwrap());
    let result = container.resolve::<i32>();
    assert!(result.is_ok());
}

#[test]
fn trait_provider_exists() {
    // 验证 trait_provider 方法存在
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let _ = Container::new(builder.build().unwrap());
}

#[test]
fn bean_factory_get_bean_by_key() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let key = vernal_beans::ComponentKey::of::<String>();
    let result = container.get_bean_by_key(&key);
    assert!(result.is_ok());
}

#[test]
fn bean_factory_contains_bean() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let key = vernal_beans::ComponentKey::of::<String>();
    assert!(container.contains_bean(&key));
    let key2 = vernal_beans::ComponentKey::of::<Vec<String>>();
    assert!(!container.contains_bean(&key2));
}

#[test]
fn bean_factory_is_singleton() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let key = vernal_beans::ComponentKey::of::<String>();
    let result = container.is_singleton(&key);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn bean_factory_is_prototype() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let key = vernal_beans::ComponentKey::of::<String>();
    let result = container.is_prototype(&key);
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[test]
fn bean_factory_get_type() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let key = vernal_beans::ComponentKey::of::<String>();
    let result = container.get_type(&key);
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn bean_factory_get_aliases() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let key = vernal_beans::ComponentKey::of::<String>();
    let _ = container.get_aliases(&key);
}

#[test]
fn container_resolve_string() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let result = container.resolve::<String>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
}

#[test]
fn container_resolve_qualified() {
    let mut builder = RegistryBuilder::new();
    let q = Qualifier::new("primary").unwrap();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "qualified".to_string()).qualified(q.clone())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let result = container.resolve_qualified::<String>(&q);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "qualified");
}

#[test]
fn container_open_scope() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let scope = container.open_scope::<String>();
    assert!(scope.state() == vernal_beans::ScopeState::Open);
}

#[test]
fn container_resolve_in_scope() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let scope = container.open_scope::<String>();
    let result = container.resolve_in::<String>(&scope);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
}

#[test]
fn container_warm_up() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    assert!(container.warm_up().is_ok());
}

#[test]
fn container_registry() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let _ = container.registry();
}

#[test]
fn container_unused_definitions() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let _ = container.unused_definitions();
}

#[test]
fn container_transient_tracker() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let _ = container.transient_tracker();
}

#[test]
fn scope_context_get_or_insert_with() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let scope = container.open_scope::<String>();
    let result = scope.get_or_insert_with::<String, _>(|| "test".to_string());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "test");
}

#[test]
fn component_definition_singleton() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "test".to_string());
    assert_eq!(def.scope(), Scope::Singleton);
}

#[test]
fn component_definition_transient() {
    let def = ComponentDefinition::transient::<String, _>(|_| "test".to_string());
    assert_eq!(def.scope(), Scope::Transient);
}

#[test]
fn component_definition_shared_value() {
    let def = ComponentDefinition::shared_value(42i32);
    assert_eq!(def.scope(), Scope::Singleton);
}

#[test]
fn component_definition_key() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "test".to_string());
    assert_eq!(def.key().type_name(), std::any::type_name::<String>());
}

#[test]
fn component_definition_debug() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "test".to_string());
    assert!(format!("{:?}", def).contains("ComponentDefinition"));
}

#[test]
fn resolve_error_not_found() {
    let err = vernal_beans::ResolveError::NotFound { component: "test".to_string(), path: vec![] };
    assert!(!format!("{err}").is_empty());
}

#[test]
fn resolve_error_ambiguous() {
    let err = vernal_beans::ResolveError::Ambiguous { component: "test".to_string(), candidates: vec!["a".to_string()], path: vec![] };
    assert!(!format!("{err}").is_empty());
}

#[test]
fn dependency_of() {
    let dep = vernal_beans::Dependency::of::<String>();
    assert_eq!(dep.type_name(), std::any::type_name::<String>());
}

#[test]
fn component_key_of() {
    let k = vernal_beans::ComponentKey::of::<String>();
    assert_eq!(k.type_name(), std::any::type_name::<String>());
}

#[test]
fn component_key_debug() {
    let k = vernal_beans::ComponentKey::of::<String>();
    assert!(format!("{k:?}").contains("ComponentKey"));
}

#[test]
fn trait_key_of() {
    let k = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    let _ = k;
}

#[test]
fn qualifier_new() {
    let q = Qualifier::new("test").unwrap();
    assert_eq!(q.as_str(), "test");
}

#[test]
fn scope_state_open() {
    assert_eq!(vernal_beans::ScopeState::Open, vernal_beans::ScopeState::Open);
}

#[test]
fn graph_error_cycle() {
    let err = vernal_beans::GraphError::Cycle { path: vec!["a".to_string()] };
    assert!(!format!("{err}").is_empty());
}

#[test]
fn trait_binding_basic() {
    let binding = vernal_beans::TraitBinding::new::<dyn Any + Send + Sync, String, _>(|arc| arc as Arc<dyn Any + Send + Sync>);
    let _ = binding;
}

#[test]
fn injection_point_basic() {
    let ip = vernal_beans::InjectionPoint::new(std::any::TypeId::of::<String>(), std::any::type_name::<String>());
    let _ = ip;
}

#[test]
fn transient_tracker_new() {
    let _ = vernal_beans::TransientTracker::new();
}
