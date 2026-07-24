//! 类型化自定义 Scope 组件派生合同测试。

#[path = "custom_scope_support/request_scope.rs"]
mod request_scope;
#[path = "custom_scope_support/scoped_greeting.rs"]
mod scoped_greeting;

use std::sync::Arc;

use request_scope::RequestScope;
use scoped_greeting::ScopedGreeting;
use vernal_ioc::{Component, RegistryBuilder, ResolveError};

#[tokio::test]
async fn derive_generates_type_driven_custom_scope_definition() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(vernal_ioc::ComponentDefinition::shared_value(String::from(
            "grow components",
        )))
        .expect("message definition");
    registry
        .register(ScopedGreeting::definition())
        .expect("derived scoped definition");
    let container = registry.build().expect("valid graph").container();

    assert!(matches!(
        container.resolve::<ScopedGreeting>(),
        Err(ResolveError::ScopeNotActive { .. })
    ));
    let scope = container.open_scope::<RequestScope>();
    let first = container
        .resolve_in::<ScopedGreeting>(&scope)
        .expect("scoped component");
    let second = container
        .resolve_in::<ScopedGreeting>(&scope)
        .expect("cached scoped component");

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.message.as_str(), "grow components");
    scope.close().await.expect("scope close");
}
