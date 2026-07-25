//! Singleton 组件派生宏的运行时合同测试。

use std::sync::Arc;

use vernal_ioc::{Component, ComponentProvider, Qualifier, RegistryBuilder};

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
        .register(vernal_ioc::ComponentDefinition::shared_value(String::from(
            "vernal",
        )))
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
        .register(vernal_ioc::ComponentDefinition::transient::<
            GeneratedSequence,
            _,
        >(move |_| {
            GeneratedSequence(observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
        }))
        .expect("transient provider target");
    registry
        .register(
            vernal_ioc::ComponentDefinition::transient::<NamedGeneratedExtension, _>(|_| {
                NamedGeneratedExtension("blue")
            })
            .qualified(Qualifier::new("blue").expect("valid qualifier")),
        )
        .expect("blue provider target");
    registry
        .register(
            vernal_ioc::ComponentDefinition::transient::<NamedGeneratedExtension, _>(|_| {
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
