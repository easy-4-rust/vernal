//! BeanFactory trait 实现和 container 内部方法的全面覆盖测试。

use std::any::Any;
use std::sync::Arc;

use vernal_beans::{
    BeanFactory, ComponentDefinition, Container, Qualifier,
    RegistryBuilder, Resolver, Scope,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Container 辅助函数
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

fn build_qualified_container() -> Container {
    let mut builder = RegistryBuilder::new();
    let q = Qualifier::new("primary").unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
                "qualified".to_string()
            })
            .qualified(q),
        )
        .unwrap();
    let registry = builder.build().unwrap();
    Container::new(registry)
}

// ═══════════════════════════════════════════════════════════════════════════════
// BeanFactory trait 方法测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_factory_get_bean_by_type_id() {
    let container = build_string_container();
    let key = vernal_beans::ComponentKey::of::<String>();
    let result = container.get_bean_by_key(&key);
    assert!(result.is_ok());
}

#[test]
fn bean_factory_get_bean_by_type_id_not_found() {
    let container = build_string_container();
    let key = vernal_beans::ComponentKey::of::<Vec<String>>();
    let result = container.get_bean_by_key(&key);
    assert!(result.is_err());
}

#[test]
fn bean_factory_contains_bean() {
    let container = build_string_container();
    let key = vernal_beans::ComponentKey::of::<String>();
    assert!(container.contains_bean(&key));

    let key2 = vernal_beans::ComponentKey::of::<Vec<String>>();
    assert!(!container.contains_bean(&key2));
}

#[test]
fn bean_factory_is_singleton() {
    let container = build_string_container();
    let key = vernal_beans::ComponentKey::of::<String>();
    let result = container.is_singleton(&key);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn bean_factory_is_prototype() {
    let container = build_string_container();
    let key = vernal_beans::ComponentKey::of::<String>();
    let result = container.is_prototype(&key);
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[test]
fn bean_factory_get_type() {
    let container = build_string_container();
    let key = vernal_beans::ComponentKey::of::<String>();
    let result = container.get_type(&key);
    assert!(result.is_ok());
}

#[test]
fn bean_factory_get_type_not_found() {
    let container = build_string_container();
    let key = vernal_beans::ComponentKey::of::<Vec<String>>();
    let result = container.get_type(&key);
    assert!(result.is_err());
}

#[test]
fn bean_factory_get_aliases() {
    let container = build_string_container();
    let key = vernal_beans::ComponentKey::of::<String>();
    let aliases = container.get_aliases(&key);
    let _ = aliases;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container Resolve 补充测试
// ═══════════════════════════════════════════════════════════════════════════════

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
    let container = build_qualified_container();
    let q = Qualifier::new("primary").unwrap();
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

#[test]
fn container_resolve_trait() {
    let container = build_string_container();
    let result = container.resolve_trait::<dyn Any + Send + Sync>();
    let _ = result;
}

#[test]
fn container_resolve_trait_not_found() {
    let container = build_string_container();
    let result = container.resolve_trait::<Vec<String>>();
    assert!(result.is_err());
}

#[test]
fn container_resolve_all_traits() {
    let container = build_string_container();
    let result = container.resolve_all_traits::<dyn Any + Send + Sync>();
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container Scope 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_open_scope() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let state = scope.state();
    assert!(state == vernal_beans::ScopeState::Open);
}

#[test]
fn container_open_scope_with_cancellation() {
    use tokio_util::sync::CancellationToken;

    let container = build_string_container();
    let token = CancellationToken::new();
    let scope = container.open_scope_with_cancellation::<String>(token);
    let state = scope.state();
    assert!(state == vernal_beans::ScopeState::Open);
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
fn container_resolve_in_scope_not_found() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let result = container.resolve_in::<Vec<String>>(&scope);
    assert!(result.is_err());
}

#[test]
fn container_resolve_qualified_in_scope() {
    let container = build_qualified_container();
    let scope = container.open_scope::<String>();
    let q = Qualifier::new("primary").unwrap();
    let result = container.resolve_qualified_in::<String>(&q, &scope);
    assert!(result.is_ok());
}

#[test]
fn container_resolve_qualified_in_scope_not_found() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let q = Qualifier::new("missing").unwrap();
    let result = container.resolve_qualified_in::<String>(&q, &scope);
    assert!(result.is_err());
}

#[test]
fn container_resolve_trait_in_scope() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let result = container.resolve_trait_in::<dyn Any + Send + Sync>(&scope);
    let _ = result;
}

#[test]
fn container_resolve_qualified_trait() {
    let container = build_string_container();
    let q = Qualifier::new("primary").unwrap();
    let result = container.resolve_qualified_trait::<dyn Any + Send + Sync>(&q);
    let _ = result;
}

#[test]
fn container_resolve_qualified_trait_in_scope() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let q = Qualifier::new("primary").unwrap();
    let result = container.resolve_qualified_trait_in::<dyn Any + Send + Sync>(&q, &scope);
    let _ = result;
}

#[test]
fn container_resolve_all_traits_in_scope() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let result = container.resolve_all_traits_in::<dyn Any + Send + Sync>(&scope);
    let _ = result;
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
// Container 其他方法测试
// ═══════════════════════════════════════════════════════════════════════════════

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
    assert!(state == vernal_beans::ScopeState::Open);
}

#[test]
fn scope_context_child() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let child = scope.child::<String>();
    let state = child.state();
    assert!(state == vernal_beans::ScopeState::Open);
}

#[test]
fn scope_context_get_or_insert_with() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let result = scope.get_or_insert_with::<String, _>(|| "test".to_string());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "test");
}

#[test]
fn scope_context_get_or_insert_with_existing() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let first = scope.get_or_insert_with::<String, _>(|| "first".to_string());
    assert!(first.is_ok());
    let second = scope.get_or_insert_with::<String, _>(|| "second".to_string());
    assert!(second.is_ok());
    assert_eq!(*first.unwrap(), *second.unwrap());
}

#[test]
fn scope_context_on_close() {
    let container = build_string_container();
    let scope = container.open_scope::<String>();
    let result = scope.on_close(move || async { Ok::<(), std::io::Error>(()) });
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentDefinition 测试
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
// RegistryBuilder 测试
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
// ResolveError 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn resolve_error_not_found() {
    let err = vernal_beans::ResolveError::NotFound {
        component: "test".to_string(),
        path: vec![],
    };
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

#[test]
fn resolve_error_ambiguous() {
    let err = vernal_beans::ResolveError::Ambiguous {
        component: "test".to_string(),
        candidates: vec!["a".to_string(), "b".to_string()],
        path: vec![],
    };
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

#[test]
fn resolve_error_scope_not_active() {
    let key = vernal_beans::ComponentKey::of::<String>();
    let scope_key = vernal_beans::ScopeKey::of::<String>();
    let err = vernal_beans::ResolveError::ScopeNotActive {
        component: key,
        scope: scope_key,
    };
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

#[test]
fn resolve_error_type_mismatch() {
    let key = vernal_beans::ComponentKey::of::<String>();
    let err = vernal_beans::ResolveError::TypeMismatch {
        component: key,
    };
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

#[test]
fn resolve_error_provider_used_during_construction() {
    let key = vernal_beans::ComponentKey::of::<String>();
    let err = vernal_beans::ResolveError::ProviderUsedDuringConstruction {
        component: key,
        dependency: "test".to_string(),
    };
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

#[test]
fn resolve_error_scope_owner_mismatch() {
    let scope_key = vernal_beans::ScopeKey::of::<String>();
    let err = vernal_beans::ResolveError::ScopeOwnerMismatch {
        scope: scope_key,
    };
    let msg = format!("{err}");
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
    assert!(!msg.is_empty());
}

#[test]
fn resolve_error_circular_runtime() {
    let err = vernal_beans::ResolveError::CircularRuntime {
        path: vec!["a".to_string(), "b".to_string(), "a".to_string()],
    };
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

#[test]
fn resolve_error_scope_unavailable() {
    let key = vernal_beans::ComponentKey::of::<String>();
    let scope_key = vernal_beans::ScopeKey::of::<String>();
    let err = vernal_beans::ResolveError::ScopeUnavailable {
        component: key,
        scope: scope_key,
        state: vernal_beans::ScopeState::Closed,
        cancelled: false,
    };
    let msg = format!("{err}");
    assert!(!msg.is_empty());
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
    assert!(!msg.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dependency 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn dependency_of() {
    let dep = vernal_beans::Dependency::of::<String>();
    assert_eq!(dep.type_name(), std::any::type_name::<String>());
}

#[test]
fn dependency_qualified() {
    let q = Qualifier::new("test").unwrap();
    let dep = vernal_beans::Dependency::qualified::<String>(q);
    assert!(dep.qualifier().is_some());
}

#[test]
fn dependency_optional_of() {
    let dep = vernal_beans::Dependency::optional_of::<String>();
    let _ = dep;
}

#[test]
fn dependency_trait_of() {
    let dep = vernal_beans::Dependency::trait_of::<dyn Any + Send + Sync>();
    let _ = dep;
}

#[test]
fn dependency_display() {
    let dep = vernal_beans::Dependency::of::<String>();
    let msg = format!("{dep}");
    assert!(!msg.is_empty());
}

#[test]
fn dependency_debug() {
    let dep = vernal_beans::Dependency::of::<String>();
    let debug = format!("{dep:?}");
    assert!(debug.contains("Dependency"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentKey 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_key_of() {
    let k = vernal_beans::ComponentKey::of::<String>();
    assert_eq!(k.type_name(), std::any::type_name::<String>());
}

#[test]
fn component_key_display() {
    let k = vernal_beans::ComponentKey::of::<String>();
    let msg = format!("{k}");
    assert!(!msg.is_empty());
}

#[test]
fn component_key_debug() {
    let k = vernal_beans::ComponentKey::of::<String>();
    let debug = format!("{k:?}");
    assert!(debug.contains("ComponentKey"));
}

#[test]
fn component_key_clone() {
    let k = vernal_beans::ComponentKey::of::<String>();
    let cloned = k.clone();
    assert_eq!(k, cloned);
}

#[test]
fn component_key_eq() {
    let k1 = vernal_beans::ComponentKey::of::<String>();
    let k2 = vernal_beans::ComponentKey::of::<String>();
    assert_eq!(k1, k2);

    let k3 = vernal_beans::ComponentKey::of::<i32>();
    assert_ne!(k1, k3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TraitKey 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn trait_key_of() {
    let k = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    let _ = k;
}

#[test]
fn trait_key_display() {
    let k = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    let msg = format!("{k}");
    assert!(!msg.is_empty());
}

#[test]
fn trait_key_debug() {
    let k = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    let debug = format!("{k:?}");
    assert!(debug.contains("TraitKey"));
}

#[test]
fn trait_key_clone() {
    let k = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    let cloned = k.clone();
    assert_eq!(k, cloned);
}

#[test]
fn trait_key_eq() {
    let k1 = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    let k2 = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    assert_eq!(k1, k2);

    let k3 = vernal_beans::TraitKey::of::<String>();
    assert_ne!(k1, k3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Qualifier 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn qualifier_new() {
    let q = Qualifier::new("test").unwrap();
    assert_eq!(q.as_str(), "test");
}

#[test]
fn qualifier_empty() {
    let result = Qualifier::new("");
    assert!(result.is_err());
}

#[test]
fn qualifier_whitespace() {
    let result = Qualifier::new(" bad ");
    assert!(result.is_err());
}

#[test]
fn qualifier_display() {
    let q = Qualifier::new("test").unwrap();
    let msg = format!("{q}");
    assert_eq!(msg, "test");
}

#[test]
fn qualifier_debug() {
    let q = Qualifier::new("test").unwrap();
    let debug = format!("{:?}", q);
    assert!(debug.contains("test"));
}

#[test]
fn qualifier_clone() {
    let q = Qualifier::new("test").unwrap();
    let cloned = q.clone();
    assert_eq!(q, cloned);
}

#[test]
fn qualifier_hash() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    let q1 = Qualifier::new("test").unwrap();
    let q2 = Qualifier::new("test").unwrap();
    map.insert(q1, 1);
    assert_eq!(map.get(&q2), Some(&1));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeKey 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_key_of() {
    let k = vernal_beans::ScopeKey::of::<String>();
    let _ = k;
}

#[test]
fn scope_key_display() {
    let k = vernal_beans::ScopeKey::of::<String>();
    let msg = format!("{k}");
    assert!(!msg.is_empty());
}

#[test]
fn scope_key_debug() {
    let k = vernal_beans::ScopeKey::of::<String>();
    let debug = format!("{k:?}");
    assert!(debug.contains("ScopeKey"));
}

#[test]
fn scope_key_hash() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    let k1 = vernal_beans::ScopeKey::of::<String>();
    let k2 = vernal_beans::ScopeKey::of::<String>();
    map.insert(k1, 1);
    assert_eq!(map.get(&k2), Some(&1));
}

#[test]
fn scope_key_eq() {
    let k1 = vernal_beans::ScopeKey::of::<String>();
    let k2 = vernal_beans::ScopeKey::of::<String>();
    assert_eq!(k1, k2);

    let k3 = vernal_beans::ScopeKey::of::<i32>();
    assert_ne!(k1, k3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeState 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_state_open() {
    let state = vernal_beans::ScopeState::Open;
    assert_eq!(state, vernal_beans::ScopeState::Open);
}

#[test]
fn scope_state_closing() {
    let state = vernal_beans::ScopeState::Closing;
    assert_eq!(state, vernal_beans::ScopeState::Closing);
}

#[test]
fn scope_state_closed() {
    let state = vernal_beans::ScopeState::Closed;
    assert_eq!(state, vernal_beans::ScopeState::Closed);
}

#[test]
fn scope_state_default() {
    let state = vernal_beans::ScopeState::default();
    assert_eq!(state, vernal_beans::ScopeState::Open);
}

#[test]
fn scope_state_debug() {
    let state = vernal_beans::ScopeState::Open;
    let debug = format!("{:?}", state);
    assert!(debug.contains("Open"));
}

#[test]
fn scope_state_clone() {
    let state = vernal_beans::ScopeState::Closing;
    let cloned = state;
    assert_eq!(state, cloned);
}

// ═══════════════════════════════════════════════════════════════════════════════
// GraphError 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn graph_error_missing_dependency() {
    let err = vernal_beans::GraphError::MissingDependency {
        path: vec!["a".to_string(), "b".to_string()],
    };
    let msg = format!("{err}");
    assert!(msg.contains("missing dependency"));
}

#[test]
fn graph_error_ambiguous_dependency() {
    let err = vernal_beans::GraphError::AmbiguousDependency {
        path: vec!["a".to_string()],
        candidates: vec!["c1".to_string(), "c2".to_string()],
    };
    let msg = format!("{err}");
    assert!(msg.contains("ambiguous"));
}

#[test]
fn graph_error_missing_trait_binding_target() {
    let err = vernal_beans::GraphError::MissingTraitBindingTarget {
        binding: "my binding".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("trait binding"));
}

#[test]
fn graph_error_cycle() {
    let err = vernal_beans::GraphError::Cycle {
        path: vec!["a".to_string(), "b".to_string(), "a".to_string()],
    };
    let msg = format!("{err}");
    assert!(msg.contains("cycle"));
}

#[test]
fn graph_error_debug() {
    let err = vernal_beans::GraphError::Cycle {
        path: vec!["a".to_string()],
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("Cycle"));
}

#[test]
fn graph_error_clone() {
    let err = vernal_beans::GraphError::Cycle {
        path: vec!["a".to_string()],
    };
    let cloned = err.clone();
    let _ = cloned;
}

// ═══════════════════════════════════════════════════════════════════════════════
// GenericBeanDefinition 测试
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

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentSnapshot 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_snapshot_debug() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "test".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let snapshot = registry.snapshot();
    let debug = format!("{:?}", snapshot);
    assert!(!debug.is_empty());
}
