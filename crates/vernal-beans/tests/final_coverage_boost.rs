//! 最终覆盖率提升测试

use std::any::Any;
use std::sync::Arc;
use vernal_beans::{BeanFactory, ComponentDefinition, Container, Qualifier, RegistryBuilder, Resolver, Scope};

// ═══════════════════════════════════════════════════════════════════════════════
// Container 核心测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_new_and_resolve() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let result = container.resolve::<String>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
}

#[test]
fn container_resolve_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let result = container.resolve::<Vec<String>>();
    assert!(result.is_err());
}

#[test]
fn container_resolve_qualified() {
    let mut builder = RegistryBuilder::new();
    let q = Qualifier::new("primary").unwrap();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "qualified".to_string()).qualified(q.clone())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let result = container.resolve_qualified::<String>(&q);
    assert!(result.is_ok());
}

#[test]
fn container_resolve_trait() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let result = container.resolve_trait::<dyn Any + Send + Sync>();
    let _ = result;
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
}

#[test]
fn container_warm_up() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    assert!(container.warm_up().is_ok());
}

#[test]
fn bean_factory_get_bean_by_key() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let key = vernal_beans::ComponentKey::of::<String>();
    assert!(container.get_bean_by_key(&key).is_ok());
}

#[test]
fn bean_factory_contains_bean() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let key = vernal_beans::ComponentKey::of::<String>();
    assert!(container.contains_bean(&key));
}

#[test]
fn bean_factory_is_singleton() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let key = vernal_beans::ComponentKey::of::<String>();
    assert!(container.is_singleton(&key).unwrap());
}

#[test]
fn bean_factory_get_type() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let container = Container::new(builder.build().unwrap());
    let key = vernal_beans::ComponentKey::of::<String>();
    assert!(container.get_type(&key).unwrap().is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentDefinition 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_definition_all_variants() {
    let def1 = ComponentDefinition::singleton::<String, _>(|_| "test".to_string());
    assert_eq!(def1.scope(), Scope::Singleton);
    
    let def2 = ComponentDefinition::transient::<String, _>(|_| "test".to_string());
    assert_eq!(def2.scope(), Scope::Transient);
    
    let def3 = ComponentDefinition::try_singleton::<String, _>(|_| Ok("test".to_string()));
    assert_eq!(def3.scope(), Scope::Singleton);
    
    let def4 = ComponentDefinition::try_transient::<String, _>(|_| Ok("test".to_string()));
    assert_eq!(def4.scope(), Scope::Transient);
    
    let def5 = ComponentDefinition::scoped::<String, String, _>(|_| "test".to_string());
    assert!(matches!(def5.scope(), Scope::Custom(_)));
    
    let def6 = ComponentDefinition::try_scoped::<String, String, _>(|_| Ok("test".to_string()));
    assert!(matches!(def6.scope(), Scope::Custom(_)));
    
    let def7 = ComponentDefinition::shared_value(42i32);
    assert_eq!(def7.scope(), Scope::Singleton);
    
    let def8 = ComponentDefinition::shared_arc(Arc::new(42i32));
    assert_eq!(def8.scope(), Scope::Singleton);
}

#[test]
fn component_definition_qualified() {
    let q = Qualifier::new("primary").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_| "test".to_string()).qualified(q);
    assert!(def.key().qualifier().is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// ResolveError 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn resolve_error_all_variants() {
    let err1 = vernal_beans::ResolveError::NotFound { component: "test".to_string(), path: vec![] };
    assert!(!format!("{err1}").is_empty());
    
    let err2 = vernal_beans::ResolveError::Ambiguous { component: "test".to_string(), candidates: vec!["a".to_string()], path: vec![] };
    assert!(!format!("{err2}").is_empty());
    
    let err3 = vernal_beans::ResolveError::TypeMismatch { component: vernal_beans::ComponentKey::of::<String>() };
    assert!(!format!("{err3}").is_empty());
    
    let err4 = vernal_beans::ResolveError::UndeclaredDependency { component: vernal_beans::ComponentKey::of::<String>(), dependency: "test".to_string() };
    assert!(!format!("{err4}").is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dependency 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn dependency_all_methods() {
    let dep1 = vernal_beans::Dependency::of::<String>();
    assert_eq!(dep1.type_name(), std::any::type_name::<String>());
    
    let dep2 = vernal_beans::Dependency::optional_of::<String>();
    let _ = dep2;
    
    let dep3 = vernal_beans::Dependency::trait_of::<dyn Any + Send + Sync>();
    let _ = dep3;
    
    let q = Qualifier::new("test").unwrap();
    let dep4 = vernal_beans::Dependency::qualified::<String>(q);
    assert!(dep4.qualifier().is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentKey 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_key_all_methods() {
    let k1 = vernal_beans::ComponentKey::of::<String>();
    assert_eq!(k1.type_name(), std::any::type_name::<String>());
    let k2 = k1.clone();
    assert_eq!(k1, k2);
    let k3 = vernal_beans::ComponentKey::of::<i32>();
    assert_ne!(k1, k3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Qualifier 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn qualifier_all_methods() {
    let q1 = Qualifier::new("test").unwrap();
    assert_eq!(q1.as_str(), "test");
    let q2 = q1.clone();
    assert_eq!(q1, q2);
    assert!(Qualifier::new("").is_err());
    assert!(Qualifier::new(" bad ").is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeKey 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_key_all_methods() {
    let k1 = vernal_beans::ScopeKey::of::<String>();
    let k2 = k1.clone();
    assert_eq!(k1, k2);
    let k3 = vernal_beans::ScopeKey::of::<i32>();
    assert_ne!(k1, k3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeState 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_state_all() {
    let s1 = vernal_beans::ScopeState::Open;
    let s2 = s1.clone();
    assert_eq!(s1, s2);
    assert_eq!(vernal_beans::ScopeState::default(), vernal_beans::ScopeState::Open);
}

// ═══════════════════════════════════════════════════════════════════════════════
// GraphError 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn graph_error_all_variants() {
    let e1 = vernal_beans::GraphError::MissingDependency { path: vec!["a".to_string()] };
    assert!(!format!("{e1}").is_empty());
    
    let e2 = vernal_beans::GraphError::AmbiguousDependency { path: vec!["a".to_string()], candidates: vec!["c1".to_string()] };
    assert!(!format!("{e2}").is_empty());
    
    let e3 = vernal_beans::GraphError::MissingTraitBindingTarget { binding: "test".to_string() };
    assert!(!format!("{e3}").is_empty());
    
    let e4 = vernal_beans::GraphError::Cycle { path: vec!["a".to_string(), "b".to_string(), "a".to_string()] };
    assert!(!format!("{e4}").is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// GenericBeanDefinition 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn generic_bean_definition_from_root() {
    let mut root = vernal_beans::RootBeanDefinition::new();
    root.set_bean_class_name("FromRoot");
    root.set_scope(Scope::Transient);
    root.set_lazy_init(true);
    root.set_primary(true);
    root.set_description("root desc");
    root.set_autowire_mode(vernal_beans::Autowire::ByType);

    let gbd = vernal_beans::GenericBeanDefinition::from_root(&root);
    assert_eq!(gbd.get_bean_class_name(), Some("FromRoot"));
    assert_eq!(gbd.scope(), Scope::Transient);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TraitBinding 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn trait_binding_all() {
    let binding = vernal_beans::TraitBinding::new::<dyn Any + Send + Sync, String, _>(|arc| arc as Arc<dyn Any + Send + Sync>);
    let msg = format!("{}", binding);
    assert!(!msg.is_empty());
    let debug = format!("{:?}", binding);
    assert!(debug.contains("TraitBinding"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// InjectionPoint 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn injection_point_basic() {
    let ip = vernal_beans::InjectionPoint::new(std::any::TypeId::of::<String>(), std::any::type_name::<String>());
    let _ = ip;
}

// ═══════════════════════════════════════════════════════════════════════════════
// DependencyDescriptor 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn dependency_descriptor_all() {
    let dd1 = vernal_beans::DependencyDescriptor::for_field(std::any::TypeId::of::<String>(), std::any::type_name::<String>());
    let _ = dd1;
    
    let dd2 = vernal_beans::DependencyDescriptor::for_constructor_parameter(0, std::any::TypeId::of::<String>(), std::any::type_name::<String>());
    let _ = dd2;
    
    let dd3 = vernal_beans::DependencyDescriptor::for_method_parameter(0, std::any::TypeId::of::<String>(), std::any::type_name::<String>());
    let _ = dd3;
}

// ═══════════════════════════════════════════════════════════════════════════════
// TransientTracker 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn transient_tracker_new() {
    let _ = vernal_beans::TransientTracker::new();
}

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentSnapshot 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_snapshot_debug() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "test".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let snapshot = registry.snapshot();
    let debug = format!("{:?}", snapshot);
    assert!(!debug.is_empty());
}
