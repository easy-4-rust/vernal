//! Container 公共 API 覆盖测试 — 覆盖 bean_post_processor_count、open_scope、
//! resolve_in、resolve_qualified_in、resolve_trait_in、resolve_qualified_trait_in
//! 等未覆盖的公共方法。

use std::any::Any;
use std::sync::Arc;

use vernal_beans::{
    ComponentDefinition, Container, Qualifier, RegistryBuilder, Resolver, Scope, ScopeState,
};

fn q(name: &str) -> Qualifier {
    Qualifier::new(name).unwrap()
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
    container.add_bean_post_processor(Arc::new(TestPostProcessor));
    assert_eq!(container.bean_post_processor_count(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — open_scope / open_scope_with_cancellation
// ═══════════════════════════════════════════════════════════════════════════════

struct RequestScope;

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
    builder.register(
        ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
            .qualified(q("greeting")),
    ).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_qualified_in::<String>(&q("greeting"), &scope);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_trait / resolve_trait_in
// ═══════════════════════════════════════════════════════════════════════════════

trait MyTrait: Send + Sync {
    fn value(&self) -> i32;
}

struct MyTraitImpl(i32);
impl MyTrait for MyTraitImpl {
    fn value(&self) -> i32 { self.0 }
}

#[test]
fn container_resolve_trait() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<MyTraitImpl, _>(|_| MyTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn MyTrait, MyTraitImpl, _>(
        |arc| arc as Arc<dyn MyTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_trait::<dyn MyTrait>();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), 42);
}

#[test]
fn container_resolve_trait_in() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<MyTraitImpl, _>(|_| MyTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn MyTrait, MyTraitImpl, _>(
        |arc| arc as Arc<dyn MyTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_trait_in::<dyn MyTrait>(&scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), 42);
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_qualified_trait / resolve_qualified_trait_in
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_qualified_trait() {
    let mut builder = RegistryBuilder::new();
    let qual = q("my_trait");
    builder.register(
        ComponentDefinition::singleton::<MyTraitImpl, _>(|_| MyTraitImpl(42)),
    ).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn MyTrait, MyTraitImpl, _>(
        |arc| arc as Arc<dyn MyTrait>,
    ).qualified(qual.clone())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_qualified_trait::<dyn MyTrait>(&qual);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), 42);
}

#[test]
fn container_resolve_qualified_trait_in() {
    let mut builder = RegistryBuilder::new();
    let qual = q("my_trait");
    builder.register(
        ComponentDefinition::singleton::<MyTraitImpl, _>(|_| MyTraitImpl(42)),
    ).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn MyTrait, MyTraitImpl, _>(
        |arc| arc as Arc<dyn MyTrait>,
    ).qualified(qual.clone())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_qualified_trait_in::<dyn MyTrait>(&qual, &scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), 42);
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_all_traits / resolve_all_traits_in
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_all_traits() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<MyTraitImpl, _>(|_| MyTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn MyTrait, MyTraitImpl, _>(
        |arc| arc as Arc<dyn MyTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_all_traits::<dyn MyTrait>();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
}

#[test]
fn container_resolve_all_traits_in() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<MyTraitImpl, _>(|_| MyTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn MyTrait, MyTraitImpl, _>(
        |arc| arc as Arc<dyn MyTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_all_traits_in::<dyn MyTrait>(&scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — warm_up / registry / unused_definitions
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
fn container_unused_definitions() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let unused = container.unused_definitions();
    // String is used in the test, so it might or might not be unused
    let _ = unused;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — scope with transient components
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_in_transient() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::transient::<String, _>(|_| "transient".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_in::<String>(&scope);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "transient");
    scope.close();
}

#[test]
fn container_resolve_in_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_in::<Vec<String>>(&scope);
    assert!(result.is_err());
    scope.close();
}

#[test]
fn container_resolve_qualified_in_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(
        ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
            .qualified(q("greeting")),
    ).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_qualified_in::<String>(&q("other"), &scope);
    assert!(result.is_err());
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_trait with ambiguous
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_trait_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_trait::<dyn MyTrait>();
    assert!(result.is_err());
}

#[test]
fn container_resolve_trait_in_wrong_scope() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<MyTraitImpl, _>(|_| MyTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn MyTrait, MyTraitImpl, _>(
        |arc| arc as Arc<dyn MyTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry.clone());
    // Create scope from a different container
    let container2 = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container2.resolve_trait_in::<dyn MyTrait>(&scope);
    assert!(result.is_err());
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test helpers
// ═══════════════════════════════════════════════════════════════════════════════

struct TestPostProcessor;

impl vernal_beans::BeanPostProcessor for TestPostProcessor {
    fn post_process_after_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(Some(bean))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_all_traits_in
// ═══════════════════════════════════════════════════════════════════════════════

trait DisplayTrait: Send + Sync {
    fn display(&self) -> &str;
}

struct DisplayImpl(String);
impl DisplayTrait for DisplayImpl {
    fn display(&self) -> &str { &self.0 }
}

#[test]
fn container_resolve_all_traits_in_with_scope() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<DisplayImpl, _>(|_| DisplayImpl("hello".to_string()))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn DisplayTrait, DisplayImpl, _>(
        |arc| arc as Arc<dyn DisplayTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_all_traits_in::<dyn DisplayTrait>(&scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_trait with scope
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_trait_with_scope() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<DisplayImpl, _>(|_| DisplayImpl("scoped".to_string()))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn DisplayTrait, DisplayImpl, _>(
        |arc| arc as Arc<dyn DisplayTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_trait_in::<dyn DisplayTrait>(&scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().display(), "scoped");
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_qualified_trait with scope
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_qualified_trait_with_scope() {
    let mut builder = RegistryBuilder::new();
    let qual = q("my_display");
    builder.register(ComponentDefinition::singleton::<DisplayImpl, _>(|_| DisplayImpl("qualified".to_string()))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn DisplayTrait, DisplayImpl, _>(
        |arc| arc as Arc<dyn DisplayTrait>,
    ).qualified(qual.clone())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_qualified_trait_in::<dyn DisplayTrait>(&qual, &scope);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().display(), "qualified");
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_qualified_in with scope
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_qualified_in_with_scope() {
    let mut builder = RegistryBuilder::new();
    builder.register(
        ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
            .qualified(q("greeting")),
    ).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container.resolve_qualified_in::<String>(&q("greeting"), &scope);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_qualified_in wrong scope owner
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_qualified_in_wrong_scope() {
    let mut builder = RegistryBuilder::new();
    builder.register(
        ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
            .qualified(q("greeting")),
    ).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry.clone());
    let container2 = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container2.resolve_qualified_in::<String>(&q("greeting"), &scope);
    assert!(result.is_err());
    scope.close();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_trait not found
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_trait_not_found_display() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_trait::<dyn DisplayTrait>();
    assert!(result.is_err());
}

#[test]
fn container_resolve_trait_in_wrong_owner_scope() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<DisplayImpl, _>(|_| DisplayImpl("hello".to_string()))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn DisplayTrait, DisplayImpl, _>(
        |arc| arc as Arc<dyn DisplayTrait>,
    )).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry.clone());
    let container2 = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container2.resolve_trait_in::<dyn DisplayTrait>(&scope);
    assert!(result.is_err());
    scope.close();
}

#[test]
fn container_resolve_qualified_trait_in_wrong_scope() {
    let mut builder = RegistryBuilder::new();
    let qual = q("my_display");
    builder.register(ComponentDefinition::singleton::<DisplayImpl, _>(|_| DisplayImpl("hello".to_string()))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn DisplayTrait, DisplayImpl, _>(
        |arc| arc as Arc<dyn DisplayTrait>,
    ).qualified(qual.clone())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry.clone());
    let container2 = Container::new(registry);
    let scope = container.open_scope::<RequestScope>();
    let result = container2.resolve_qualified_trait_in::<dyn DisplayTrait>(&qual, &scope);
    assert!(result.is_err());
    scope.close();
}
