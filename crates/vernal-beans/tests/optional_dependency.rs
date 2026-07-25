//! 立即可选组件依赖的依赖图与运行时合同测试。

mod optional_dependency_support;

use std::sync::Arc;

use optional_dependency_support::{OptionalConsumer, OptionalPort, OptionalService};
use vernal_beans::{
    ComponentDefinition, GraphError, Qualifier, RegistryBuilder, ResolveError, TraitBinding,
};

#[test]
fn absent_optional_dependencies_resolve_to_none() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(OptionalConsumer::definition())
        .expect("可选消费组件定义应合法");

    let container = registry
        .build()
        .expect("零候选应满足可选依赖图")
        .container();
    let consumer = container
        .resolve::<OptionalConsumer>()
        .expect("可选消费组件应完成构造");

    assert!(consumer.service().is_none());
    assert!(consumer.port().is_none());
    assert!(consumer.named_number().is_none());
    assert!(consumer.named_port().is_none());
}

#[test]
fn present_optional_dependencies_keep_identity_binding_and_qualifier_semantics() {
    let service = Arc::new(OptionalService::new("vernal"));
    let blue = Qualifier::new("blue").expect("测试 qualifier 应合法");
    let red = Qualifier::new("red").expect("测试 qualifier 应合法");
    let named = Qualifier::new("named").expect("测试 qualifier 应合法");
    let mut registry = RegistryBuilder::new();
    registry
        .register_all([
            ComponentDefinition::shared_arc(Arc::clone(&service)),
            ComponentDefinition::shared_value(7_u32).qualified(blue.clone()),
            ComponentDefinition::shared_value(9_u32).qualified(red),
            OptionalConsumer::definition(),
        ])
        .expect("原生对象和消费组件应原子注册");
    registry
        .bind_all([
            TraitBinding::new::<dyn OptionalPort, OptionalService, _>(|service| service).primary(),
            TraitBinding::new::<dyn OptionalPort, OptionalService, _>(|service| service)
                .qualified(named),
        ])
        .expect("默认与命名 Trait Binding 应原子注册");

    let container = registry
        .build()
        .expect("存在候选时可选依赖仍应形成合法图")
        .container();
    let consumer = container
        .resolve::<OptionalConsumer>()
        .expect("消费组件应解析");

    assert!(Arc::ptr_eq(
        consumer.service().expect("具体服务应存在"),
        &service
    ));
    assert_eq!(consumer.port().expect("Trait 端口应存在").value(), "vernal");
    assert_eq!(**consumer.named_number().expect("blue 数值应存在"), 7);
    assert_eq!(
        consumer
            .named_port()
            .expect("命名 Trait 端口应存在")
            .value(),
        "vernal"
    );
}

#[test]
fn optional_dependency_does_not_hide_ambiguity() {
    let blue = Qualifier::new("blue").expect("测试 qualifier 应合法");
    let red = Qualifier::new("red").expect("测试 qualifier 应合法");
    let mut registry = RegistryBuilder::new();
    registry
        .register_all([
            ComponentDefinition::shared_value(OptionalService::new("blue")).qualified(blue),
            ComponentDefinition::shared_value(OptionalService::new("red")).qualified(red),
            OptionalConsumer::definition(),
        ])
        .expect("重复类型的命名定义本身合法");

    let error = registry
        .build()
        .expect_err("无命名可选依赖仍必须拒绝多候选");
    assert!(matches!(error, GraphError::AmbiguousDependency { .. }));
}

#[test]
fn optional_dependency_does_not_hide_construction_failure() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::try_singleton::<OptionalService, _>(
            |_| Err(std::io::Error::other("optional target failed").into()),
        ))
        .expect("失败工厂定义应注册");
    registry
        .register(OptionalConsumer::definition())
        .expect("消费组件定义应注册");

    let container = registry
        .build()
        .expect("工厂失败属于运行时解析阶段")
        .container();
    let Err(error) = container.resolve::<OptionalConsumer>() else {
        panic!("可选依赖不能把构造失败转换为 None");
    };
    assert!(matches!(error, ResolveError::Construction { .. }));
}
