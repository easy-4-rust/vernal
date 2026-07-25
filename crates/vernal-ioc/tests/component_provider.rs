//! 类型安全延迟组件 Provider 合同测试。

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use vernal_core::BoxError;
use vernal_ioc::{
    ComponentDefinition, ComponentProvider, GraphError, Qualifier, RegistryBuilder, ResolveError,
};

/// 每次构造都携带不同序号的瞬时对象。
struct Sequence(usize);

/// 长生命周期组件只保存受限 Provider，不捕获某个瞬时对象。
struct SequenceFactory {
    provider: ComponentProvider<Sequence>,
}

#[test]
fn provider_defers_transient_construction_and_returns_a_new_instance_per_call() {
    let constructions = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&constructions);
    let mut registry = RegistryBuilder::new();
    registry
        .register(
            ComponentDefinition::try_singleton::<SequenceFactory, _>(
                |resolver| -> Result<SequenceFactory, BoxError> {
                    Ok(SequenceFactory {
                        provider: resolver.provider::<Sequence>()?,
                    })
                },
            )
            .depends_on_provider::<Sequence>(),
        )
        .expect("provider consumer definition");
    registry
        .register(ComponentDefinition::transient::<Sequence, _>(move |_| {
            Sequence(observed.fetch_add(1, Ordering::SeqCst))
        }))
        .expect("transient target definition");
    let container = registry.build().expect("valid provider graph").container();

    container
        .warm_up()
        .expect("provider consumer singleton should warm up");
    assert_eq!(constructions.load(Ordering::SeqCst), 0);

    let factory = container
        .resolve::<SequenceFactory>()
        .expect("provider consumer should resolve");
    let first = factory.provider.get().expect("first transient");
    let second = factory.provider.get().expect("second transient");

    assert_eq!(first.0, 0);
    assert_eq!(second.0, 1);
    assert!(!Arc::ptr_eq(&first, &second));
}

/// 用于验证构造期 Provider 重入保护的目标。
#[derive(Debug)]
struct ConstructionTarget;

/// 工厂尚未返回时不能通过 Provider 发起新的解析。
struct ConstructionConsumer;

#[test]
fn provider_rejects_use_before_its_consumer_factory_has_completed() {
    let rejected = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed = Arc::clone(&rejected);
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::transient::<ConstructionTarget, _>(
            |_| ConstructionTarget,
        ))
        .expect("construction target");
    registry
        .register(
            ComponentDefinition::try_singleton::<ConstructionConsumer, _>(
                move |resolver| -> Result<ConstructionConsumer, BoxError> {
                    let provider = resolver.provider::<ConstructionTarget>()?;
                    let error = provider
                        .get()
                        .expect_err("provider must reject construction-time re-entry");
                    observed.store(
                        matches!(error, ResolveError::ProviderUsedDuringConstruction { .. }),
                        Ordering::SeqCst,
                    );
                    Ok(ConstructionConsumer)
                },
            )
            .depends_on_provider::<ConstructionTarget>(),
        )
        .expect("construction consumer");
    let container = registry.build().expect("valid provider graph").container();

    container
        .resolve::<ConstructionConsumer>()
        .expect("consumer should complete after observing structured rejection");
    assert!(rejected.load(Ordering::SeqCst));
}

/// 不一定由应用安装的扩展类型。
struct OptionalExtension;

/// 通过可选 Provider 表达“零或一个”扩展点。
struct OptionalExtensionConsumer {
    provider: ComponentProvider<OptionalExtension>,
}

#[test]
fn optional_provider_allows_a_missing_definition_without_hiding_other_errors() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(
            ComponentDefinition::try_singleton::<OptionalExtensionConsumer, _>(
                |resolver| -> Result<OptionalExtensionConsumer, BoxError> {
                    Ok(OptionalExtensionConsumer {
                        provider: resolver.optional_provider::<OptionalExtension>()?,
                    })
                },
            )
            .depends_on_optional_provider::<OptionalExtension>(),
        )
        .expect("optional provider consumer definition");
    let container = registry
        .build()
        .expect("missing optional target is valid")
        .container();
    let consumer = container
        .resolve::<OptionalExtensionConsumer>()
        .expect("optional provider consumer");

    assert!(consumer.provider.is_optional());
    assert!(
        consumer
            .provider
            .get_if_available()
            .expect("optional lookup")
            .is_none()
    );
    assert!(matches!(
        consumer.provider.get(),
        Err(ResolveError::NotFound { .. })
    ));
}

/// 必须安装的 Provider 目标。
struct RequiredExtension;

/// 声明 required Provider 的消费方。
struct RequiredExtensionConsumer;

#[test]
fn required_provider_rejects_a_missing_target_while_building_the_graph() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(
            ComponentDefinition::singleton::<RequiredExtensionConsumer, _>(|_| {
                RequiredExtensionConsumer
            })
            .depends_on_provider::<RequiredExtension>(),
        )
        .expect("required provider consumer definition");

    let error = registry
        .build()
        .expect_err("missing required provider target must fail");
    let GraphError::MissingDependency { path } = error else {
        panic!("expected missing provider dependency");
    };
    assert_eq!(path.len(), 2);
    assert!(path[0].ends_with("RequiredExtensionConsumer"));
    assert!(path[1].contains("provider<"));
    assert!(path[1].ends_with("RequiredExtension>"));
}

/// 验证 optional 只放宽零候选，而不会默默选择多个候选。
struct AmbiguousExtension;

/// 声明 optional Provider 的歧义消费方。
struct AmbiguousExtensionConsumer;

#[test]
fn optional_provider_does_not_hide_ambiguous_candidates() {
    let primary = Qualifier::new("primary").expect("valid qualifier");
    let secondary = Qualifier::new("secondary").expect("valid qualifier");
    let mut registry = RegistryBuilder::new();
    registry
        .register(
            ComponentDefinition::singleton::<AmbiguousExtension, _>(|_| AmbiguousExtension)
                .qualified(primary),
        )
        .expect("primary extension");
    registry
        .register(
            ComponentDefinition::singleton::<AmbiguousExtension, _>(|_| AmbiguousExtension)
                .qualified(secondary),
        )
        .expect("secondary extension");
    registry
        .register(
            ComponentDefinition::singleton::<AmbiguousExtensionConsumer, _>(|_| {
                AmbiguousExtensionConsumer
            })
            .depends_on_optional_provider::<AmbiguousExtension>(),
        )
        .expect("optional provider consumer");

    let error = registry
        .build()
        .expect_err("optional provider ambiguity must fail");
    let GraphError::AmbiguousDependency { path, candidates } = error else {
        panic!("expected ambiguous provider dependency");
    };
    assert_eq!(path.len(), 2);
    assert_eq!(candidates.len(), 2);
    assert!(path[1].contains("optional_provider<"));
}

/// 验证限定符选择的值对象。
struct NamedSequence(&'static str);

/// 只允许访问 `blue` 候选的 Provider 消费方。
struct QualifiedConsumer {
    provider: ComponentProvider<NamedSequence>,
}

#[test]
fn qualified_provider_preserves_the_exact_build_time_selector() {
    let blue = Qualifier::new("blue").expect("valid qualifier");
    let red = Qualifier::new("red").expect("valid qualifier");
    let mut registry = RegistryBuilder::new();
    registry
        .register(
            ComponentDefinition::transient::<NamedSequence, _>(|_| NamedSequence("blue"))
                .qualified(blue.clone()),
        )
        .expect("blue target");
    registry
        .register(
            ComponentDefinition::transient::<NamedSequence, _>(|_| NamedSequence("red"))
                .qualified(red),
        )
        .expect("red target");
    registry
        .register(
            ComponentDefinition::try_singleton::<QualifiedConsumer, _>({
                let factory_qualifier = blue.clone();
                move |resolver| -> Result<QualifiedConsumer, BoxError> {
                    Ok(QualifiedConsumer {
                        provider: resolver
                            .qualified_provider::<NamedSequence>(&factory_qualifier)?,
                    })
                }
            })
            .depends_on_qualified_provider::<NamedSequence>(blue),
        )
        .expect("qualified consumer");
    let container = registry
        .build()
        .expect("qualified provider graph")
        .container();

    let consumer = container
        .resolve::<QualifiedConsumer>()
        .expect("qualified consumer");
    assert_eq!(consumer.provider.get().expect("blue target").0, "blue");
}

/// 请求作用域标记。
struct RequestScope;

/// 只允许在请求作用域中创建的对象。
struct RequestValue;

/// Singleton 通过 Provider 在每次请求传入的 Scope 中取得对象。
struct RequestValueFactory {
    provider: ComponentProvider<RequestValue>,
}

#[test]
fn provider_requires_the_callers_current_scope_instead_of_capturing_one() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<RequestValue, RequestScope, _>(|_| RequestValue))
        .expect("request value definition");
    registry
        .register(
            ComponentDefinition::try_singleton::<RequestValueFactory, _>(
                |resolver| -> Result<RequestValueFactory, BoxError> {
                    Ok(RequestValueFactory {
                        provider: resolver.provider::<RequestValue>()?,
                    })
                },
            )
            .depends_on_provider::<RequestValue>(),
        )
        .expect("request provider definition");
    let frozen = registry.build().expect("scoped provider graph");
    let container = frozen.container();
    let other_container = frozen.container();
    let factory = container
        .resolve::<RequestValueFactory>()
        .expect("provider factory");

    assert!(matches!(
        factory.provider.get(),
        Err(ResolveError::ScopeNotActive { .. })
    ));
    let scope = container.open_scope::<RequestScope>();
    let first = factory.provider.get_in(&scope).expect("request value");
    let second = factory
        .provider
        .get_in(&scope)
        .expect("cached request value");
    assert!(Arc::ptr_eq(&first, &second));

    let foreign_scope = other_container.open_scope::<RequestScope>();
    assert!(matches!(
        factory.provider.get_in(&foreign_scope),
        Err(ResolveError::ScopeOwnerMismatch { .. })
    ));
}

/// Provider 环的一侧；构造时不会立即请求 `ProviderCycleB`。
struct ProviderCycleA {
    provider: ComponentProvider<ProviderCycleB>,
}

/// 环的另一侧直接依赖已经构造的 `ProviderCycleA`。
struct ProviderCycleB {
    a: Arc<ProviderCycleA>,
}

#[test]
fn deferred_provider_edge_can_break_an_eager_constructor_cycle() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(
            ComponentDefinition::try_singleton::<ProviderCycleA, _>(
                |resolver| -> Result<ProviderCycleA, BoxError> {
                    Ok(ProviderCycleA {
                        provider: resolver.provider::<ProviderCycleB>()?,
                    })
                },
            )
            .depends_on_provider::<ProviderCycleB>(),
        )
        .expect("cycle A definition");
    registry
        .register(
            ComponentDefinition::try_singleton::<ProviderCycleB, _>(
                |resolver| -> Result<ProviderCycleB, BoxError> {
                    Ok(ProviderCycleB {
                        a: resolver.resolve::<ProviderCycleA>()?,
                    })
                },
            )
            .depends_on::<ProviderCycleA>(),
        )
        .expect("cycle B definition");
    let container = registry
        .build()
        .expect("deferred edge should not form constructor cycle")
        .container();

    container.warm_up().expect("both singletons should warm up");
    let a = container.resolve::<ProviderCycleA>().expect("cycle A");
    let b = a.provider.get().expect("cycle B through provider");
    assert!(Arc::ptr_eq(&a, &b.a));
}
