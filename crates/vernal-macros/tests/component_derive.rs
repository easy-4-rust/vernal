//! Singleton 组件派生宏的运行时合同测试。

use std::sync::Arc;

use vernal_beans::{
    Component, ComponentProvider, Qualifier, RegistryBuilder, TraitBinding, TraitProvider,
};

/// 使用一个注入字段和一个默认字段的测试组件。
#[derive(vernal_macros::Component)]
struct GreetingService {
    name: Arc<String>,
    #[component(default)]
    retries: usize,
}

#[test]
fn derive_generates_factory_and_explicit_dependency_metadata() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(vernal_beans::ComponentDefinition::shared_value(
            String::from("vernal"),
        ))
        .expect("String dependency should register");
    registry
        .register(GreetingService::definition())
        .expect("derived component should register");
    let container = registry.build().expect("graph should be valid").container();

    let first = container
        .resolve::<GreetingService>()
        .expect("derived component should resolve");
    let second = container
        .resolve::<GreetingService>()
        .expect("derived singleton should resolve again");

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.name.as_str(), "vernal");
    assert_eq!(first.retries, 0);
}

/// 瞬时 Provider 目标。
struct GeneratedSequence(usize);

/// 验证派生宏直接识别 Provider 字段及 optional 属性。
#[derive(vernal_macros::Component)]
struct ProviderService {
    sequence: ComponentProvider<GeneratedSequence>,
    #[component(optional)]
    extension: ComponentProvider<OptionalGeneratedExtension>,
    #[component(qualifier = "blue")]
    named: ComponentProvider<NamedGeneratedExtension>,
}

/// 未安装时由可选 Provider 返回 `None` 的扩展类型。
struct OptionalGeneratedExtension;

/// 验证派生宏保留限定符的扩展类型。
struct NamedGeneratedExtension(&'static str);

#[test]
fn derive_generates_required_and_optional_provider_metadata() {
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = Arc::clone(&counter);
    let mut registry = RegistryBuilder::new();
    registry
        .register(vernal_beans::ComponentDefinition::transient::<
            GeneratedSequence,
            _,
        >(move |_| {
            GeneratedSequence(observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
        }))
        .expect("transient provider target");
    registry
        .register(
            vernal_beans::ComponentDefinition::transient::<NamedGeneratedExtension, _>(|_| {
                NamedGeneratedExtension("blue")
            })
            .qualified(Qualifier::new("blue").expect("valid qualifier")),
        )
        .expect("blue provider target");
    registry
        .register(
            vernal_beans::ComponentDefinition::transient::<NamedGeneratedExtension, _>(|_| {
                NamedGeneratedExtension("red")
            })
            .qualified(Qualifier::new("red").expect("valid qualifier")),
        )
        .expect("red provider target");
    registry
        .register(ProviderService::definition())
        .expect("derived provider service");
    let container = registry.build().expect("provider graph").container();

    container.warm_up().expect("singleton provider service");
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 0);
    let service = container
        .resolve::<ProviderService>()
        .expect("provider service");
    assert_eq!(service.sequence.get().expect("first transient").0, 0);
    assert_eq!(service.sequence.get().expect("second transient").0, 1);
    assert!(
        service
            .extension
            .get_if_available()
            .expect("optional extension")
            .is_none()
    );
    assert_eq!(service.named.get().expect("blue extension").0, "blue");
}

/// 派生宏 Trait Provider 使用的可替换端口。
trait GeneratedPort: Send + Sync {
    /// 返回实现名称。
    fn name(&self) -> &'static str;
}

/// 蓝色端口实现。
struct GeneratedBluePort;

impl GeneratedPort for GeneratedBluePort {
    fn name(&self) -> &'static str {
        "blue"
    }
}

/// 红色端口实现。
struct GeneratedRedPort;

impl GeneratedPort for GeneratedRedPort {
    fn name(&self) -> &'static str {
        "red"
    }
}

/// 未安装的可选 Trait 扩展点。
trait GeneratedOptionalPort: Send + Sync {}

/// 验证 `TraitProvider` 的 required、optional 与 qualifier 宏分支。
#[derive(vernal_macros::Component)]
struct TraitProviderService {
    primary: TraitProvider<dyn GeneratedPort>,
    #[component(optional)]
    extension: TraitProvider<dyn GeneratedOptionalPort>,
    #[component(qualifier = "blue")]
    named: TraitProvider<dyn GeneratedPort>,
    #[component(optional, qualifier = "missing")]
    missing_named: TraitProvider<dyn GeneratedPort>,
}

#[test]
fn derive_generates_trait_provider_binding_metadata() {
    let blue = Qualifier::new("blue").expect("valid qualifier");
    let red = Qualifier::new("red").expect("valid qualifier");
    let mut registry = RegistryBuilder::new();
    registry
        .register(vernal_beans::ComponentDefinition::transient::<
            GeneratedBluePort,
            _,
        >(|_| GeneratedBluePort))
        .expect("blue target");
    registry
        .register(vernal_beans::ComponentDefinition::transient::<
            GeneratedRedPort,
            _,
        >(|_| GeneratedRedPort))
        .expect("red target");
    registry
        .bind(
            TraitBinding::new::<dyn GeneratedPort, GeneratedBluePort, _>(|port| port)
                .qualified(blue)
                .primary(),
        )
        .expect("blue binding");
    registry
        .bind(
            TraitBinding::new::<dyn GeneratedPort, GeneratedRedPort, _>(|port| port).qualified(red),
        )
        .expect("red binding");
    registry
        .register(TraitProviderService::definition())
        .expect("derived trait provider service");
    let container = registry.build().expect("trait provider graph").container();

    let service = container
        .resolve::<TraitProviderService>()
        .expect("trait provider service");
    assert_eq!(service.primary.get().expect("primary port").name(), "blue");
    assert_eq!(service.named.get().expect("named port").name(), "blue");
    assert!(
        service
            .extension
            .get_if_available()
            .expect("optional trait")
            .is_none()
    );
    assert!(
        service
            .missing_named
            .get_if_available()
            .expect("optional named trait")
            .is_none()
    );
}
