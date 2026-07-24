//! 应用级 Tokio 任务监督、失败传播与停机合同测试。

#[path = "managed_task_support/task_owning_lifecycle.rs"]
mod task_owning_lifecycle;

use std::{future, io, sync::Arc, time::Duration};

use task_owning_lifecycle::TaskOwningLifecycle;
use tokio::{runtime::Handle, sync::Notify};
use tokio_util::sync::CancellationToken;
use vernal_context::{
    ContextError, ManagedTaskError, ManagedTaskSupervisor, TaskShutdownPolicy,
    VernalApplicationBuilder,
};
use vernal_ioc::ComponentDefinition;

#[tokio::test]
async fn task_failure_cancels_application_and_returns_one_shared_result() {
    let cancellation = Arc::new(CancellationToken::new());
    let supervisor =
        ManagedTaskSupervisor::new(Arc::new(Handle::current()), Arc::clone(&cancellation));
    supervisor
        .spawn("test.failing-worker", async {
            Err::<(), _>(io::Error::other("database password must stay private"))
        })
        .expect("task accepted");

    cancellation.cancelled().await;
    let policy = TaskShutdownPolicy::new(Duration::from_secs(1), Duration::from_secs(1));
    assert!(matches!(
        supervisor.shutdown(policy).await,
        Err(ManagedTaskError::TaskFailed {
            task: "test.failing-worker",
            ..
        })
    ));
    assert!(matches!(
        supervisor.shutdown(policy).await,
        Err(ManagedTaskError::TaskFailed {
            task: "test.failing-worker",
            ..
        })
    ));
    assert_eq!(supervisor.active_count(), 0);
    assert_eq!(supervisor.completed_count(), 1);
    assert!(!supervisor.is_accepting());
    assert!(matches!(
        supervisor.spawn("test.late-worker", async { Ok::<_, io::Error>(()) }),
        Err(ManagedTaskError::SpawnRejected {
            task: "test.late-worker"
        })
    ));
}

#[tokio::test]
async fn empty_task_name_is_rejected_before_spawning_user_future() {
    let cancellation = Arc::new(CancellationToken::new());
    let supervisor =
        ManagedTaskSupervisor::new(Arc::new(Handle::current()), Arc::clone(&cancellation));

    assert!(matches!(
        supervisor.spawn("  ", async { Ok::<_, io::Error>(()) }),
        Err(ManagedTaskError::InvalidName)
    ));
    assert_eq!(supervisor.active_count(), 0);
    assert!(supervisor.is_accepting());
}

#[tokio::test]
async fn cancelling_one_shutdown_waiter_never_abandons_managed_tasks() {
    let cancellation = Arc::new(CancellationToken::new());
    let supervisor =
        ManagedTaskSupervisor::new(Arc::new(Handle::current()), Arc::clone(&cancellation));
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let task_entered = Arc::clone(&entered);
    let task_release = Arc::clone(&release);
    supervisor
        .spawn("test.blocking-worker", async move {
            task_entered.notify_one();
            task_release.notified().await;
            Ok::<_, io::Error>(())
        })
        .expect("task accepted");
    entered.notified().await;

    let policy = TaskShutdownPolicy::new(Duration::from_secs(5), Duration::from_secs(1));
    let waiter = {
        let supervisor = Arc::clone(&supervisor);
        tokio::spawn(async move { supervisor.shutdown(policy).await })
    };
    cancellation.cancelled().await;
    waiter.abort();
    assert!(
        waiter
            .await
            .expect_err("waiter should be cancelled")
            .is_cancelled()
    );

    release.notify_one();
    supervisor
        .shutdown(policy)
        .await
        .expect("later waiter joins coordinator");
    assert_eq!(supervisor.active_count(), 0);
    assert_eq!(supervisor.completed_count(), 1);
}

#[tokio::test]
async fn shutdown_timeout_aborts_non_cooperative_task_and_still_settles() {
    let cancellation = Arc::new(CancellationToken::new());
    let supervisor =
        ManagedTaskSupervisor::new(Arc::new(Handle::current()), Arc::clone(&cancellation));
    supervisor
        .spawn("test.non-cooperative-worker", async {
            future::pending::<()>().await;
            #[allow(unreachable_code)]
            Ok::<_, io::Error>(())
        })
        .expect("task accepted");

    assert!(matches!(
        supervisor
            .shutdown(TaskShutdownPolicy::new(
                Duration::from_millis(1),
                Duration::from_secs(1),
            ))
            .await,
        Err(ManagedTaskError::ShutdownTimeout { remaining: 1, .. })
    ));
    assert_eq!(supervisor.active_count(), 0);
    assert_eq!(supervisor.completed_count(), 1);
}

#[tokio::test]
async fn panicking_task_is_captured_without_panicking_the_context_runtime() {
    let cancellation = Arc::new(CancellationToken::new());
    let supervisor =
        ManagedTaskSupervisor::new(Arc::new(Handle::current()), Arc::clone(&cancellation));
    supervisor
        .spawn("test.panicking-worker", async {
            panic!("test worker panic");
            #[allow(unreachable_code)]
            Ok::<_, io::Error>(())
        })
        .expect("task accepted");

    cancellation.cancelled().await;
    assert!(matches!(
        supervisor
            .shutdown(TaskShutdownPolicy::new(
                Duration::from_secs(1),
                Duration::from_secs(1),
            ))
            .await,
        Err(ManagedTaskError::TaskPanicked {
            task: "test.panicking-worker",
            ..
        })
    ));
}

#[tokio::test]
async fn context_drains_managed_tasks_before_stopping_lifecycle_components() {
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    builder
        .register(
            ComponentDefinition::try_singleton::<TaskOwningLifecycle, _>(
                |resolver| -> Result<_, vernal_core::BoxError> {
                    Ok(TaskOwningLifecycle::new(
                        resolver.resolve::<ManagedTaskSupervisor>()?,
                    ))
                },
            )
            .depends_on::<ManagedTaskSupervisor>(),
        )
        .expect("lifecycle component definition");
    builder.lifecycle::<TaskOwningLifecycle>();
    let context = builder.build().expect("context build");
    context.refresh().await.expect("context refresh");
    context.start().await.expect("context start");
    let component = context
        .container()
        .resolve::<TaskOwningLifecycle>()
        .expect("lifecycle component");

    context.close().await.expect("context close");
    assert!(component.task_completed());
    assert!(component.stop_observed_completion());
}

#[tokio::test]
async fn context_reports_task_failure_with_only_a_redacted_warning_code() {
    let context = VernalApplicationBuilder::new(Handle::current())
        .build()
        .expect("context build");
    context.refresh().await.expect("context refresh");
    context.start().await.expect("context start");
    context
        .managed_tasks()
        .expect("managed task supervisor")
        .spawn("test.secret-worker", async {
            Err::<(), _>(io::Error::other("secret token must not enter diagnostics"))
        })
        .expect("task accepted");
    context.cancellation_token().cancelled().await;

    assert!(matches!(
        context.close().await,
        Err(ContextError::ManagedTask {
            source: ManagedTaskError::TaskFailed {
                task: "test.secret-worker",
                ..
            }
        })
    ));
    let report = context.startup_report().await;
    assert_eq!(report.warnings(), ["context.managed-task.shutdown-failed"]);
    assert!(
        report
            .warnings()
            .iter()
            .all(|warning| !warning.contains("secret") && !warning.contains("token"))
    );
}
