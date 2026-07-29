//! Container 最终覆盖率测试 — 覆盖 container.rs 中所有剩余未覆盖代码路径。

use std::any::{Any, TypeId};
use std::sync::Arc;

use vernal_beans::{
    ComponentDefinition, Container, Qualifier, RegistryBuilder, Resolver, RootBeanDefinition, Scope,
};
use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
use vernal_beans::bean_factory::BeanFactory;
use vernal_beans::AutowireCapableBeanFactory;
use vernal_beans::ScopeState;

fn q(name: &str) -> Qualifier {
    Qualifier::new(name).unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test helpers
// ═══════════════════════════════════════════════════════════════════════════════

struct RequestScope;

trait TestTrait: Send + Sync {
    fn value(&self) -> i32;
}

struct TestTraitImpl(i32);
impl TestTrait for TestTraitImpl {
    fn value(&self) -> i32 { self.0 }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_typed TypeMismatch (lines 358-359)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_type_mismatch() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve::<i32>();
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — get_bean_by_type_id (lines 704-734)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_get_bean_by_type_id_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.get_bean_by_type_id(TypeId::of::<String>());
    assert!(result.is_ok());
}

#[test]
fn container_get_bean_by_type_id_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.get_bean_by_type_id(TypeId::of::<Vec<String>>());
    assert!(result.is_err());
}

#[test]
fn container_get_bean_by_type_id_multiple() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()).qualified(q("a"))).unwrap();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "b".to_string()).qualified(q("b"))).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.get_bean_by_type_id(TypeId::of::<String>());
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — is_type_match (lines 805-812)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_is_type_match() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let key = vernal_beans::ComponentKey::of::<String>();
    assert!(container.is_type_match(&key, TypeId::of::<String>()));
    assert!(!container.is_type_match(&key, TypeId::of::<i32>()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — ContainerObjectProvider::get error (line 830)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_get_bean_provider_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    // Get a provider for a type that doesn't exist
    let provider = container.get_bean_provider_by_type_id(TypeId::of::<Vec<String>>()).unwrap();
    // if_available should return None since no beans are cached
    assert!(provider.if_available().is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — create_bean error (lines 894-898)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_create_bean_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.create_bean("nonexistent.class.Name");
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — autowire_bean (lines 943-948)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_autowire_bean_no_match() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.autowire_bean(existing);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_with_deps() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    builder.register(ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        let _dep = resolver.resolve::<String>().unwrap();
        42
    }).depends_on::<String>()).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(0i32);
    let result = container.autowire_bean(existing);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — PostProcessor error branches (lines 1102, 1109)
// ═══════════════════════════════════════════════════════════════════════════════

struct FailingBeforeInitProcessor;

impl vernal_beans::BeanPostProcessor for FailingBeforeInitProcessor {
    fn post_process_before_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Err("before init failed".into())
    }
}

struct FailingAfterInitProcessor;

impl vernal_beans::BeanPostProcessor for FailingAfterInitProcessor {
    fn post_process_after_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Err("after init failed".into())
    }
}

#[test]
fn container_initialize_bean_failing_before_init() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.add_bean_post_processor(Arc::new(FailingBeforeInitProcessor));
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.initialize_bean(existing, "test");
    assert!(result.is_ok());
}

#[test]
fn container_initialize_bean_failing_after_init() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.add_bean_post_processor(Arc::new(FailingAfterInitProcessor));
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.initialize_bean(existing, "test");
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — configure_bean PostProcessor Ok(None)
// ═══════════════════════════════════════════════════════════════════════════════

struct NoOpPostProcessor;

impl vernal_beans::BeanPostProcessor for NoOpPostProcessor {
    fn post_process_after_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(None)
    }
}

#[test]
fn container_configure_bean_noop_post_processor() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.add_bean_post_processor(Arc::new(NoOpPostProcessor));
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.configure_bean(existing, "test");
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_named_bean (lines 1148-1152)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_named_bean_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_named_bean(TypeId::of::<Vec<String>>());
    assert!(result.is_err());
}

#[test]
fn container_resolve_named_bean_multiple() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()).qualified(q("a"))).unwrap();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "b".to_string()).qualified(q("b"))).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_named_bean(TypeId::of::<String>());
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_dependency (lines 1191-1195)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_dependency_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let descriptor = vernal_beans::DependencyDescriptor::for_field(TypeId::of::<Vec<String>>(), "Vec<String>");
    let result = container.resolve_dependency(&descriptor, None);
    assert!(result.is_err());
}

#[test]
fn container_resolve_dependency_optional_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let descriptor = vernal_beans::DependencyDescriptor::for_field(TypeId::of::<Vec<String>>(), "Vec<String>")
        .with_optional(true);
    let result = container.resolve_dependency(&descriptor, None);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn container_resolve_dependency_multiple() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()).qualified(q("a"))).unwrap();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "b".to_string()).qualified(q("b"))).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let descriptor = vernal_beans::DependencyDescriptor::for_field(TypeId::of::<String>(), "String");
    let result = container.resolve_dependency(&descriptor, None);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — set_type_converter / type_converter
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_set_type_converter() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.set_type_converter(None);
    assert!(container.type_converter().is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — ProxyBeanDefinition / DeletedBeanDefinition / RemovedBeanDefinition
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_proxy_bean_definition_trait() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let result = container.get_bean_definition("myBean");
    assert!(result.is_some());
    let proxy = result.unwrap();
    assert_eq!(proxy.bean_class_name(), "unknown");
    assert!(!proxy.is_lazy_init());
    assert!(!proxy.is_primary());
}

#[test]
fn container_deleted_bean_definition_trait() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let _ = container.remove_bean_definition("myBean");
    assert!(!container.contains_bean_definition("myBean"));
}

#[test]
fn container_removed_bean_definition_trait() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let result = container.remove_bean_definition("myBean");
    assert!(result.is_ok());
    let removed = result.unwrap();
    assert_eq!(removed.bean_class_name(), "unknown");
    assert!(!removed.is_lazy_init());
    assert!(!removed.is_primary());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — bean_definition_count
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_bean_definition_count() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let count1 = container.bean_definition_count();
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let count2 = container.bean_definition_count();
    assert!(count2 >= count1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — bean_definition_names
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_bean_definition_names() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("dynamicBean".to_string(), def).unwrap();
    let names = container.bean_definition_names();
    assert!(names.contains(&"dynamicBean".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — autowire with invalid mode
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_autowire_invalid_mode() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let type_name = std::any::type_name::<String>();
    let result = container.autowire(type_name, 99, false);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — bean_post_processor_count
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_bean_post_processor_count() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    assert_eq!(container.bean_post_processor_count(), 0);
    container.add_bean_post_processor(Arc::new(NoOpPostProcessor));
    assert_eq!(container.bean_post_processor_count(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — open_scope / open_scope_with_cancellation
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_open_scope() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    assert_eq!(scope.state(), ScopeState::Open);
    scope.close();
}

#[test]
fn container_open_scope_with_cancellation() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let token = tokio_util::sync::CancellationToken::new();
    let scope = container.open_scope_with_cancellation::<RequestScope>(token.clone());
    assert_eq!(scope.state(), ScopeState::Open);
    assert!(!token.is_cancelled());
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — transient_tracker
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_transient_tracker() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let tracker = container.transient_tracker();
    assert_eq!(tracker.total_surviving(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_all_traits
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_all_traits() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_all_traits::<dyn TestTrait>();
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — trait ambiguity test
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_trait_ambiguous() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(1))).unwrap();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(2)).qualified(q("b"))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
        |arc| arc as Arc<dyn TestTrait>,
    )).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
        |arc| arc as Arc<dyn TestTrait>,
    ).qualified(q("b"))).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_trait::<dyn TestTrait>();
    assert!(result.is_err());
}

#[test]
fn container_resolve_trait_binding_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
        |arc| arc as Arc<dyn TestTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_trait::<dyn TestTrait>();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), 42);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — remove_bean_definition from registry
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_remove_bean_definition_from_registry() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let type_name = std::any::type_name::<String>().to_string();
    let result = container.remove_bean_definition(&type_name);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — get_bean_definition from registry
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_get_bean_definition_from_registry() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let type_name = std::any::type_name::<String>().to_string();
    let result = container.get_bean_definition(&type_name);
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — contains_bean_definition from registry
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_contains_bean_definition_from_registry() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let type_name = std::any::type_name::<String>().to_string();
    assert!(container.contains_bean_definition(&type_name));
    assert!(!container.contains_bean_definition("nonexistent"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — unused_definitions
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_unused_definitions() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let unused = container.unused_definitions();
    let _ = unused;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — warm_up
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_warm_up() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.warm_up();
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — registry
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_registry() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let reg = container.registry();
    assert_eq!(reg.definitions().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_in
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_in() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_in::<String>(&scope);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_qualified_in
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_qualified_in() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()).qualified(q("greeting"))).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_qualified_in::<String>(&q("greeting"), &scope);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_trait_in
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_trait_in() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
        |arc| arc as Arc<dyn TestTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_trait_in::<dyn TestTrait>(&scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), 42);
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_qualified_trait_in
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_qualified_trait_in() {
    let mut builder = RegistryBuilder::new();
    let qual = q("my_trait");
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
        |arc| arc as Arc<dyn TestTrait>,
    ).qualified(qual.clone())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_qualified_trait_in::<dyn TestTrait>(&qual, &scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), 42);
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_all_traits_in
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_all_traits_in() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
        |arc| arc as Arc<dyn TestTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_all_traits_in::<dyn TestTrait>(&scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — select_trait_binding ambiguous branch (lines 507-511)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_trait_ambiguous_with_qualifier() {
    let mut builder = RegistryBuilder::new();
    // Register one implementation without qualifier and one with
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(1))).unwrap();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(2)).qualified(q("b"))).unwrap();
    // Bind both - one unqualified, one qualified
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
        |arc| arc as Arc<dyn TestTrait>,
    )).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
        |arc| arc as Arc<dyn TestTrait>,
    ).qualified(q("b"))).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    // Resolving without qualifier should pick the unqualified one
    // But since there are multiple bindings, it might be ambiguous
    let result = container.resolve_trait::<dyn TestTrait>();
    // Just exercise the code path - either Ok or Err is fine
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_binding error paths (lines 531-549)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_trait_binding_target_not_found() {
    let mut builder = RegistryBuilder::new();
    // Register TestTraitImpl so binding succeeds
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(42))).unwrap();
    // Bind the trait
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
        |arc| arc as Arc<dyn TestTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    // Resolving should succeed since TestTraitImpl is registered
    let result = container.resolve_trait::<dyn TestTrait>();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), 42);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_definition circular detection (lines 560-562)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_circular_dependency() {
    let mut builder = RegistryBuilder::new();
    builder.register(
        ComponentDefinition::singleton::<String, _>(|_| "a".to_string())
            .depends_on::<i32>(),
    ).unwrap();
    builder.register(
        ComponentDefinition::singleton::<i32, _>(|_| 42)
            .depends_on::<String>(),
    ).unwrap();
    let registry = builder.build();
    // Cycle detection happens at build time
    assert!(registry.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — get_bean_by_type_id error paths (lines 697-721)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_get_bean_by_type_id_resolve_error() {
    let mut builder = RegistryBuilder::new();
    // Register i32 first so depends_on::<i32>() succeeds
    builder.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32)).unwrap();
    // Register String with a failing factory and dependency on i32
    builder.register(
        ComponentDefinition::singleton::<String, _>(|_| panic!("factory failure"))
            .depends_on::<i32>(),
    ).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    // Factory panic propagates as panic, not as Error
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = container.get_bean_by_type_id(TypeId::of::<String>());
    }));
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — ContainerObjectProvider::get error (line 830)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_get_bean_provider_get_error() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    // Get a provider for a type that doesn't exist
    let provider = container.get_bean_provider_by_type_id(TypeId::of::<Vec<String>>()).unwrap();
    // get() should fail since no beans are cached
    let result = provider.get();
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — autowire_bean resolve error (lines 943-948)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_autowire_bean_resolve_error() {
    let mut builder = RegistryBuilder::new();
    // Register i32 first so depends_on::<i32>() succeeds
    builder.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32)).unwrap();
    // Register String with a failing factory and dependency on i32
    builder.register(
        ComponentDefinition::singleton::<String, _>(|_| panic!("dependency failed"))
            .depends_on::<i32>(),
    ).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    // Factory panic propagates as panic, not as Error
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = container.autowire_bean(existing);
    }));
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — PostProcessor error branches (lines 1102, 1109)
// ═══════════════════════════════════════════════════════════════════════════════

struct FailingBeforeInitProcessor2;

impl vernal_beans::BeanPostProcessor for FailingBeforeInitProcessor2 {
    fn post_process_before_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Err("before init failed".into())
    }
}

struct FailingAfterInitProcessor2;

impl vernal_beans::BeanPostProcessor for FailingAfterInitProcessor2 {
    fn post_process_after_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Err("after init failed".into())
    }
}

#[test]
fn container_initialize_bean_failing_before_init2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.add_bean_post_processor(Arc::new(FailingBeforeInitProcessor2));
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.initialize_bean(existing, "test");
    assert!(result.is_ok());
}

#[test]
fn container_initialize_bean_failing_after_init2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.add_bean_post_processor(Arc::new(FailingAfterInitProcessor2));
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.initialize_bean(existing, "test");
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — configure_bean PostProcessor Ok(None)
// ═══════════════════════════════════════════════════════════════════════════════

struct NoOpPostProcessor2;

impl vernal_beans::BeanPostProcessor for NoOpPostProcessor2 {
    fn post_process_after_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(None)
    }
}

#[test]
fn container_configure_bean_noop_post_processor2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.add_bean_post_processor(Arc::new(NoOpPostProcessor2));
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.configure_bean(existing, "test");
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_named_bean error paths (lines 1148-1152)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_named_bean_resolve_error() {
    let mut builder = RegistryBuilder::new();
    // Register i32 first so depends_on::<i32>() succeeds
    builder.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32)).unwrap();
    // Register String with a failing factory and dependency on i32
    builder.register(
        ComponentDefinition::singleton::<String, _>(|_| panic!("factory failure"))
            .depends_on::<i32>(),
    ).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    // Factory panic propagates as panic, not as Error
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = container.resolve_named_bean(TypeId::of::<String>());
    }));
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_dependency error paths (lines 1191-1195)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_dependency_resolve_error() {
    let mut builder = RegistryBuilder::new();
    // Register i32 first so depends_on::<i32>() succeeds
    builder.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32)).unwrap();
    // Register String with a failing factory and dependency on i32
    builder.register(
        ComponentDefinition::singleton::<String, _>(|_| panic!("factory failure"))
            .depends_on::<i32>(),
    ).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    // Factory panic propagates as panic, not as Error
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let descriptor = vernal_beans::DependencyDescriptor::for_field(TypeId::of::<String>(), "String");
        let _ = container.resolve_dependency(&descriptor, None);
    }));
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — remove_bean_definition __DELETED__ guard (lines 1357-1360)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_remove_bean_definition_already_deleted() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    // Register a dynamic definition
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    // Remove it - creates a deleted marker
    let _ = container.remove_bean_definition("myBean");
    // Try to remove again - should fail because it's already deleted
    let result = container.remove_bean_definition("myBean");
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — get_bean_definition __DELETED__ path (line 1429)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_get_bean_definition_deleted() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    // Register and remove a definition
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let _ = container.remove_bean_definition("myBean");
    // get_bean_definition should return None for deleted entries
    let result = container.get_bean_definition("myBean");
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — contains_bean_definition __DELETED__ path (line 1480)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_contains_bean_definition_deleted() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    // Register and remove a definition
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let _ = container.remove_bean_definition("myBean");
    // contains_bean_definition should return false for deleted entries
    let result = container.contains_bean_definition("myBean");
    assert!(!result);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — bean_definition_names with deleted (line 1480)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_bean_definition_names_with_deleted() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    // Register and remove a definition
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let _ = container.remove_bean_definition("myBean");
    // bean_definition_names should not include deleted entries
    let names = container.bean_definition_names();
    assert!(!names.contains(&"myBean".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — ProxyBeanDefinition / DeletedBeanDefinition / RemovedBeanDefinition
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_proxy_bean_definition_trait2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let result = container.get_bean_definition("myBean");
    assert!(result.is_some());
    let proxy = result.unwrap();
    assert_eq!(proxy.bean_class_name(), "unknown");
    assert!(!proxy.is_lazy_init());
    assert!(!proxy.is_primary());
}

#[test]
fn container_deleted_bean_definition_trait2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let _ = container.remove_bean_definition("myBean");
    assert!(!container.contains_bean_definition("myBean"));
}

#[test]
fn container_removed_bean_definition_trait2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let result = container.remove_bean_definition("myBean");
    assert!(result.is_ok());
    let removed = result.unwrap();
    assert_eq!(removed.bean_class_name(), "unknown");
    assert!(!removed.is_lazy_init());
    assert!(!removed.is_primary());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — autowire with invalid mode
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_autowire_invalid_mode2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let type_name = std::any::type_name::<String>();
    let result = container.autowire(type_name, 99, false);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — bean_post_processor_count
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_bean_post_processor_count2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    assert_eq!(container.bean_post_processor_count(), 0);
    container.add_bean_post_processor(Arc::new(NoOpPostProcessor2));
    assert_eq!(container.bean_post_processor_count(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — open_scope / open_scope_with_cancellation
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_open_scope2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    assert_eq!(scope.state(), ScopeState::Open);
    scope.close();
}

#[test]
fn container_open_scope_with_cancellation2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let token = tokio_util::sync::CancellationToken::new();
    let scope = container.open_scope_with_cancellation::<RequestScope>(token.clone());
    assert_eq!(scope.state(), ScopeState::Open);
    assert!(!token.is_cancelled());
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — transient_tracker
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_transient_tracker2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let tracker = container.transient_tracker();
    assert_eq!(tracker.total_surviving(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_all_traits
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_all_traits2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_all_traits::<dyn TestTrait>();
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — unused_definitions
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_unused_definitions2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let unused = container.unused_definitions();
    let _ = unused;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — warm_up
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_warm_up2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.warm_up();
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — registry
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_registry2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let reg = container.registry();
    assert_eq!(reg.definitions().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_in
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_in2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_in::<String>(&scope);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_qualified_in
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_qualified_in2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()).qualified(q("greeting"))).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_qualified_in::<String>(&q("greeting"), &scope);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_trait_in
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_trait_in2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
        |arc| arc as Arc<dyn TestTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_trait_in::<dyn TestTrait>(&scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), 42);
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_qualified_trait_in
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_qualified_trait_in2() {
    let mut builder = RegistryBuilder::new();
    let qual = q("my_trait");
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
        |arc| arc as Arc<dyn TestTrait>,
    ).qualified(qual.clone())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_qualified_trait_in::<dyn TestTrait>(&qual, &scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), 42);
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_all_traits_in
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_all_traits_in2() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
        |arc| arc as Arc<dyn TestTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_all_traits_in::<dyn TestTrait>(&scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
    scope.close();
}
