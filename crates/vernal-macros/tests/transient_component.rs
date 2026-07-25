//! Transient 组件派生宏的运行时合同测试。

use std::sync::Arc;

use vernal_beans::{Component, RegistryBuilder};

/// 验证宏生成 Transient 定义的无依赖组件。
#[derive(vernal_macros::Component)]
#[component(scope = "transient")]
struct RequestMarker;

#[test]
fn transient_scope_creates_a_new_instance_for_each_resolution() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(RequestMarker::definition())
        .expect("derived transient should register");
    let container = registry.build().expect("graph should be valid").container();

    let first = container
        .resolve::<RequestMarker>()
        .expect("first transient should resolve");
    let second = container
        .resolve::<RequestMarker>()
        .expect("second transient should resolve");

    assert!(!Arc::ptr_eq(&first, &second));
}
