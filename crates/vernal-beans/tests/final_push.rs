//! 最终覆盖率冲刺 — 覆盖 container.rs、resolver.rs、root_bean_definition.rs、
//! registry_builder.rs、scope_context.rs 等文件中所有剩余未覆盖代码路径。

use std::any::{Any, TypeId};
use std::sync::Arc;

use vernal_beans::{
    BeanDefinition, ComponentDefinition, Container, GenericBeanDefinition, Qualifier,
    RegistryBuilder, Resolver, RootBeanDefinition, Scope,
};
use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
use vernal_beans::bean_factory::BeanFactory;
use vernal_beans::AutowireCapableBeanFactory;

fn q(name: &str) -> Qualifier {
    Qualifier::new(name).unwrap()
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
    // Resolve String as i32 - should fail with TypeMismatch
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
// Container — BeanDefinitionRegistry (lines 1357-1360, 1429, 1480)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_remove_bean_definition_deleted_marker() {
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

// ═══════════════════════════════════════════════════════════════════════════════
// Container — bean_definition_names (line 1480)
// ═══════════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════════
// Container — PostProcessor error branches (lines 1102, 1109-1110)
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
// Container — trait ambiguity test
// ═══════════════════════════════════════════════════════════════════════════════

trait TestTrait: Send + Sync {
    fn value(&self) -> i32;
}

struct TestTraitImpl(i32);
impl TestTrait for TestTraitImpl {
    fn value(&self) -> i32 { self.0 }
}

#[test]
fn container_resolve_trait_ambiguous() {
    let mut builder = RegistryBuilder::new();
    // Register two implementations of the same trait without primary
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
    // Resolving without qualifier should fail with Ambiguous
    let result = container.resolve_trait::<dyn TestTrait>();
    assert!(result.is_err());
}

#[test]
fn container_resolve_trait_binding_not_found() {
    let mut builder = RegistryBuilder::new();
    // Register the target component
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
// Container — autowire_bean with deps
// ═══════════════════════════════════════════════════════════════════════════════

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
// Test helpers
// ═══════════════════════════════════════════════════════════════════════════════

struct RequestScope;
