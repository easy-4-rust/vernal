//! Vernal 生命周期执行预算、Tokio abort 与逆序回滚合同测试。

#[path = "lifecycle_support/timeout_lifecycle.rs"]
mod timeout_lifecycle;
#[path = "lifecycle_support/timeout_stop_lifecycle.rs"]
mod timeout_stop_lifecycle;

use std::{sync::Arc, time::Duration};

use timeout_lifecycle::TimeoutLifecycle;
use timeout_stop_lifecycle::TimeoutStopLifecycle;
use tokio::sync::Mutex;
use vernal_context::{
    ContextError, ContextState, LifecycleExecutionPolicy, LifecyclePhase, VernalApplicationBuilder,
};
use vernal_beans::{ComponentDefinition, Qualifier};

const SHORT_TIMEOUT: Duration = Duration::from_millis(100);
const SETTLEMENT_TIMEOUT: Duration = Duration::from_secs(1);

/// 创建仅缩短指定生命周期阶段的测试策略。
fn timeout_policy() -> LifecycleExecutionPolicy {
    LifecycleExecutionPolicy::new(
        SHORT_TIMEOUT,
        SHORT_TIMEOUT,
        SHORT_TIMEOUT,
        SETTLEMENT_TIMEOUT,
    )
}

/// 使用高层建造器安装单个超时组件，使策略同时进入 Context 和 `IoC`。
fn timeout_context(component: Arc<TimeoutLifecycle>) -> vernal_context::ApplicationContext {
    let mut application =
        VernalApplicationBuilder::current().expect("Tokio runtime should be available");
    application.lifecycle_execution_policy(timeout_policy());
    application.lifecycle::<TimeoutLifecycle>();
    application
        .register(ComponentDefinition::shared_arc(component))
        .expect("timeout lifecycle definition");
    application.build().expect("timeout context should build")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn initialize_timeout_aborts_hook_and_rolls_back_component() {
    let component = Arc::new(TimeoutLifecycle::blocking_initialize());
    let context = timeout_context(Arc::clone(&component));

    let error = context
        .refresh()
        .await
        .expect_err("initialize should exceed policy");
    assert!(matches!(
        error,
        ContextError::LifecycleTimeout {
            phase: LifecyclePhase::Initialize,
            timeout: SHORT_TIMEOUT,
            abort_settled: true,
            ..
        }
    ));
    assert!(component.stopped());
    assert!(context.cancellation_token().is_cancelled());
    assert_eq!(context.state().await, ContextState::Closed);
    assert_eq!(context.lifecycle_execution_policy(), &timeout_policy());
    assert!(
        context
            .startup_report()
            .await
            .warnings()
            .iter()
            .any(|warning| warning == "context.lifecycle-hook.timeout")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn start_timeout_aborts_hook_and_rolls_back_refreshed_component() {
    let component = Arc::new(TimeoutLifecycle::blocking_start());
    let context = timeout_context(Arc::clone(&component));
    context.refresh().await.expect("initialize should succeed");

    let error = context
        .start()
        .await
        .expect_err("start should exceed policy");
    assert!(matches!(
        error,
        ContextError::LifecycleTimeout {
            phase: LifecyclePhase::Start,
            timeout: SHORT_TIMEOUT,
            abort_settled: true,
            ..
        }
    ));
    assert!(component.stopped());
    assert_eq!(context.state().await, ContextState::Closed);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_timeout_does_not_abandon_remaining_reverse_shutdown() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let dependency = Qualifier::new("dependency").expect("valid qualifier");
    let dependent = Qualifier::new("dependent").expect("valid qualifier");
    let dependency_component = Arc::new(TimeoutStopLifecycle::completing(
        "dependency:stop",
        Arc::clone(&events),
    ));
    let dependent_component = Arc::new(TimeoutStopLifecycle::blocking(
        "dependent:stop",
        Arc::clone(&events),
    ));

    let mut application =
        VernalApplicationBuilder::current().expect("Tokio runtime should be available");
    application.lifecycle_execution_policy(timeout_policy());
    application
        .lifecycle_qualified::<TimeoutStopLifecycle>(dependent.clone())
        .lifecycle_qualified::<TimeoutStopLifecycle>(dependency.clone());
    application
        .register(
            ComponentDefinition::shared_arc(dependency_component).qualified(dependency.clone()),
        )
        .expect("dependency lifecycle");
    application
        .register(
            ComponentDefinition::shared_arc(dependent_component)
                .qualified(dependent.clone())
                .depends_on_qualified::<TimeoutStopLifecycle>(dependency),
        )
        .expect("dependent lifecycle");
    let context = application.build().expect("qualified lifecycle context");
    context.refresh().await.expect("refresh should succeed");
    context.start().await.expect("start should succeed");

    let error = context
        .close()
        .await
        .expect_err("blocking stop should exceed policy");
    assert!(matches!(
        error,
        ContextError::LifecycleTimeout {
            phase: LifecyclePhase::Stop,
            timeout: SHORT_TIMEOUT,
            abort_settled: true,
            ..
        }
    ));
    assert_eq!(context.state().await, ContextState::Closed);
    assert_eq!(*events.lock().await, ["dependent:stop", "dependency:stop"]);
}
