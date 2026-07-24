//! Vernal 高层应用建造器与 Context 内建组件合同测试。

use std::sync::Arc;

use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;
use vernal_aop::{InvocationPlanCatalog, LocalInvocationPlanCatalog, Operation};
use vernal_context::{ApplicationBuildError, EventBus, Lifecycle, VernalApplicationBuilder};
use vernal_core::BoxError;
use vernal_ioc::ComponentDefinition;

/// 模拟同时使用全部 Context 内建资源的业务服务。
struct RuntimeAwareService {
    runtime: Arc<Handle>,
    cancellation: Arc<CancellationToken>,
    events: Arc<EventBus>,
    invocation_plans: Arc<InvocationPlanCatalog>,
    local_invocation_plans: Arc<LocalInvocationPlanCatalog>,
}

impl Lifecycle for RuntimeAwareService {}

#[test]
fn current_builder_rejects_threads_without_a_tokio_runtime() {
    assert!(matches!(
        VernalApplicationBuilder::current(),
        Err(ApplicationBuildError::TokioRuntimeUnavailable { .. })
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn managed_context_injects_tokio_events_cancellation_and_aop_plans() {
    let operation = Operation::new("RuntimeAwareService", "execute");
    let mut application =
        VernalApplicationBuilder::current().expect("Tokio runtime should be available");
    application.operation(operation.clone());
    application.lifecycle::<RuntimeAwareService>();
    application
        .register(
            ComponentDefinition::try_singleton::<RuntimeAwareService, _>(
                |resolver| -> Result<RuntimeAwareService, BoxError> {
                    Ok(RuntimeAwareService {
                        runtime: resolver.resolve::<Handle>()?,
                        cancellation: resolver.resolve::<CancellationToken>()?,
                        events: resolver.resolve::<EventBus>()?,
                        invocation_plans: resolver.resolve::<InvocationPlanCatalog>()?,
                        local_invocation_plans: resolver.resolve::<LocalInvocationPlanCatalog>()?,
                    })
                },
            )
            .depends_on::<Handle>()
            .depends_on::<CancellationToken>()
            .depends_on::<EventBus>()
            .depends_on::<InvocationPlanCatalog>()
            .depends_on::<LocalInvocationPlanCatalog>(),
        )
        .expect("runtime-aware service definition should be valid");

    let context = application
        .build()
        .expect("managed application should build");
    context.refresh().await.expect("context should refresh");
    context.start().await.expect("context should start");
    let service = context
        .container()
        .resolve::<RuntimeAwareService>()
        .expect("runtime-aware service should resolve");

    assert_eq!(
        service
            .runtime
            .spawn(async { "tokio-component" })
            .await
            .expect("Tokio task should finish"),
        "tokio-component"
    );
    assert!(context.runtime_handle().is_some());
    assert!(std::ptr::eq(service.events.as_ref(), context.events()));
    assert!(std::ptr::eq(
        service.invocation_plans.as_ref(),
        context.invocation_plans()
    ));
    assert!(service.invocation_plans.get(&operation).is_some());
    assert!(std::ptr::eq(
        service.local_invocation_plans.as_ref(),
        context.local_invocation_plans()
    ));
    assert!(service.local_invocation_plans.get(&operation).is_some());

    let mut receiver = context.events().subscribe::<String>().await;
    assert_eq!(
        service.events.publish(String::from("context-event")).await,
        1
    );
    assert_eq!(
        receiver
            .recv()
            .await
            .expect("shared EventBus should deliver the event")
            .as_str(),
        "context-event"
    );

    assert!(!service.cancellation.is_cancelled());
    context.close().await.expect("context should close");
    assert!(service.cancellation.is_cancelled());
}
