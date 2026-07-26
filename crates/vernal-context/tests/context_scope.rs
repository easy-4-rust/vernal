//! `ApplicationContext` 自定义 Scope 集成合同测试。

#[path = "context_scope_support/task_scope.rs"]
mod task_scope;
#[path = "context_scope_support/task_value.rs"]
mod task_value;

use task_scope::TaskScope;
use task_value::TaskValue;
use vernal_beans::{ComponentDefinition, ResolveError, ScopeState};
use vernal_context::VernalApplicationBuilder;

#[tokio::test]
async fn application_scope_joins_context_cancellation_tree() {
    let mut builder = VernalApplicationBuilder::current().expect("Tokio application builder");
    builder
        .register(ComponentDefinition::scoped::<TaskValue, TaskScope, _>(
            |_| TaskValue {
                value: "vernal-task",
            },
        ))
        .expect("task component registration");
    let context = builder.build().expect("application context");
    context.refresh().await.expect("context refresh");
    context.start().await.expect("context start");

    let scope = context.open_scope::<TaskScope>();
    let value = context
        .container()
        .resolve_in::<TaskValue>(&scope)
        .expect("task component");
    assert_eq!(value.value, "vernal-task");

    context.close().await.expect("context close");
    assert!(matches!(
        context.container().resolve_in::<TaskValue>(&scope),
        Err(ResolveError::ScopeUnavailable {
            state: ScopeState::Open,
            cancelled: true,
            ..
        })
    ));
    scope
        .close()
        .await
        .expect("scope owner still performs cleanup");
}
