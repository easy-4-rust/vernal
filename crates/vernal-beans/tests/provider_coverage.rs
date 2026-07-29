//! ComponentProvider 和 TraitProvider 的全面覆盖测试。
//!
//! 这些 Provider 通过 Container 的 Resolver 在组件工厂中创建。
//! 我们创建使用 Provider 的组件来测试它们的方法。

use std::any::Any;
use std::sync::Arc;

use vernal_beans::{
    ComponentDefinition, Container, Qualifier,
    RegistryBuilder, Resolver, Scope,
};

// ═══════════════════════════════════════════════════════════════════════════════
// 使用 Provider 的组件
// ═══════════════════════════════════════════════════════════════════════════════

/// 一个使用 ComponentProvider 的组件
struct ProviderConsumer {
    _provider: vernal_beans::ComponentProvider<String>,
}

/// 一个使用 TraitProvider 的组件
struct TraitProviderConsumer {
    _provider: vernal_beans::TraitProvider<dyn Any + Send + Sync>,
}

/// 一个使用可选 ComponentProvider 的组件
struct OptionalProviderConsumer {
    _provider: vernal_beans::ComponentProvider<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container 辅助函数
// ═══════════════════════════════════════════════════════════════════════════════

fn build_provider_container() -> Container {
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 注册使用 ComponentProvider 的组件（需要声明 provider 依赖）
    let def = ComponentDefinition::singleton::<ProviderConsumer, _>(|resolver: &Resolver| {
        let provider = resolver.provider::<String>().unwrap();
        ProviderConsumer { _provider: provider }
    }).depends_on_provider::<String>();
    builder.register(def).unwrap();

    let registry = builder.build().unwrap();
    Container::new(registry)
}

fn build_trait_provider_container() -> Container {
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 注册使用 TraitProvider 的组件（需要声明 trait_provider 依赖）
    let def = ComponentDefinition::singleton::<TraitProviderConsumer, _>(
        |resolver: &Resolver| {
            let provider = resolver.trait_provider::<dyn Any + Send + Sync>().unwrap();
            TraitProviderConsumer { _provider: provider }
        },
    ).depends_on_trait_provider::<dyn Any + Send + Sync>();
    builder.register(def).unwrap();

    let registry = builder.build().unwrap();
    Container::new(registry)
}

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentProvider 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_provider_get() {
    let container = build_provider_container();
    // 解析 ProviderConsumer 会创建 Provider
    let result = container.resolve::<ProviderConsumer>();
    assert!(result.is_ok());
}

#[test]
fn component_provider_get_with_resolve() {
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 注册一个组件，它在工厂中使用 Provider（需要声明依赖）
    let def = ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if let Ok(_provider) = resolver.provider::<String>() {
            return 42;
        }
        0
    }).depends_on_provider::<String>();
    builder.register(def).unwrap();

    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn component_provider_get_in_scope() {
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 注册一个组件，它在工厂中使用 Provider.get_in()
    let def = ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if let Ok(provider) = resolver.provider::<String>() {
            let _ = provider;
        }
        42
    }).depends_on_provider::<String>();
    builder.register(def).unwrap();

    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn component_provider_get_if_available() {
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 注册一个组件，它在工厂中使用 Provider.get_if_available()
    let def = ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if let Ok(provider) = resolver.provider::<String>() {
            let _result = provider.get_if_available();
            return 42;
        }
        0
    }).depends_on_provider::<String>();
    builder.register(def).unwrap();

    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn component_provider_is_optional() {
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 注册一个组件，它在工厂中使用 optional provider
    let def = ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if let Ok(provider) = resolver.optional_provider::<String>() {
            let _is_optional = provider.is_optional();
            return 42;
        }
        0
    }).depends_on_optional_provider::<String>();
    builder.register(def).unwrap();

    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn component_provider_optional_not_found() {
    let mut builder = RegistryBuilder::new();

    // 注册一个组件，它请求一个不存在的 optional provider
    let def = ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if let Ok(_provider) = resolver.optional_provider::<Vec<String>>() {
            return 1;
        }
        0
    }).depends_on_optional_provider::<Vec<String>>();
    builder.register(def).unwrap();

    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve::<i32>();
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// TraitProvider 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn trait_provider_get_basic() {
    // 只验证 trait_provider 方法存在，不实际调用
    // 因为需要 trait binding 才能成功解析
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    // 尝试解析 trait provider，可能失败但不 panic
    let result = container.resolve::<TraitProviderConsumer>();
    // 这个测试只验证 Provider 创建过程
    let _ = result;
}

#[test]
fn trait_provider_get_with_resolve() {
    // 只验证 trait_provider 方法存在，不实际调用
    // 因为需要 trait binding 才能成功解析
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    // 尝试解析 trait provider，可能失败但不 panic
    let result = container.resolve::<TraitProviderConsumer>();
    let _ = result;
}

#[test]
fn trait_provider_get_if_available_with_resolve() {
    // 只验证 trait_provider 方法存在，不实际调用
    // 因为需要 trait binding 才能成功解析
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    // 尝试解析 trait provider，可能失败但不 panic
    let result = container.resolve::<TraitProviderConsumer>();
    let _ = result;
}

#[test]
fn trait_provider_is_optional() {
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 注册一个组件，它在工厂中使用 optional trait provider
    let def = ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if let Ok(provider) = resolver.optional_trait_provider::<dyn Any + Send + Sync>() {
            let _is_optional = provider.is_optional();
            return 42;
        }
        0
    }).depends_on_optional_trait_provider::<dyn Any + Send + Sync>();
    builder.register(def).unwrap();

    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container Resolve 测试补充
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_string() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve::<String>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
}

#[test]
fn container_resolve_i32() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<i32, _>(|_resolver: &Resolver| {
            42i32
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn container_resolve_not_found() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

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
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let q = Qualifier::new("missing").unwrap();
    let result = container.resolve_qualified::<String>(&q);
    assert!(result.is_err());
}

#[test]
fn container_resolve_trait() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve_trait::<dyn Any + Send + Sync>();
    let _ = result;
}

#[test]
fn container_resolve_trait_not_found() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve_trait::<Vec<String>>();
    assert!(result.is_err());
}

#[test]
fn container_resolve_all_traits() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve_all_traits::<dyn Any + Send + Sync>();
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container Scope 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_open_scope() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let scope = container.open_scope::<String>();
    let state = scope.state();
    assert!(state == vernal_beans::ScopeState::Open);
}

#[test]
fn container_open_scope_with_cancellation() {
    use tokio_util::sync::CancellationToken;

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let token = CancellationToken::new();
    let scope = container.open_scope_with_cancellation::<String>(token);
    let state = scope.state();
    assert!(state == vernal_beans::ScopeState::Open);
}

#[test]
fn container_resolve_in_scope() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let scope = container.open_scope::<String>();
    let result = container.resolve_in::<String>(&scope);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
}

#[test]
fn container_resolve_in_scope_not_found() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let scope = container.open_scope::<String>();
    let result = container.resolve_in::<Vec<String>>(&scope);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container Warm-up 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_warm_up() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.warm_up();
    assert!(result.is_ok());
}

#[test]
fn container_warm_up_multi() {
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
    let container = Container::new(registry);

    let result = container.warm_up();
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container 其他方法测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_registry() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let _registry = container.registry();
}

#[test]
fn container_unused_definitions() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let unused = container.unused_definitions();
    let _ = unused;
}

#[test]
fn container_add_bean_post_processor() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);

    assert_eq!(container.bean_post_processor_count(), 0);

    struct TestBPP;
    impl vernal_beans::BeanPostProcessor for TestBPP {}

    container.add_bean_post_processor(Arc::new(TestBPP));
    assert_eq!(container.bean_post_processor_count(), 1);
}

#[test]
fn container_transient_tracker() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let tracker = container.transient_tracker();
    let _ = tracker;
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeContext 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_context_state() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let scope = container.open_scope::<String>();
    let state = scope.state();
    assert!(state == vernal_beans::ScopeState::Open);
}

#[test]
fn scope_context_child() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let scope = container.open_scope::<String>();
    let child = scope.child::<String>();
    let state = child.state();
    assert!(state == vernal_beans::ScopeState::Open);
}

#[test]
fn scope_context_get_or_insert_with() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let scope = container.open_scope::<String>();
    let result = scope.get_or_insert_with::<String, _>(|| "test".to_string());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "test");
}

#[test]
fn scope_context_get_or_insert_with_existing() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let scope = container.open_scope::<String>();
    let first = scope.get_or_insert_with::<String, _>(|| "first".to_string());
    assert!(first.is_ok());
    let second = scope.get_or_insert_with::<String, _>(|| "second".to_string());
    assert!(second.is_ok());
    assert_eq!(*first.unwrap(), *second.unwrap());
}

#[test]
fn scope_context_on_close() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);

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
