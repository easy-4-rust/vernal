//! Singleton 组件派生宏的运行时合同测试。

use std::sync::Arc;

use vernal_ioc::{Component, RegistryBuilder};

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
