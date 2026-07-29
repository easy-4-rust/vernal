//! Container 覆盖率提升测试 — 覆盖 container.rs 中所有剩余未覆盖代码路径。

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

struct RequestScope;

trait TestTrait: Send + Sync {
    fn value(&self) -> i32;
}

struct TestTraitImpl(i32);
impl TestTrait for TestTraitImpl {
    fn value(&self) -> i32 { self.0 }
}

#[test]
fn container_resolve_type_mismatch() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve::<i32>();
    assert!(result.is_err());
}

#[test]
fn container_resolve_trait_ambiguous_with_qualifier() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(1))).unwrap();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(2)).qualified(q("b"))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(|arc| arc as Arc<dyn TestTrait>)).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(|arc| arc as Arc<dyn TestTrait>).qualified(q("b"))).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_trait::<dyn TestTrait>();
    let _ = result;
}

#[test]
fn container_resolve_trait_binding_target() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(|arc| arc as Arc<dyn TestTrait>)).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_trait::<dyn TestTrait>();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), 42);
}

#[test]
fn container_resolve_circular_dependency() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()).depends_on::<i32>()).unwrap();
    builder.register(ComponentDefinition::singleton::<i32, _>(|_| 42).depends_on::<String>()).unwrap();
    let registry = builder.build();
    assert!(registry.is_err());
}

#[test]
fn container_get_bean_by_type_id_resolve_error() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32)).unwrap();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| panic!("factory failure")).depends_on::<i32>()).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = container.get_bean_by_type_id(TypeId::of::<String>());
    }));
    assert!(result.is_err());
}

#[test]
fn container_get_bean_provider_get_error() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let provider = container.get_bean_provider_by_type_id(TypeId::of::<Vec<String>>()).unwrap();
    let result = provider.get();
    assert!(result.is_err());
}

#[test]
fn container_autowire_bean_resolve_error() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32)).unwrap();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| panic!("dependency failed")).depends_on::<i32>()).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = container.autowire_bean(existing);
    }));
    assert!(result.is_err());
}

struct FailingBeforeInitProcessor;
impl vernal_beans::BeanPostProcessor for FailingBeforeInitProcessor {
    fn post_process_before_initialization(&self, _bean: Arc<dyn Any + Send + Sync>, _bean_name: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Err("before init failed".into())
    }
}

struct FailingAfterInitProcessor;
impl vernal_beans::BeanPostProcessor for FailingAfterInitProcessor {
    fn post_process_after_initialization(&self, _bean: Arc<dyn Any + Send + Sync>, _bean_name: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
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

struct NoOpPostProcessor;
impl vernal_beans::BeanPostProcessor for NoOpPostProcessor {
    fn post_process_after_initialization(&self, _bean: Arc<dyn Any + Send + Sync>, _bean_name: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
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

#[test]
fn container_resolve_named_bean_resolve_error() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32)).unwrap();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| panic!("factory failure")).depends_on::<i32>()).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = container.resolve_named_bean(TypeId::of::<String>());
    }));
    assert!(result.is_err());
}

#[test]
fn container_resolve_dependency_resolve_error() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32)).unwrap();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| panic!("factory failure")).depends_on::<i32>()).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let descriptor = vernal_beans::DependencyDescriptor::for_field(TypeId::of::<String>(), "String");
        let _ = container.resolve_dependency(&descriptor, None);
    }));
    assert!(result.is_err());
}

#[test]
fn container_remove_bean_definition_already_deleted() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let _ = container.remove_bean_definition("myBean");
    let result = container.remove_bean_definition("myBean");
    assert!(result.is_err());
}

#[test]
fn container_get_bean_definition_deleted() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let _ = container.remove_bean_definition("myBean");
    let result = container.get_bean_definition("myBean");
    assert!(result.is_none());
}

#[test]
fn container_contains_bean_definition_deleted() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let _ = container.remove_bean_definition("myBean");
    let result = container.contains_bean_definition("myBean");
    assert!(!result);
}

#[test]
fn container_bean_definition_names_with_deleted() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    let _ = container.remove_bean_definition("myBean");
    let names = container.bean_definition_names();
    assert!(!names.contains(&"myBean".to_string()));
}

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

#[test]
fn container_transient_tracker() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let tracker = container.transient_tracker();
    assert_eq!(tracker.total_surviving(), 0);
}

#[test]
fn container_resolve_all_traits() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_all_traits::<dyn TestTrait>();
    let _ = result;
}

#[test]
fn container_unused_definitions() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let unused = container.unused_definitions();
    let _ = unused;
}

#[test]
fn container_warm_up() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.warm_up();
    assert!(result.is_ok());
}

#[test]
fn container_registry() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let reg = container.registry();
    assert_eq!(reg.definitions().len(), 1);
}

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

#[test]
fn container_resolve_trait_in() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(|arc| arc as Arc<dyn TestTrait>)).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_trait_in::<dyn TestTrait>(&scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), 42);
    scope.close();
}

#[test]
fn container_resolve_qualified_trait_in() {
    let mut builder = RegistryBuilder::new();
    let qual = q("my_trait");
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(|arc| arc as Arc<dyn TestTrait>).qualified(qual.clone())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_qualified_trait_in::<dyn TestTrait>(&qual, &scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), 42);
    scope.close();
}

#[test]
fn container_resolve_all_traits_in() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(|arc| arc as Arc<dyn TestTrait>)).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_all_traits_in::<dyn TestTrait>(&scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
    scope.close();
}
