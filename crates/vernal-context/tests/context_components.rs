//! Vernal 高层应用建造器与 Context 内建组件合同测试。

use std::{sync::Arc, time::Duration};

use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;
use vernal_aop::{InvocationPlanCatalog, LocalInvocationPlanCatalog, Operation};
use vernal_context::{
    ApplicationBuildError, ApplicationEnvironment, EventBus, Lifecycle, LifecycleExecutionPolicy,
    ManagedTaskSupervisor, MapPropertySource, ScopeCleanupPolicy, SystemShutdownSignalListener,
    TaskShutdownPolicy, VernalApplicationBuilder,
};
use vernal_core::BoxError;
use vernal_beans::ComponentDefinition;

/// 模拟同时使用全部 Context 内建资源的业务服务。
struct RuntimeAwareService {
    runtime: Arc<Handle>,
    cancellation: Arc<CancellationToken>,
    events: Arc<EventBus>,
    managed_tasks: Arc<ManagedTaskSupervisor>,
    task_shutdown_policy: Arc<TaskShutdownPolicy>,
    lifecycle_execution_policy: Arc<LifecycleExecutionPolicy>,
    shutdown_signals: Arc<SystemShutdownSignalListener>,
    environment: Arc<ApplicationEnvironment>,
    scope_cleanup_policy: Arc<ScopeCleanupPolicy>,
    invocation_plans: Arc<InvocationPlanCatalog>,
    local_invocation_plans: Arc<LocalInvocationPlanCatalog>,
}

impl Lifecycle for RuntimeAwareService {}

/// 校验任务监督器与停机策略在 `IoC` 服务和 Context 中保持同一实例。
fn assert_managed_task_components(
    service: &RuntimeAwareService,
    context: &vernal_context::ApplicationContext,
) {
    assert!(std::ptr::eq(
        service.managed_tasks.as_ref(),
        context
            .managed_tasks()
            .expect("managed task supervisor")
            .as_ref()
    ));
    assert_eq!(
        *service.task_shutdown_policy,
        TaskShutdownPolicy::new(Duration::from_secs(9), Duration::from_secs(2))
    );
    assert!(std::ptr::eq(
        service.task_shutdown_policy.as_ref(),
        context.task_shutdown_policy()
    ));
}

/// 校验生命周期执行策略作为普通 `IoC` 原生组件保持同一实例。
fn assert_lifecycle_execution_component(
    service: &RuntimeAwareService,
    context: &vernal_context::ApplicationContext,
) {
    assert_eq!(
        *service.lifecycle_execution_policy,
        LifecycleExecutionPolicy::new(
            Duration::from_secs(11),
            Duration::from_secs(12),
            Duration::from_secs(13),
            Duration::from_secs(3),
        )
    );
    assert!(std::ptr::eq(
        service.lifecycle_execution_policy.as_ref(),
        context.lifecycle_execution_policy()
    ));
}

/// 配置并校验 Context-local 应用环境的 `IoC` 身份与类型化读取。
fn configure_environment(application: &mut VernalApplicationBuilder) {
    application
        .environment()
        .active_profile("production")
        .expect("valid active profile")
        .add_last(Arc::new(
            MapPropertySource::new(
                "application",
                [("service.port", "8088"), ("service.name", "vernal")],
            )
            .expect("valid property source"),
        ))
        .expect("unique property source");
}

/// 校验业务组件与 Context 使用完全相同的应用环境。
fn assert_environment_component(
    service: &RuntimeAwareService,
    context: &vernal_context::ApplicationContext,
) {
    assert!(std::ptr::eq(
        service.environment.as_ref(),
        context.environment()
    ));
    assert_eq!(
        service
            .environment
            .require::<u16>("service.port")
            .expect("typed property"),
        8088
    );
    assert!(
        service
            .environment
            .is_profile_active("production")
            .expect("valid profile")
    );
}

/// 构造声明全部十一类框架内建依赖的业务组件定义。
fn runtime_aware_definition() -> ComponentDefinition {
    ComponentDefinition::try_singleton::<RuntimeAwareService, _>(
        |resolver| -> Result<RuntimeAwareService, BoxError> {
            Ok(RuntimeAwareService {
                runtime: resolver.resolve::<Handle>()?,
                cancellation: resolver.resolve::<CancellationToken>()?,
                events: resolver.resolve::<EventBus>()?,
                managed_tasks: resolver.resolve::<ManagedTaskSupervisor>()?,
                task_shutdown_policy: resolver.resolve::<TaskShutdownPolicy>()?,
                lifecycle_execution_policy: resolver.resolve::<LifecycleExecutionPolicy>()?,
                shutdown_signals: resolver.resolve::<SystemShutdownSignalListener>()?,
                environment: resolver.resolve::<ApplicationEnvironment>()?,
                scope_cleanup_policy: resolver.resolve::<ScopeCleanupPolicy>()?,
                invocation_plans: resolver.resolve::<InvocationPlanCatalog>()?,
                local_invocation_plans: resolver.resolve::<LocalInvocationPlanCatalog>()?,
            })
        },
    )
    .depends_on::<Handle>()
    .depends_on::<CancellationToken>()
    .depends_on::<EventBus>()
    .depends_on::<ManagedTaskSupervisor>()
    .depends_on::<TaskShutdownPolicy>()
    .depends_on::<LifecycleExecutionPolicy>()
    .depends_on::<SystemShutdownSignalListener>()
    .depends_on::<ApplicationEnvironment>()
    .depends_on::<ScopeCleanupPolicy>()
    .depends_on::<InvocationPlanCatalog>()
    .depends_on::<LocalInvocationPlanCatalog>()
}

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
    application.scope_cleanup_policy(ScopeCleanupPolicy::bounded(Duration::from_secs(7)));
    application.task_shutdown_policy(TaskShutdownPolicy::new(
        Duration::from_secs(9),
        Duration::from_secs(2),
    ));
    application.lifecycle_execution_policy(LifecycleExecutionPolicy::new(
        Duration::from_secs(11),
        Duration::from_secs(12),
        Duration::from_secs(13),
        Duration::from_secs(3),
    ));
    configure_environment(&mut application);
    application.operation(operation.clone());
    application.lifecycle::<RuntimeAwareService>();
    application
        .register(runtime_aware_definition())
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
    assert_managed_task_components(&service, &context);
    assert_lifecycle_execution_component(&service, &context);
    assert!(std::ptr::eq(
        service.shutdown_signals.as_ref(),
        context.shutdown_signal_listener()
    ));
    assert_environment_component(&service, &context);
    assert_eq!(
        service.scope_cleanup_policy.timeout(),
        Some(Duration::from_secs(7))
    );
    assert!(std::ptr::eq(
        service.scope_cleanup_policy.as_ref(),
        context.scope_cleanup_policy()
    ));
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
