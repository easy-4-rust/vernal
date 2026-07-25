//! `Option<Arc<T>>` 与 `Option<Arc<dyn Trait>>` 派生注入合同测试。

mod optional_component_support;

use std::sync::Arc;

use optional_component_support::{
    OptionalDerivedPort, OptionalDerivedService, OptionalNativeClient,
};
use vernal_ioc::{Component, ComponentDefinition, Qualifier, RegistryBuilder, TraitBinding};

#[test]
fn derive_uses_option_type_as_eager_optional_dependency_contract() {
    let client = Arc::new(OptionalNativeClient::new("tokio-native"));
    let blue = Qualifier::new("blue").expect("测试 qualifier 应合法");
    let red = Qualifier::new("red").expect("测试 qualifier 应合法");
    let missing = Qualifier::new("missing").expect("测试 qualifier 应合法");
    let mut registry = RegistryBuilder::new();
    registry
        .register_all([
            ComponentDefinition::shared_arc(Arc::clone(&client)),
            ComponentDefinition::shared_value(7_u32).qualified(blue),
            ComponentDefinition::shared_value(9_u32).qualified(red),
            OptionalDerivedService::definition(),
        ])
        .expect("派生组件和原生对象应原子注册");
    registry
        .bind(
            TraitBinding::new::<dyn OptionalDerivedPort, OptionalNativeClient, _>(|client| client)
                .primary(),
        )
        .expect("主 Trait Binding 应注册");

    let registry = registry.build().expect("可选依赖图应合法");
    let definition = registry
        .definitions()
        .iter()
        .find(|definition| {
            definition.key().type_name() == std::any::type_name::<OptionalDerivedService>()
        })
        .expect("派生定义应存在");
    assert!(
        definition
            .dependencies()
            .iter()
            .any(|dependency| dependency.to_string().starts_with("optional<"))
    );
    assert!(
        definition
            .dependencies()
            .iter()
            .any(|dependency| dependency.qualifier() == Some(&missing))
    );

    let service = registry
        .container()
        .resolve::<OptionalDerivedService>()
        .expect("派生可选组件应解析");
    assert!(Arc::ptr_eq(
        service.client().expect("客户端应存在"),
        &client
    ));
    assert_eq!(
        service.port().expect("Trait 端口应存在").name(),
        "tokio-native"
    );
    assert!(service.missing_number().is_none());
    assert_eq!(**service.named_number().expect("blue 数值应存在"), 7);
    assert!(service.missing_named_port().is_none());
}

#[test]
fn derive_allows_every_optional_candidate_to_be_absent() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(OptionalDerivedService::definition())
        .expect("派生定义应注册");

    let service = registry
        .build()
        .expect("全部零候选仍应通过依赖图")
        .container()
        .resolve::<OptionalDerivedService>()
        .expect("组件应以 None 完成构造");
    assert!(service.client().is_none());
    assert!(service.port().is_none());
    assert!(service.missing_number().is_none());
    assert!(service.named_number().is_none());
    assert!(service.missing_named_port().is_none());
}
