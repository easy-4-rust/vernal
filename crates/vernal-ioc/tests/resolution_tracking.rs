//! Container 成功解析追踪与实例隔离合同测试。

#[path = "custom_scope_support/request_scope.rs"]
mod request_scope;

use request_scope::RequestScope;
use vernal_ioc::{ComponentDefinition, RegistryBuilder, ResolveError};

#[test]
fn successful_resolution_removes_definition_from_deterministic_unused_snapshot() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::transient(|_| 42_u32))
        .expect("u32 definition");
    builder
        .register(ComponentDefinition::transient(|_| 84_u64))
        .expect("u64 definition");
    let container = builder.build().expect("registry").container();

    let initially_unused = container.unused_definitions();
    assert_eq!(initially_unused.len(), 2);
    assert!(initially_unused[0].ends_with("u32"));
    assert!(initially_unused[1].ends_with("u64"));

    assert_eq!(*container.resolve::<u64>().expect("u64 resolution"), 84);
    let after_u64 = container.unused_definitions();
    assert_eq!(after_u64.len(), 1);
    assert!(after_u64[0].ends_with("u32"));

    assert_eq!(*container.resolve::<u32>().expect("u32 resolution"), 42);
    assert!(container.unused_definitions().is_empty());
}

#[test]
fn registries_share_metadata_but_containers_never_share_resolution_history() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::transient(|_| 42_u32))
        .expect("u32 definition");
    let registry = builder.build().expect("registry");
    let first = registry.container();
    let second = registry.container();

    first.resolve::<u32>().expect("first container resolution");

    assert!(first.unused_definitions().is_empty());
    assert_eq!(second.unused_definitions().len(), 1);
    assert!(second.unused_definitions()[0].ends_with("u32"));
}

#[tokio::test]
async fn failed_scope_resolution_does_not_hide_an_unused_definition() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::scoped::<u16, RequestScope, _>(|_| 7))
        .expect("scoped u16 definition");
    let container = builder.build().expect("registry").container();

    assert!(matches!(
        container.resolve::<u16>(),
        Err(ResolveError::ScopeNotActive { .. })
    ));
    assert_eq!(container.unused_definitions().len(), 1);
    assert!(container.unused_definitions()[0].ends_with("u16"));

    let scope = container.open_scope::<RequestScope>();
    assert_eq!(
        *container
            .resolve_in::<u16>(&scope)
            .expect("scoped resolution"),
        7
    );
    assert!(container.unused_definitions().is_empty());
    scope.close().await.expect("scope close");
}
