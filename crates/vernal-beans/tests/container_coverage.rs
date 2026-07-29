//! Container 和其他核心模块的全面覆盖测试。

use std::any::Any;
use std::sync::Arc;

use vernal_beans::{
    ComponentDefinition, Container, Qualifier,
    RegistryBuilder, Resolver, Scope, ScopeKey,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Container 基础测试
// ═══════════════════════════════════════════════════════════════════════════════

fn build_string_container() -> Container {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    Container::new(registry)
}

fn build_i32_container() -> Container {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<i32, _>(|_resolver: &Resolver| {
            42i32
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    Container::new(registry)
}

fn build_multi_container() -> Container {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    builder
        .register(ComponentDefinition::singleton::<i32, _>(|_resolver: &Resolver| {
            42i32
        }))
        .unwrap();
    builder
        .register(ComponentDefinition::singleton::<bool, _>(|_resolver: &Resolver| {
            true
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    Container::new(registry)
}

#[test]
fn container_new() {
    let container = build_string_container();
    assert_eq!(container.bean_post_processor_count(), 0);
}

#[test]
fn container_registry() {
    let container = build_string_container();
    let _registry = container.registry();
}

#[test]
fn container_unused_definitions() {
    let container = build_string_container();
    let unused = container.unused_definitions();
    let _ = unused;
}

#[test]
fn container_add_bean_post_processor() {
    let mut container = build_string_container();
    assert_eq!(container.bean_post_processor_count(), 0);

    struct TestBPP;
    impl vernal_beans::BeanPostProcessor for TestBPP {}

    container.add_bean_post_processor(Arc::new(TestBPP));
    assert_eq!(container.bean_post_processor_count(), 1);
}

#[test]
fn container_add_multiple_bean_post_processors() {
    let mut container = build_string_container();
    struct TestBPP1;
    impl vernal_beans::BeanPostProcessor for TestBPP1 {}
    struct TestBPP2;
    impl vernal_beans::BeanPostProcessor for TestBPP2 {}

    container.add_bean_post_processor(Arc::new(TestBPP1));
    container.add_bean_post_processor(Arc::new(TestBPP2));
    assert_eq!(container.bean_post_processor_count(), 2);
}

#[test]
fn container_resolve_string() {
    let container = build_string_container();
    let result = container.resolve::<String>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
}

#[test]
fn container_resolve_i32() {
    let container = build_i32_container();
    let result = container.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn container_resolve_not_found() {
    let container = build_string_container();
    let result = container.resolve::<Vec<String>>();
    assert!(result.is_err());
}

#[test]
fn container_resolve_qualified() {
    let mut builder = RegistryBuilder::new();
    let q = Qualifier::new("primary").unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
                "qualified".to_string()
            })
            .qualified(q.clone()),
        )
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve_qualified::<String>(&q);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "qualified");
}

#[test]
fn container_resolve_qualified_not_found() {
    let container = build_string_container();
    let q = Qualifier::new("missing").unwrap();
    let result = container.resolve_qualified::<String>(&q);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container Scope 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_open_scope() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    assert!(scope.state() == vernal_beans::ScopeState::Open);
}

#[test]
fn container_open_scope_with_cancellation() {
    use tokio_util::sync::CancellationToken;

    let container = build_string_container();
    let token = CancellationToken::new();
    let scope = container.open_scope_with_cancellation::<String>(token);
    assert!(scope.state() == vernal_beans::ScopeState::Open);
}

#[test]
fn container_resolve_in_scope() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let result = container.resolve_in::<String>(&scope);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
}

#[test]
fn container_resolve_qualified_in_scope() {
    let mut builder = RegistryBuilder::new();
    let q = Qualifier::new("primary").unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
                "qualified".to_string()
            })
            .qualified(q.clone()),
        )
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<String>();

    let result = container.resolve_qualified_in::<String>(&q, &scope);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container Warm-up 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_warm_up() {
    let container = build_string_container();
    let result = container.warm_up();
    assert!(result.is_ok());
}

#[test]
fn container_warm_up_multi() {
    let container = build_multi_container();
    let result = container.warm_up();
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container Transient Tracker 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_transient_tracker() {
    let container = build_string_container();
    let tracker = container.transient_tracker();
    let _ = tracker;
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeContext 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_context_state() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let state = scope.state();
    assert_eq!(state, vernal_beans::ScopeState::Open);
}

#[test]
fn scope_context_child() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let child = scope.child::<String>();
    assert!(child.state() == vernal_beans::ScopeState::Open);
}

#[test]
fn scope_context_close() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let state_before = scope.state();
    assert!(state_before == vernal_beans::ScopeState::Open || state_before == vernal_beans::ScopeState::Closing);
    // close() 是异步的，这里只测试调用不 panic
    let _ = scope.close();
}

#[test]
fn scope_context_get_or_insert_with() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let result = scope.get_or_insert_with::<String, _>(|| "test".to_string());
    assert!(result.is_ok());
}

#[test]
fn scope_context_get_or_insert_with_existing() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let _ = scope.get_or_insert_with::<String, _>(|| "first".to_string());
    let result = scope.get_or_insert_with::<String, _>(|| "second".to_string());
    assert!(result.is_ok());
}

#[test]
fn scope_context_on_close() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    // on_close 注册钩子，这里只测试调用不 panic
    let result = scope.on_close(move || {
        async { Ok::<(), std::io::Error>(()) }
    });
    // 注册可能失败（如果 scope 已关闭），这里只验证不 panic
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentDefinition 测试补充
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_definition_singleton() {
    let def = ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
        "test".to_string()
    });
    assert_eq!(def.scope(), Scope::Singleton);
}

#[test]
fn component_definition_transient() {
    let def = ComponentDefinition::transient::<String, _>(|_resolver: &Resolver| {
        "test".to_string()
    });
    assert_eq!(def.scope(), Scope::Transient);
}

#[test]
fn component_definition_try_singleton() {
    let def = ComponentDefinition::try_singleton::<String, _>(|_resolver: &Resolver| {
        Ok("test".to_string())
    });
    assert_eq!(def.scope(), Scope::Singleton);
}

#[test]
fn component_definition_try_transient() {
    let def = ComponentDefinition::try_transient::<String, _>(|_resolver: &Resolver| {
        Ok("test".to_string())
    });
    assert_eq!(def.scope(), Scope::Transient);
}

#[test]
fn component_definition_scoped() {
    let def = ComponentDefinition::scoped::<String, String, _>(|_resolver: &Resolver| {
        "test".to_string()
    });
    assert!(matches!(def.scope(), Scope::Custom(_)));
}

#[test]
fn component_definition_try_scoped() {
    let def = ComponentDefinition::try_scoped::<String, String, _>(|_resolver: &Resolver| {
        Ok("test".to_string())
    });
    assert!(matches!(def.scope(), Scope::Custom(_)));
}

#[test]
fn component_definition_qualified() {
    let q = Qualifier::new("primary").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
        "test".to_string()
    })
    .qualified(q);
    assert!(def.key().qualifier().is_some());
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
fn component_definition_key() {
    let def = ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
        "test".to_string()
    });
    assert_eq!(def.key().type_name(), std::any::type_name::<String>());
}

#[test]
fn component_definition_debug() {
    let def = ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
        "test".to_string()
    });
    let debug = format!("{:?}", def);
    assert!(debug.contains("ComponentDefinition"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// RegistryBuilder 测试补充
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn registry_builder_new() {
    let builder = RegistryBuilder::new();
    let _ = builder;
}

#[test]
fn registry_builder_default() {
    let builder = RegistryBuilder::default();
    let _ = builder;
}

#[test]
fn registry_builder_register_duplicate() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "first".to_string()
        }))
        .unwrap();

    let result = builder.register(ComponentDefinition::singleton::<String, _>(
        |_resolver: &Resolver| "second".to_string(),
    ));
    assert!(result.is_err());
}

#[test]
fn registry_builder_build() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "test".to_string()
        }))
        .unwrap();

    let registry = builder.build();
    assert!(registry.is_ok());
}

#[test]
fn registry_snapshot() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "test".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let snapshot = registry.snapshot();
    let _ = snapshot;
}

// ═══════════════════════════════════════════════════════════════════════════════
// ResolveError 测试补充
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn resolve_error_not_found() {
    let err = vernal_beans::ResolveError::NotFound {
        component: "test".to_string(),
        path: vec![],
    };
    let msg = format!("{err}");
    assert!(msg.contains("not found") || msg.contains("NotFound"));
}

#[test]
fn resolve_error_ambiguous() {
    let err = vernal_beans::ResolveError::Ambiguous {
        component: "test".to_string(),
        candidates: vec!["a".to_string(), "b".to_string()],
        path: vec![],
    };
    let msg = format!("{err}");
    assert!(msg.contains("ambiguous") || msg.contains("Ambiguous"));
}

#[test]
fn resolve_error_scope_not_active() {
    let key = vernal_beans::ComponentKey::of::<String>();
    let scope_key = ScopeKey::of::<String>();
    let err = vernal_beans::ResolveError::ScopeNotActive {
        component: key,
        scope: scope_key,
    };
    let msg = format!("{err}");
    assert!(msg.contains("scope") || msg.contains("Scope"));
}

#[test]
fn resolve_error_type_mismatch() {
    let key = vernal_beans::ComponentKey::of::<String>();
    let err = vernal_beans::ResolveError::TypeMismatch {
        component: key,
    };
    let msg = format!("{err}");
    assert!(msg.contains("type") || msg.contains("Type"));
}

#[test]
fn resolve_error_provider_used_during_construction() {
    let key = vernal_beans::ComponentKey::of::<String>();
    let err = vernal_beans::ResolveError::ProviderUsedDuringConstruction {
        component: key,
        dependency: "test".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("provider") || msg.contains("Provider"));
}

#[test]
fn resolve_error_scope_owner_mismatch() {
    let scope_key = ScopeKey::of::<String>();
    let err = vernal_beans::ResolveError::ScopeOwnerMismatch {
        scope: scope_key,
    };
    let msg = format!("{err}");
    // 只验证格式化不 panic
    assert!(!msg.is_empty());
}

#[test]
fn resolve_error_undeclared_dependency() {
    let key = vernal_beans::ComponentKey::of::<String>();
    let err = vernal_beans::ResolveError::UndeclaredDependency {
        component: key,
        dependency: "test".to_string(),
    };
    let msg = format!("{err}");
    // 只验证格式化不 panic
    assert!(!msg.is_empty());
}

#[test]
fn resolve_error_circular_runtime() {
    let err = vernal_beans::ResolveError::CircularRuntime {
        path: vec!["a".to_string(), "b".to_string(), "a".to_string()],
    };
    let msg = format!("{err}");
    // 只验证格式化不 panic
    assert!(!msg.is_empty());
}

#[test]
fn resolve_error_scope_unavailable() {
    let key = vernal_beans::ComponentKey::of::<String>();
    let scope_key = ScopeKey::of::<String>();
    let err = vernal_beans::ResolveError::ScopeUnavailable {
        component: key,
        scope: scope_key,
        state: vernal_beans::ScopeState::Closed,
        cancelled: false,
    };
    let msg = format!("{err}");
    assert!(msg.contains("unavailable") || msg.contains("Unavailable"));
}

#[test]
fn resolve_error_trait_binding_type_mismatch() {
    let trait_key = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    let target = vernal_beans::ComponentKey::of::<String>();
    let err = vernal_beans::ResolveError::TraitBindingTypeMismatch {
        binding: trait_key,
        target,
    };
    let msg = format!("{err}");
    assert!(msg.contains("trait") || msg.contains("Trait"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeKey 补充测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_key_hash() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    let k1 = ScopeKey::of::<String>();
    let k2 = ScopeKey::of::<String>();
    map.insert(k1, 1);
    assert_eq!(map.get(&k2), Some(&1));
}

#[test]
fn scope_key_ne() {
    let k1 = ScopeKey::of::<String>();
    let k2 = ScopeKey::of::<i32>();
    assert_ne!(k1, k2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// GenericBeanDefinition 补充测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn generic_bean_definition_set_init_order() {
    let mut gbd = vernal_beans::GenericBeanDefinition::new();
    gbd.set_init_order(100);
    let _ = gbd;
}

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
    assert!(gbd.is_lazy_init());
    assert!(gbd.is_primary());
    assert_eq!(gbd.description().unwrap(), "root desc");
    assert_eq!(gbd.autowire_mode(), vernal_beans::Autowire::ByType);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TraitBinding 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn trait_binding_basic() {
    let binding = vernal_beans::TraitBinding::new::<dyn Any + Send + Sync, String, _>(
        |arc| arc as Arc<dyn Any + Send + Sync>,
    );
    let _ = binding;
}

#[test]
fn trait_binding_display() {
    let binding = vernal_beans::TraitBinding::new::<dyn Any + Send + Sync, String, _>(
        |arc| arc as Arc<dyn Any + Send + Sync>,
    );
    let msg = format!("{binding}");
    assert!(!msg.is_empty());
}

#[test]
fn trait_binding_debug() {
    let binding = vernal_beans::TraitBinding::new::<dyn Any + Send + Sync, String, _>(
        |arc| arc as Arc<dyn Any + Send + Sync>,
    );
    let debug = format!("{binding:?}");
    assert!(debug.contains("TraitBinding"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ObjectProvider trait 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn object_provider_trait_exists() {
    fn _assert<T: vernal_beans::ObjectProvider<String>>() {}
}

// ═══════════════════════════════════════════════════════════════════════════════
// InjectionPoint 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn injection_point_basic() {
    let ip = vernal_beans::InjectionPoint::new(
        std::any::TypeId::of::<String>(),
        std::any::type_name::<String>(),
    );
    let _ = ip;
}

// ═══════════════════════════════════════════════════════════════════════════════
// DependencyDescriptor 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn dependency_descriptor_for_field() {
    let dd = vernal_beans::DependencyDescriptor::for_field(
        std::any::TypeId::of::<String>(),
        std::any::type_name::<String>(),
    );
    let _ = dd;
}

#[test]
fn dependency_descriptor_for_constructor_parameter() {
    let dd = vernal_beans::DependencyDescriptor::for_constructor_parameter(
        0,
        std::any::TypeId::of::<String>(),
        std::any::type_name::<String>(),
    );
    let _ = dd;
}

#[test]
fn dependency_descriptor_for_method_parameter() {
    let dd = vernal_beans::DependencyDescriptor::for_method_parameter(
        0,
        std::any::TypeId::of::<String>(),
        std::any::type_name::<String>(),
    );
    let _ = dd;
}

// ═══════════════════════════════════════════════════════════════════════════════
// TransientTracker 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn transient_tracker_new() {
    let tracker = vernal_beans::TransientTracker::new();
    let _ = tracker;
}
