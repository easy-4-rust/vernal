//! 类型安全 Trait Object 延迟 Provider 合同测试。

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use vernal_core::BoxError;
use vernal_beans::{
    ComponentDefinition, GraphError, Qualifier, RegistryBuilder, ResolveError, TraitBinding,
    TraitProvider,
};

/// 可由多个组件实现的消息端口。
trait MessagePort: Send + Sync {
    /// 返回实现名称。
    fn name(&self) -> &'static str;

    /// 返回构造序号。
    fn sequence(&self) -> usize;
}

/// 被标记为 Primary 的瞬时端口。
struct PrimaryPort(usize);

impl MessagePort for PrimaryPort {
    fn name(&self) -> &'static str {
        "primary"
    }

    fn sequence(&self) -> usize {
        self.0
    }
}

/// 用于证明 Primary 选择而不会被误取的第二实现。
struct SecondaryPort;

impl MessagePort for SecondaryPort {
    fn name(&self) -> &'static str {
        "secondary"
    }

    fn sequence(&self) -> usize {
        usize::MAX
    }
}

/// 长生命周期组件只持有固定 Trait 选择器。
struct MessageFactory {
    provider: TraitProvider<dyn MessagePort>,
}

#[test]
fn trait_provider_selects_primary_and_constructs_transient_target_per_call() {
    let constructions = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&constructions);
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::transient::<PrimaryPort, _>(
            move |_| PrimaryPort(observed.fetch_add(1, Ordering::SeqCst)),
        ))
        .expect("primary target");
    registry
        .register(ComponentDefinition::transient::<SecondaryPort, _>(|_| {
            SecondaryPort
        }))
        .expect("secondary target");
    registry
        .bind(TraitBinding::new::<dyn MessagePort, PrimaryPort, _>(|port| port).primary())
        .expect("primary binding");
    registry
        .bind(TraitBinding::new::<dyn MessagePort, SecondaryPort, _>(
            |port| port,
        ))
        .expect("secondary binding");
    registry
        .register(
            ComponentDefinition::try_singleton::<MessageFactory, _>(
                |resolver| -> Result<MessageFactory, BoxError> {
                    Ok(MessageFactory {
                        provider: resolver.trait_provider::<dyn MessagePort>()?,
                    })
                },
            )
            .depends_on_trait_provider::<dyn MessagePort>(),
        )
        .expect("trait provider consumer");
    let container = registry.build().expect("valid trait graph").container();

    container.warm_up().expect("consumer singleton warm-up");
    assert_eq!(constructions.load(Ordering::SeqCst), 0);
    let factory = container.resolve::<MessageFactory>().expect("factory");
    let first = factory.provider.get().expect("first primary");
    let second = factory.provider.get().expect("second primary");

    assert_eq!(first.name(), "primary");
    assert_eq!(first.sequence(), 0);
    assert_eq!(second.sequence(), 1);
    assert!(!Arc::ptr_eq(&first, &second));
}

/// 没有安装时允许缺失的插件端口。
trait OptionalPlugin: Send + Sync {}

/// 持有可选 Trait Provider 的组件。
struct OptionalPluginConsumer {
    provider: TraitProvider<dyn OptionalPlugin>,
}

#[test]
fn optional_trait_provider_allows_zero_bindings_but_required_does_not() {
    let mut optional_registry = RegistryBuilder::new();
    optional_registry
        .register(
            ComponentDefinition::try_singleton::<OptionalPluginConsumer, _>(
                |resolver| -> Result<OptionalPluginConsumer, BoxError> {
                    Ok(OptionalPluginConsumer {
                        provider: resolver.optional_trait_provider::<dyn OptionalPlugin>()?,
                    })
                },
            )
            .depends_on_optional_trait_provider::<dyn OptionalPlugin>(),
        )
        .expect("optional trait provider consumer");
    let optional_container = optional_registry
        .build()
        .expect("zero optional bindings are valid")
        .container();
    let consumer = optional_container
        .resolve::<OptionalPluginConsumer>()
        .expect("optional consumer");
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

    let mut required_registry = RegistryBuilder::new();
    required_registry
        .register(
            ComponentDefinition::singleton::<OptionalPluginConsumer, _>(|_| {
                unreachable!("invalid graph must fail before construction")
            })
            .depends_on_trait_provider::<dyn OptionalPlugin>(),
        )
        .expect("required trait provider consumer");
    assert!(matches!(
        required_registry.build(),
        Err(GraphError::MissingDependency { .. })
    ));
}

/// 用限定符选择的路由端口。
trait NamedPort: Send + Sync {
    /// 返回绑定名称。
    fn name(&self) -> &'static str;
}

/// 蓝色路由实现。
struct BluePort;

impl NamedPort for BluePort {
    fn name(&self) -> &'static str {
        "blue"
    }
}

/// 红色路由实现。
struct RedPort;

impl NamedPort for RedPort {
    fn name(&self) -> &'static str {
        "red"
    }
}

/// 只允许访问蓝色命名绑定的消费方。
struct NamedPortConsumer {
    provider: TraitProvider<dyn NamedPort>,
}

#[test]
fn qualified_trait_provider_preserves_exact_binding_selector() {
    let blue = Qualifier::new("blue").expect("valid qualifier");
    let red = Qualifier::new("red").expect("valid qualifier");
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::singleton::<BluePort, _>(|_| BluePort))
        .expect("blue target");
    registry
        .register(ComponentDefinition::singleton::<RedPort, _>(|_| RedPort))
        .expect("red target");
    registry
        .bind(TraitBinding::new::<dyn NamedPort, BluePort, _>(|port| port).qualified(blue.clone()))
        .expect("blue binding");
    registry
        .bind(TraitBinding::new::<dyn NamedPort, RedPort, _>(|port| port).qualified(red))
        .expect("red binding");
    registry
        .register(
            ComponentDefinition::try_singleton::<NamedPortConsumer, _>({
                let factory_qualifier = blue.clone();
                move |resolver| -> Result<NamedPortConsumer, BoxError> {
                    Ok(NamedPortConsumer {
                        provider: resolver
                            .qualified_trait_provider::<dyn NamedPort>(&factory_qualifier)?,
                    })
                }
            })
            .depends_on_qualified_trait_provider::<dyn NamedPort>(blue),
        )
        .expect("named consumer");
    let container = registry.build().expect("named trait graph").container();

    let consumer = container.resolve::<NamedPortConsumer>().expect("consumer");
    assert_eq!(
        consumer.provider.get().expect("blue binding").name(),
        "blue"
    );
}

/// 验证 optional Trait Provider 不会吞掉多绑定歧义。
struct AmbiguousNamedConsumer;

#[test]
fn optional_trait_provider_does_not_hide_ambiguous_bindings() {
    let blue = Qualifier::new("blue").expect("valid qualifier");
    let red = Qualifier::new("red").expect("valid qualifier");
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::singleton::<BluePort, _>(|_| BluePort))
        .expect("blue target");
    registry
        .register(ComponentDefinition::singleton::<RedPort, _>(|_| RedPort))
        .expect("red target");
    registry
        .bind(TraitBinding::new::<dyn NamedPort, BluePort, _>(|port| port).qualified(blue))
        .expect("blue binding");
    registry
        .bind(TraitBinding::new::<dyn NamedPort, RedPort, _>(|port| port).qualified(red))
        .expect("red binding");
    registry
        .register(
            ComponentDefinition::singleton::<AmbiguousNamedConsumer, _>(|_| AmbiguousNamedConsumer)
                .depends_on_optional_trait_provider::<dyn NamedPort>(),
        )
        .expect("ambiguous consumer");

    assert!(matches!(
        registry.build(),
        Err(GraphError::AmbiguousDependency { .. })
    ));
}

/// 请求作用域标记。
struct RequestScope;

/// 请求级 Trait 能力。
trait RequestPort: Send + Sync {}

/// 请求级 Trait 实现。
struct RequestPortImpl;

impl RequestPort for RequestPortImpl {}

/// Singleton 按每次调用传入的 Scope 解析 Trait 实现。
struct RequestPortFactory {
    provider: TraitProvider<dyn RequestPort>,
}

#[test]
fn trait_provider_uses_explicit_scope_and_rejects_foreign_scope() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<
            RequestPortImpl,
            RequestScope,
            _,
        >(|_| RequestPortImpl))
        .expect("request target");
    registry
        .bind(TraitBinding::new::<dyn RequestPort, RequestPortImpl, _>(
            |port| port,
        ))
        .expect("request binding");
    registry
        .register(
            ComponentDefinition::try_singleton::<RequestPortFactory, _>(
                |resolver| -> Result<RequestPortFactory, BoxError> {
                    Ok(RequestPortFactory {
                        provider: resolver.trait_provider::<dyn RequestPort>()?,
                    })
                },
            )
            .depends_on_trait_provider::<dyn RequestPort>(),
        )
        .expect("request factory");
    let frozen = registry.build().expect("request trait graph");
    let container = frozen.container();
    let other_container = frozen.container();
    let factory = container
        .resolve::<RequestPortFactory>()
        .expect("request factory");

    assert!(matches!(
        factory.provider.get(),
        Err(ResolveError::ScopeNotActive { .. })
    ));
    let scope = container.open_scope::<RequestScope>();
    let first = factory.provider.get_in(&scope).expect("request port");
    let second = factory
        .provider
        .get_in(&scope)
        .expect("cached request port");
    assert!(Arc::ptr_eq(&first, &second));

    let foreign_scope = other_container.open_scope::<RequestScope>();
    assert!(matches!(
        factory.provider.get_in(&foreign_scope),
        Err(ResolveError::ScopeOwnerMismatch { .. })
    ));
}

/// 构造期重入保护使用的 Trait。
trait GuardedPort: Send + Sync {}

/// 构造期重入保护使用的实现。
struct GuardedPortImpl;

impl GuardedPort for GuardedPortImpl {}

/// 在工厂中观察结构化拒绝的组件。
struct GuardedConsumer;

#[test]
fn trait_provider_rejects_construction_time_reentry() {
    let rejected = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&rejected);
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::transient::<GuardedPortImpl, _>(|_| {
            GuardedPortImpl
        }))
        .expect("guarded target");
    registry
        .bind(TraitBinding::new::<dyn GuardedPort, GuardedPortImpl, _>(
            |port| port,
        ))
        .expect("guarded binding");
    registry
        .register(
            ComponentDefinition::try_singleton::<GuardedConsumer, _>(
                move |resolver| -> Result<GuardedConsumer, BoxError> {
                    let provider = resolver.trait_provider::<dyn GuardedPort>()?;
                    observed.store(
                        matches!(
                            provider.get(),
                            Err(ResolveError::ProviderUsedDuringConstruction { .. })
                        ),
                        Ordering::SeqCst,
                    );
                    Ok(GuardedConsumer)
                },
            )
            .depends_on_trait_provider::<dyn GuardedPort>(),
        )
        .expect("guarded consumer");
    let container = registry.build().expect("guarded graph").container();

    container
        .resolve::<GuardedConsumer>()
        .expect("consumer construction");
    assert!(rejected.load(Ordering::SeqCst));
}

/// 延迟环中由 Trait Provider 访问的一侧。
trait DeferredCyclePort: Send + Sync {
    /// 返回被目标组件持有的消费方。
    fn consumer(&self) -> &Arc<DeferredCycleConsumer>;
}

/// 先完成构造、随后才使用 Trait Provider 的消费方。
struct DeferredCycleConsumer {
    provider: TraitProvider<dyn DeferredCyclePort>,
}

/// 直接依赖已经构造完成消费方的 Trait 实现。
struct DeferredCycleTarget {
    consumer: Arc<DeferredCycleConsumer>,
}

impl DeferredCyclePort for DeferredCycleTarget {
    fn consumer(&self) -> &Arc<DeferredCycleConsumer> {
        &self.consumer
    }
}

#[test]
fn deferred_trait_provider_edge_breaks_a_constructor_cycle() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(
            ComponentDefinition::try_singleton::<DeferredCycleConsumer, _>(
                |resolver| -> Result<DeferredCycleConsumer, BoxError> {
                    Ok(DeferredCycleConsumer {
                        provider: resolver.trait_provider::<dyn DeferredCyclePort>()?,
                    })
                },
            )
            .depends_on_trait_provider::<dyn DeferredCyclePort>(),
        )
        .expect("cycle consumer");
    registry
        .register(
            ComponentDefinition::try_singleton::<DeferredCycleTarget, _>(
                |resolver| -> Result<DeferredCycleTarget, BoxError> {
                    Ok(DeferredCycleTarget {
                        consumer: resolver.resolve::<DeferredCycleConsumer>()?,
                    })
                },
            )
            .depends_on::<DeferredCycleConsumer>(),
        )
        .expect("cycle target");
    registry
        .bind(TraitBinding::new::<
            dyn DeferredCyclePort,
            DeferredCycleTarget,
            _,
        >(|target| target))
        .expect("cycle binding");
    let container = registry
        .build()
        .expect("deferred trait edge is not an eager cycle")
        .container();

    container.warm_up().expect("cycle singletons");
    let consumer = container
        .resolve::<DeferredCycleConsumer>()
        .expect("cycle consumer");
    let target = consumer.provider.get().expect("cycle target");
    assert!(Arc::ptr_eq(&consumer, target.consumer()));
}
