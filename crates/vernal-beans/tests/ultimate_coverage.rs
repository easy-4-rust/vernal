//! 终极覆盖率测试 — 覆盖 container.rs、transient_tracker.rs、type_converter.rs、
//! scope_error.rs、scope_key.rs、root_bean_definition.rs、resolver.rs、
//! bean_definition_builder.rs、component_provider.rs、trait_provider.rs 等文件中
//! 所有剩余未覆盖的代码路径。

use std::any::{Any, TypeId};
use std::sync::Arc;

use vernal_beans::{
    Autowire, BeanDefinition, BeanPostProcessor, ComponentDefinition, ComponentKey, Container,
    GenericBeanDefinition, Qualifier, RegistryBuilder, Resolver, RootBeanDefinition, Scope,
    ScopeError, ScopeKey, ScopeState, TransientTracker, TypeConverter,
};
use vernal_beans::bean_definition_builder::BeanDefinitionBuilder;
use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
use vernal_beans::bean_factory::BeanFactory;
use vernal_beans::dependency_descriptor::DependencyDescriptor;
use vernal_beans::AutowireCapableBeanFactory;

fn q(name: &str) -> Qualifier {
    Qualifier::new(name).unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════════
// TransientTracker 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn transient_tracker_default() {
    let tracker = TransientTracker::default();
    let type_id = TypeId::of::<String>();
    assert_eq!(tracker.total_surviving(), 0);
    assert!(tracker.surviving_instances(type_id).is_empty());
}

#[test]
fn transient_tracker_track_and_surviving() {
    let tracker = TransientTracker::new();
    let type_id = TypeId::of::<String>();
    let instance: Arc<dyn Any + Send + Sync> = Arc::new("hello".to_string());
    tracker.track(type_id, &instance);
    let surviving = tracker.surviving_instances(type_id);
    assert_eq!(surviving.len(), 1);
    assert_eq!(surviving[0].downcast_ref::<String>().unwrap(), "hello");
}

#[test]
fn transient_tracker_surviving_filters_dead() {
    let tracker = TransientTracker::new();
    let type_id = TypeId::of::<i32>();
    {
        let instance: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        tracker.track(type_id, &instance);
    }
    let surviving = tracker.surviving_instances(type_id);
    assert_eq!(surviving.len(), 0);
}

#[test]
fn transient_tracker_surviving_unknown_type() {
    let tracker = TransientTracker::new();
    let surviving = tracker.surviving_instances(TypeId::of::<Vec<u8>>());
    assert!(surviving.is_empty());
}

#[test]
fn transient_tracker_total_surviving() {
    let tracker = TransientTracker::new();
    let t1 = TypeId::of::<String>();
    let t2 = TypeId::of::<i32>();
    let i1: Arc<dyn Any + Send + Sync> = Arc::new("a".to_string());
    let i2: Arc<dyn Any + Send + Sync> = Arc::new(1i32);
    let i3: Arc<dyn Any + Send + Sync> = Arc::new(2i32);
    tracker.track(t1, &i1);
    tracker.track(t2, &i2);
    tracker.track(t2, &i3);
    assert_eq!(tracker.total_surviving(), 3);
}

#[test]
fn transient_tracker_clear() {
    let tracker = TransientTracker::new();
    let type_id = TypeId::of::<String>();
    let instance: Arc<dyn Any + Send + Sync> = Arc::new("hello".to_string());
    tracker.track(type_id, &instance);
    assert_eq!(tracker.total_surviving(), 1);
    tracker.clear();
    assert_eq!(tracker.total_surviving(), 0);
}

#[test]
fn transient_tracker_multiple_tracks_same_type() {
    let tracker = TransientTracker::new();
    let type_id = TypeId::of::<String>();
    let i1: Arc<dyn Any + Send + Sync> = Arc::new("a".to_string());
    let i2: Arc<dyn Any + Send + Sync> = Arc::new("b".to_string());
    tracker.track(type_id, &i1);
    tracker.track(type_id, &i2);
    let surviving = tracker.surviving_instances(type_id);
    assert_eq!(surviving.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TypeConverter 测试
// ═══════════════════════════════════════════════════════════════════════════════

struct TestTypeConverter;

impl TypeConverter for TestTypeConverter {
    fn convert_if_necessary(
        &self,
        _property_name: Option<&str>,
        value: &dyn Any,
        target_type: std::any::TypeId,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(s) = value.downcast_ref::<String>() {
            if target_type == TypeId::of::<i32>() {
                let n: i32 = s.parse().map_err(|e| format!("{e}"))?;
                Ok(Box::new(n))
            } else {
                Ok(Box::new(s.clone()))
            }
        } else {
            Err("unsupported conversion".into())
        }
    }
}

#[test]
fn type_converter_convert_value_default() {
    let converter = TestTypeConverter;
    let value = "42".to_string();
    let result = converter.convert_value(&value, TypeId::of::<i32>());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap().downcast_ref::<i32>().unwrap(), 42);
}

#[test]
fn type_converter_convert_value_same_type() {
    let converter = TestTypeConverter;
    let value = "hello".to_string();
    let result = converter.convert_value(&value, TypeId::of::<String>());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().downcast_ref::<String>().unwrap(), "hello");
}

#[test]
fn type_converter_convert_if_necessary_with_property_name() {
    let converter = TestTypeConverter;
    let value = "42".to_string();
    let result = converter.convert_if_necessary(Some("myProp"), &value, TypeId::of::<i32>());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap().downcast_ref::<i32>().unwrap(), 42);
}

#[test]
fn type_converter_conversion_error() {
    let converter = TestTypeConverter;
    let value = "not_a_number".to_string();
    let result = converter.convert_value(&value, TypeId::of::<i32>());
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeError 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_error_display_resolution() {
    let scope = ScopeKey::of::<String>();
    let resolve_err = vernal_beans::ResolveError::NotFound {
        component: "test".to_string(),
        path: vec![],
    };
    let err = ScopeError::Resolution { scope, source: Box::new(resolve_err) };
    let display = format!("{err}");
    assert!(display.contains("resolution failed"));
}

#[test]
fn scope_error_display_close_hook() {
    let scope = ScopeKey::of::<String>();
    let source: vernal_core::SharedError = Arc::new(std::io::Error::new(std::io::ErrorKind::Other, "hook failed"));
    let err = ScopeError::CloseHook { scope, source };
    let display = format!("{err}");
    assert!(display.contains("close hook failed"));
}

#[test]
fn scope_error_display_close_task() {
    let scope = ScopeKey::of::<String>();
    let source: vernal_core::SharedError = Arc::new(std::io::Error::new(std::io::ErrorKind::Other, "task failed"));
    let err = ScopeError::CloseTask { scope, source };
    let display = format!("{err}");
    assert!(display.contains("close task failed"));
}

#[test]
fn scope_error_source_resolution() {
    let scope = ScopeKey::of::<String>();
    let resolve_err = vernal_beans::ResolveError::NotFound {
        component: "test".to_string(),
        path: vec![],
    };
    let err = ScopeError::Resolution { scope, source: Box::new(resolve_err) };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn scope_error_source_close_hook() {
    let scope = ScopeKey::of::<String>();
    let source: vernal_core::SharedError = Arc::new(std::io::Error::new(std::io::ErrorKind::Other, "hook failed"));
    let err = ScopeError::CloseHook { scope, source };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn scope_error_source_close_task() {
    let scope = ScopeKey::of::<String>();
    let source: vernal_core::SharedError = Arc::new(std::io::Error::new(std::io::ErrorKind::Other, "task failed"));
    let err = ScopeError::CloseTask { scope, source };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn scope_error_source_invalid_state_is_none() {
    let scope = ScopeKey::of::<String>();
    let err = ScopeError::InvalidState { operation: "test", scope, state: ScopeState::Open };
    assert!(std::error::Error::source(&err).is_none());
}

#[test]
fn scope_error_display_invalid_state() {
    let scope = ScopeKey::of::<String>();
    let err = ScopeError::InvalidState { operation: "test_op", scope, state: ScopeState::Open };
    let display = format!("{err}");
    assert!(display.contains("test_op"));
}

#[test]
fn scope_error_display_cancelled() {
    let scope = ScopeKey::of::<String>();
    let err = ScopeError::Cancelled { operation: "cancel_op", scope };
    let display = format!("{err}");
    assert!(display.contains("cancelled"));
}

#[test]
fn scope_error_display_type_mismatch() {
    let scope = ScopeKey::of::<String>();
    let err = ScopeError::TypeMismatch { scope, expected: "MyType" };
    let display = format!("{err}");
    assert!(display.contains("MyType"));
}

#[test]
fn scope_error_display_close_timeout() {
    use std::time::Duration;
    let scope = ScopeKey::of::<String>();
    let err = ScopeError::CloseTimeout { scope, timeout: Duration::from_secs(5) };
    let display = format!("{err}");
    assert!(display.contains("timeout"));
}

#[test]
fn scope_error_display_runtime_unavailable() {
    let scope = ScopeKey::of::<String>();
    let err = ScopeError::RuntimeUnavailable { scope };
    let display = format!("{err}");
    assert!(display.contains("Tokio runtime"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeKey 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_key_type_name() {
    let key = ScopeKey::of::<String>();
    assert!(key.type_name().contains("String"));
}

#[test]
fn scope_key_display() {
    let key = ScopeKey::of::<i32>();
    let display = format!("{key}");
    assert!(display.contains("i32"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// RootBeanDefinition — BeanDefinition trait 方法测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn root_bean_definition_bean_class_name_none() {
    let def = RootBeanDefinition::new();
    assert_eq!(def.bean_class_name(), "unknown");
}

#[test]
fn root_bean_definition_bean_class_name_some() {
    let mut def = RootBeanDefinition::new();
    def.set_bean_class_name("com.example.MyBean");
    assert_eq!(def.bean_class_name(), "com.example.MyBean");
}

#[test]
fn root_bean_definition_scope() {
    let mut def = RootBeanDefinition::new();
    def.set_scope(Scope::Transient);
    assert!(def.scope().is_transient());
}

#[test]
fn root_bean_definition_is_lazy_init() {
    let mut def = RootBeanDefinition::new();
    def.set_lazy_init(true);
    assert!(def.is_lazy_init());
}

#[test]
fn root_bean_definition_is_primary() {
    let mut def = RootBeanDefinition::new();
    def.set_primary(true);
    assert!(def.is_primary());
}

#[test]
fn root_bean_definition_is_fallback() {
    let mut def = RootBeanDefinition::new();
    def.set_fallback(true);
    assert!(def.is_fallback());
}

#[test]
fn root_bean_definition_is_autowire_candidate() {
    let mut def = RootBeanDefinition::new();
    def.set_autowire_candidate(false);
    assert!(!def.is_autowire_candidate());
}

#[test]
fn root_bean_definition_role() {
    let mut def = RootBeanDefinition::new();
    def.set_role(2);
    assert_eq!(def.role(), 2);
}

#[test]
fn root_bean_definition_description() {
    let mut def = RootBeanDefinition::new();
    def.set_description("test description");
    assert_eq!(def.description(), Some("test description"));
}

#[test]
fn root_bean_definition_parent_name() {
    let mut def = RootBeanDefinition::new();
    def.set_parent_name("parentBean");
    assert_eq!(def.parent_name(), Some("parentBean"));
}

#[test]
fn root_bean_definition_factory_bean_name() {
    let mut def = RootBeanDefinition::new();
    def.set_factory_bean_name("factoryBean");
    assert_eq!(def.factory_bean_name(), Some("factoryBean"));
}

#[test]
fn root_bean_definition_factory_method_name() {
    let mut def = RootBeanDefinition::new();
    def.set_factory_method_name("create");
    assert_eq!(def.factory_method_name(), Some("create"));
}

#[test]
fn root_bean_definition_init_method_name() {
    let mut def = RootBeanDefinition::new();
    def.set_init_method_name("init");
    assert_eq!(def.init_method_name(), Some("init"));
}

#[test]
fn root_bean_definition_destroy_method_name() {
    let mut def = RootBeanDefinition::new();
    def.set_destroy_method_name("destroy");
    assert_eq!(def.destroy_method_name(), Some("destroy"));
}

#[test]
fn root_bean_definition_is_abstract() {
    let mut def = RootBeanDefinition::new();
    def.set_abstract(true);
    assert!(def.is_abstract());
}

// ═══════════════════════════════════════════════════════════════════════════════
// BeanDefinitionBuilder 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_builder_generic_all_methods() {
    let def = BeanDefinitionBuilder::generic("com.example.MyBean")
        .set_fallback(true)
        .set_role(1)
        .set_description("test desc")
        .set_init_method("myInit")
        .set_destroy_method("myDestroy")
        .set_factory_bean_name("factory")
        .set_factory_method_name("create")
        .add_constructor_arg_value(42i32)
        .add_constructor_arg_typed(42i32, "i32")
        .add_property_value("myProp", "value")
        .build();
    assert!(def.is_fallback());
    assert_eq!(def.role(), 1);
    assert_eq!(def.description(), Some("test desc"));
    assert_eq!(def.init_method_name(), Some("myInit"));
    assert_eq!(def.destroy_method_name(), Some("myDestroy"));
    assert_eq!(def.factory_bean_name(), Some("factory"));
    assert_eq!(def.factory_method_name(), Some("create"));
}

#[test]
fn bean_definition_builder_get_definition() {
    let builder = BeanDefinitionBuilder::generic("com.example.MyBean");
    let _ = builder.get_definition();
}

#[test]
fn bean_definition_builder_root() {
    let def = BeanDefinitionBuilder::root("com.example.MyBean")
        .set_scope(Scope::Transient)
        .set_lazy_init(true)
        .set_primary(true)
        .set_init_method("init")
        .set_destroy_method("destroy")
        .add_depends_on("dep")
        .add_constructor_arg_value(42i32)
        .add_property_value("prop", "val")
        .build();
    assert!(def.scope().is_transient());
    assert!(def.is_lazy_init());
    assert!(def.is_primary());
    assert_eq!(def.init_method_name(), Some("init"));
    assert_eq!(def.destroy_method_name(), Some("destroy"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — AutowireCapableBeanFactory 测试
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

#[test]
fn container_create_bean_factory_failure() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| panic!("factory failure"))).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let type_name = std::any::type_name::<String>().to_string();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = container.create_bean(&type_name);
    }));
    assert!(result.is_err());
}

#[test]
fn container_autowire_bean_with_unresolved_deps() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    builder.register(
        ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
            let _dep = resolver.resolve::<String>().unwrap();
            42
        }).depends_on::<String>(),
    ).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(0i32);
    let result = container.autowire_bean(existing);
    assert!(result.is_ok());
}

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
fn container_autowire_bean_multiple_match() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()).qualified(q("a"))).unwrap();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "b".to_string()).qualified(q("b"))).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = container.autowire_bean(existing);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_resolve_error() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32)).unwrap();
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

#[test]
fn container_configure_bean_with_post_processor() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.add_bean_post_processor(Arc::new(TestPostProcessor));
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.configure_bean(existing, "test");
    assert!(result.is_ok());
}

#[test]
fn container_configure_bean_failing_post_processor() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.add_bean_post_processor(Arc::new(FailingPostProcessor));
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.configure_bean(existing, "test");
    assert!(result.is_ok());
}

#[test]
fn container_initialize_bean_with_post_processor() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.add_bean_post_processor(Arc::new(TestPostProcessor));
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.initialize_bean(existing, "test");
    assert!(result.is_ok());
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
fn container_apply_bean_property_values() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.apply_bean_property_values(existing, "test");
    assert!(result.is_ok());
}

#[test]
fn container_destroy_bean_instance() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.destroy_bean_instance("test", &"hello");
    assert!(result.is_ok());
}

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

#[test]
fn container_resolve_dependency_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let descriptor = DependencyDescriptor::for_field(TypeId::of::<Vec<String>>(), "Vec<String>");
    let result = container.resolve_dependency(&descriptor, None);
    assert!(result.is_err());
}

#[test]
fn container_resolve_dependency_optional_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let descriptor = DependencyDescriptor::for_field(TypeId::of::<Vec<String>>(), "Vec<String>")
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
    let descriptor = DependencyDescriptor::for_field(TypeId::of::<String>(), "String");
    let result = container.resolve_dependency(&descriptor, None);
    assert!(result.is_err());
}

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
// Container — autowire 各种模式
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_autowire_by_name() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    // Use the actual type_name of String
    let type_name = std::any::type_name::<String>();
    let result = container.autowire(type_name, 1, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_by_type() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let type_name = std::any::type_name::<String>();
    let result = container.autowire(type_name, 2, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_constructor() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let type_name = std::any::type_name::<String>();
    let result = container.autowire(type_name, 3, false);
    assert!(result.is_ok());
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

// ═══════════════════════════════════════════════════════════════════════════════
// Container — autowire_bean_properties 各种模式
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_autowire_bean_properties_no_mode() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.autowire_bean_properties(existing, 0, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_properties_by_name() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.autowire_bean_properties(existing, 1, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_properties_by_type() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.autowire_bean_properties(existing, 2, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_properties_unknown_mode() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.autowire_bean_properties(existing, 99, false);
    assert!(result.is_ok());
}

#[test]
fn container_autowire_bean_properties_with_matching_dep() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    builder.register(
        ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
            let _dep = resolver.resolve::<String>().unwrap();
            42
        }).depends_on::<String>(),
    ).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let existing: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = container.autowire_bean_properties(existing, 1, false);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — BeanFactory trait 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_factory_get_bean_provider() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let provider = container.get_bean_provider_by_type_id(TypeId::of::<String>());
    assert!(provider.is_ok());
    let p = provider.unwrap();
    assert!(p.if_available().is_none());
}

#[test]
fn bean_factory_get_bean_provider_get_if_unique() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let provider = container.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
    let result = provider.get_if_unique();
    assert!(result.is_err());
}

#[test]
fn bean_factory_get_bean_provider_stream() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let provider = container.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
    assert!(provider.stream().is_empty());
    assert!(provider.ordered_stream().is_empty());
}

#[test]
fn bean_factory_get_bean_provider_stream_with_cached_singleton() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let _ = container.resolve::<String>();
    let provider = container.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
    assert_eq!(provider.stream().len(), 1);
    assert_eq!(provider.ordered_stream().len(), 1);
}

#[test]
fn bean_factory_get_bean_provider_get_with_cached() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let _ = container.resolve::<String>();
    let provider = container.get_bean_provider_by_type_id(TypeId::of::<String>()).unwrap();
    assert!(provider.get().is_ok());
    assert!(provider.if_available().is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — BeanDefinitionRegistry 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_registry_register_and_get() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    assert!(container.get_bean_definition("myBean").is_some());
}

#[test]
fn bean_definition_registry_register_duplicate() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def1 = Box::new(RootBeanDefinition::new());
    let def2 = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def1).unwrap();
    let result = container.register_bean_definition("myBean".to_string(), def2);
    assert!(result.is_err());
}

#[test]
fn bean_definition_registry_remove_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let result = container.remove_bean_definition("nonexistent");
    assert!(result.is_err());
}

#[test]
fn bean_definition_registry_remove_from_dynamic() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    assert!(container.remove_bean_definition("myBean").is_ok());
}

#[test]
fn bean_definition_registry_remove_from_registry() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    let type_name = std::any::type_name::<String>().to_string();
    assert!(container.remove_bean_definition(&type_name).is_ok());
}

#[test]
fn bean_definition_registry_contains_bean_definition() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    // Use actual type_name
    let type_name = std::any::type_name::<String>().to_string();
    assert!(container.contains_bean_definition(&type_name));
    assert!(!container.contains_bean_definition("nonexistent"));
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    assert!(container.contains_bean_definition("myBean"));
}

#[test]
fn bean_definition_registry_contains_deleted() {
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
fn bean_definition_registry_count() {
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

#[test]
fn bean_definition_registry_get_bean_definition_from_registry() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let type_name = std::any::type_name::<String>().to_string();
    assert!(container.get_bean_definition(&type_name).is_some());
}

#[test]
fn bean_definition_registry_get_bean_definition_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    assert!(container.get_bean_definition("nonexistent").is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Resolver — resolve_qualified, resolve_optional, providers
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn resolver_resolve_qualified() {
    let mut builder = RegistryBuilder::new();
    builder.register(
        ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
            .qualified(q("greeting")),
    ).unwrap();
    let qual = Qualifier::new("greeting").unwrap();
    builder.register(ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        let qual = Qualifier::new("greeting").unwrap();
        resolver.resolve_qualified::<String>(&qual).unwrap().len() as i32
    }).depends_on_qualified::<String>(qual)).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    assert_eq!(*container.resolve::<i32>().unwrap(), 5);
}

#[test]
fn resolver_resolve_optional_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    builder.register(ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if let Ok(Some(val)) = resolver.resolve_optional::<String>() {
            val.len() as i32
        } else {
            0
        }
    }).depends_on_optional::<String>()).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    assert_eq!(*container.resolve::<i32>().unwrap(), 5);
}

#[test]
fn resolver_resolve_optional_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if resolver.resolve_optional::<String>().is_ok_and(|v| v.is_some()) { 1 } else { 0 }
    }).depends_on_optional::<String>()).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    assert_eq!(*container.resolve::<i32>().unwrap(), 0);
}

#[test]
fn resolver_optional_provider() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    // Provider can be created in factory, but can't call get() during construction
    builder.register(ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if let Ok(provider) = resolver.optional_provider::<String>() {
            let _is_optional = provider.is_optional();
            return 42;
        }
        0
    }).depends_on_optional_provider::<String>()).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    assert_eq!(*container.resolve::<i32>().unwrap(), 42);
}

#[test]
fn resolver_optional_qualified_provider() {
    let mut builder = RegistryBuilder::new();
    builder.register(
        ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
            .qualified(q("greeting")),
    ).unwrap();
    let qual = Qualifier::new("greeting").unwrap();
    // Provider can be created in factory, but can't call get() during construction
    builder.register(ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        let qual = Qualifier::new("greeting").unwrap();
        if let Ok(provider) = resolver.optional_qualified_provider::<String>(&qual) {
            let _is_optional = provider.is_optional();
            return 42;
        }
        0
    }).depends_on_optional_qualified_provider::<String>(qual)).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    assert_eq!(*container.resolve::<i32>().unwrap(), 42);
}

#[test]
fn resolver_resolve_optional_qualified() {
    let mut builder = RegistryBuilder::new();
    builder.register(
        ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
            .qualified(q("greeting")),
    ).unwrap();
    let qual = Qualifier::new("greeting").unwrap();
    builder.register(ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        let qual = Qualifier::new("greeting").unwrap();
        if let Ok(Some(val)) = resolver.resolve_optional_qualified::<String>(&qual) {
            val.len() as i32
        } else {
            0
        }
    }).depends_on_optional_qualified::<String>(qual)).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    assert_eq!(*container.resolve::<i32>().unwrap(), 5);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — resolve_definition with BeanPostProcessor error
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_with_failing_post_processor() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.add_bean_post_processor(Arc::new(FailingPostProcessor));
    assert!(container.resolve::<String>().is_ok());
}

#[test]
fn container_resolve_with_replacing_post_processor() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.add_bean_post_processor(Arc::new(ReplacingPostProcessor));
    assert_eq!(*container.resolve::<String>().unwrap(), "replaced");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — select_definition ambiguous/not-found
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_ambiguous_definition() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()).qualified(q("a"))).unwrap();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "b".to_string()).qualified(q("b"))).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    assert!(container.resolve::<String>().is_err());
}

#[test]
fn container_resolve_not_found() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    assert!(container.resolve::<Vec<String>>().is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — circular dependency detection
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_circular_dependency() {
    let mut builder = RegistryBuilder::new();
    // Use depends_on for eager dependencies which trigger cycle detection during build
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
// Test helpers
// ═══════════════════════════════════════════════════════════════════════════════

struct TestPostProcessor;

impl BeanPostProcessor for TestPostProcessor {
    fn post_process_after_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(Some(bean))
    }
}

struct FailingPostProcessor;

impl BeanPostProcessor for FailingPostProcessor {
    fn post_process_after_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Err("post processor error".into())
    }
}

struct ReplacingPostProcessor;

impl BeanPostProcessor for ReplacingPostProcessor {
    fn post_process_after_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        if bean.downcast_ref::<String>().is_some() {
            Ok(Some(Arc::new("replaced".to_string())))
        } else {
            Ok(None)
        }
    }
}

struct FailingBeforeInitProcessor;

impl BeanPostProcessor for FailingBeforeInitProcessor {
    fn post_process_before_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Err("before init failed".into())
    }
}
