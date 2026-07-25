//! Context 托管周期任务合同测试。

mod scheduled_task_support;

use std::{error::Error, sync::Arc, time::Duration};

use scheduled_task_support::{
    FailingScheduledTask, FixedDelayProbeTask, FixedRateProbeTask, PanickingScheduledTask,
    ScheduledTaskApplicationModule, ScheduledTaskProbe,
};
use vernal_context::{
    ApplicationBuildError, ConditionalComponentModule, ContextError, ContextState,
    DiagnosticOutcome, DiagnosticPhase, ManagedTaskError, ProfileCondition, TaskSchedule,
    TaskScheduleError, TaskScheduleMode, VernalApplicationBuilder,
};
use vernal_beans::{ComponentDefinition, Qualifier};

/// 创建绑定当前 Tokio Runtime 的测试应用建造器。
fn application() -> VernalApplicationBuilder {
    VernalApplicationBuilder::new(tokio::runtime::Handle::current())
}

#[test]
fn task_schedule_rejects_busy_loops_and_preserves_timing_mode() {
    assert_eq!(
        TaskSchedule::fixed_delay(Duration::ZERO),
        Err(TaskScheduleError::ZeroInterval)
    );
    assert_eq!(
        TaskSchedule::fixed_rate(Duration::ZERO),
        Err(TaskScheduleError::ZeroInterval)
    );

    let fixed_delay =
        TaskSchedule::fixed_delay_after(Duration::from_millis(3), Duration::from_millis(7))
            .expect("valid fixed delay");
    assert_eq!(fixed_delay.mode(), TaskScheduleMode::FixedDelay);
    assert_eq!(fixed_delay.initial_delay(), Duration::from_millis(3));
    assert_eq!(fixed_delay.interval(), Duration::from_millis(7));

    let fixed_rate =
        TaskSchedule::fixed_rate_after(Duration::from_millis(5), Duration::from_millis(11))
            .expect("valid fixed rate");
    assert_eq!(fixed_rate.mode(), TaskScheduleMode::FixedRate);
    assert_eq!(fixed_rate.initial_delay(), Duration::from_millis(5));
    assert_eq!(fixed_rate.interval(), Duration::from_millis(11));
}

#[tokio::test]
async fn fixed_delay_task_is_supervised_non_overlapping_and_cancelled_on_close() {
    let probe = Arc::new(ScheduledTaskProbe::default());
    let task = Arc::new(FixedDelayProbeTask::new(
        Arc::clone(&probe),
        TaskSchedule::fixed_delay(Duration::from_millis(5)).expect("valid schedule"),
        Duration::from_millis(15),
    ));
    let mut application = application();
    application
        .register(ComponentDefinition::shared_arc(task))
        .expect("fixed delay definition");
    application.scheduled_task::<FixedDelayProbeTask>();
    let context = application.build().expect("fixed delay context");

    context.refresh().await.expect("fixed delay refresh");
    context.start().await.expect("fixed delay start");
    probe.wait_for_executions(3).await;
    assert_eq!(probe.max_active(), 1, "one task must never overlap itself");
    assert_eq!(
        context
            .managed_tasks()
            .expect("managed task supervisor")
            .active_count(),
        1
    );
    let report = context.startup_report().await;
    assert!(report.observations().iter().any(|observation| {
        observation.phase() == DiagnosticPhase::ScheduledTaskActivation
            && observation.outcome() == DiagnosticOutcome::Succeeded
            && observation.subject().contains("FixedDelayProbeTask")
    }));
    context.close().await.expect("fixed delay close");
    assert_eq!(context.state().await, ContextState::Closed);
    assert_eq!(
        context
            .managed_tasks()
            .expect("managed task supervisor")
            .active_count(),
        0
    );
}

#[tokio::test]
async fn fixed_rate_task_skips_missed_ticks_without_reentrant_execution() {
    let probe = Arc::new(ScheduledTaskProbe::default());
    let task = Arc::new(FixedRateProbeTask::new(
        Arc::clone(&probe),
        TaskSchedule::fixed_rate(Duration::from_millis(2)).expect("valid schedule"),
        Duration::from_millis(12),
    ));
    let mut application = application();
    application
        .register(ComponentDefinition::shared_arc(task))
        .expect("fixed rate definition");
    application.scheduled_task::<FixedRateProbeTask>();
    let context = application.build().expect("fixed rate context");

    context.refresh().await.expect("fixed rate refresh");
    context.start().await.expect("fixed rate start");
    probe.wait_for_executions(3).await;
    assert_eq!(probe.max_active(), 1);
    context.close().await.expect("fixed rate close");
}

#[tokio::test]
async fn scheduled_task_activation_uses_dependency_plan_instead_of_registration_order() {
    let fixed_delay_probe = Arc::new(ScheduledTaskProbe::default());
    let fixed_rate_probe = Arc::new(ScheduledTaskProbe::default());
    let mut application = application();
    // 故意先注册依赖方并反向声明任务，证明激活顺序来自冻结依赖图。
    application
        .register(
            ComponentDefinition::shared_arc(Arc::new(FixedRateProbeTask::new(
                fixed_rate_probe,
                TaskSchedule::fixed_rate_after(Duration::from_secs(60), Duration::from_secs(60))
                    .expect("valid fixed rate"),
                Duration::ZERO,
            )))
            .depends_on::<FixedDelayProbeTask>(),
        )
        .expect("dependent scheduled task");
    application
        .register(ComponentDefinition::shared_arc(Arc::new(
            FixedDelayProbeTask::new(
                fixed_delay_probe,
                TaskSchedule::fixed_delay_after(Duration::from_secs(60), Duration::from_secs(60))
                    .expect("valid fixed delay"),
                Duration::ZERO,
            ),
        )))
        .expect("dependency scheduled task");
    application
        .scheduled_task::<FixedRateProbeTask>()
        .scheduled_task::<FixedDelayProbeTask>();
    let context = application.build().expect("ordered scheduled task context");

    context.refresh().await.expect("ordered task refresh");
    context.start().await.expect("ordered task start");
    let subjects = context
        .startup_report()
        .await
        .observations()
        .iter()
        .filter(|observation| observation.phase() == DiagnosticPhase::ScheduledTaskActivation)
        .map(|observation| observation.subject().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(subjects.len(), 2);
    assert!(subjects[0].contains("FixedDelayProbeTask"));
    assert!(subjects[1].contains("FixedRateProbeTask"));
    context.close().await.expect("ordered task close");
}

#[tokio::test]
async fn scheduled_task_failure_cancels_application_and_redacts_default_errors() {
    let mut application = application();
    application
        .register(ComponentDefinition::shared_value(FailingScheduledTask))
        .expect("failing task definition");
    application.scheduled_task::<FailingScheduledTask>();
    let context = application.build().expect("failing task context");

    context.refresh().await.expect("failing task refresh");
    context.start().await.expect("task activation succeeds");
    let error = context
        .run_until_cancelled()
        .await
        .expect_err("task failure must close context");
    assert!(matches!(
        error,
        ContextError::ManagedTask {
            source: ManagedTaskError::TaskFailed { .. }
        }
    ));
    assert_eq!(context.state().await, ContextState::Closed);
    assert!(
        !error
            .to_string()
            .contains("secret-scheduled-task-downstream-response")
    );
    assert!(
        error
            .source()
            .and_then(Error::source)
            .and_then(Error::source)
            .expect("explicit scheduled task root cause")
            .to_string()
            .contains("secret-scheduled-task-downstream-response")
    );
}

#[tokio::test]
async fn scheduled_task_panic_is_captured_and_context_still_closes() {
    let mut application = application();
    application
        .register(ComponentDefinition::shared_value(PanickingScheduledTask))
        .expect("panicking task definition");
    application.scheduled_task::<PanickingScheduledTask>();
    let context = application.build().expect("panicking task context");

    context.refresh().await.expect("panicking task refresh");
    context.start().await.expect("task activation succeeds");
    let error = context
        .run_until_cancelled()
        .await
        .expect_err("task panic must close context");
    assert!(matches!(
        error,
        ContextError::ManagedTask {
            source: ManagedTaskError::TaskPanicked { .. }
        }
    ));
    assert_eq!(context.state().await, ContextState::Closed);
}

#[tokio::test]
async fn scheduled_task_declarations_require_existing_singleton_and_unique_identity() {
    let mut missing = application();
    missing.scheduled_task::<FailingScheduledTask>();
    assert!(matches!(
        missing.build(),
        Err(ApplicationBuildError::Context {
            source: ContextError::ScheduledTaskDefinitionNotFound { .. }
        })
    ));

    let mut transient = application();
    transient
        .register(ComponentDefinition::transient::<FailingScheduledTask, _>(
            |_| FailingScheduledTask,
        ))
        .expect("transient task definition");
    transient.scheduled_task::<FailingScheduledTask>();
    assert!(matches!(
        transient.build(),
        Err(ApplicationBuildError::Context {
            source: ContextError::ScheduledTaskScope {
                scope: "transient",
                ..
            }
        })
    ));

    let mut duplicate = application();
    duplicate
        .register(ComponentDefinition::shared_value(FailingScheduledTask))
        .expect("duplicate task definition");
    duplicate
        .scheduled_task::<FailingScheduledTask>()
        .scheduled_task::<FailingScheduledTask>();
    assert!(matches!(
        duplicate.build(),
        Err(ApplicationBuildError::Context {
            source: ContextError::DuplicateScheduledTask { .. }
        })
    ));
}

#[tokio::test]
async fn module_condition_and_qualifier_paths_share_the_scheduled_task_contract() {
    let module_probe = Arc::new(ScheduledTaskProbe::default());
    let module_task = Arc::new(FixedDelayProbeTask::new(
        Arc::clone(&module_probe),
        TaskSchedule::fixed_delay(Duration::from_secs(1)).expect("valid module schedule"),
        Duration::ZERO,
    ));
    let mut module_application = application();
    module_application
        .register_module(ScheduledTaskApplicationModule::new(module_task))
        .expect("scheduled task application module");
    let module_context = module_application.build().expect("module task context");
    module_context.refresh().await.expect("module task refresh");
    module_context.start().await.expect("module task start");
    module_probe.wait_for_executions(1).await;
    module_context.close().await.expect("module task close");

    let conditional_probe = Arc::new(ScheduledTaskProbe::default());
    let mut conditional = ConditionalComponentModule::new(
        "test.conditional-scheduled-task",
        ProfileCondition::any(["scheduled"]).expect("valid scheduled profile"),
    );
    conditional
        .register(ComponentDefinition::shared_arc(Arc::new(
            FixedDelayProbeTask::new(
                Arc::clone(&conditional_probe),
                TaskSchedule::fixed_delay(Duration::from_secs(1))
                    .expect("valid conditional schedule"),
                Duration::ZERO,
            ),
        )))
        .scheduled_task::<FixedDelayProbeTask>();
    let mut conditional_application = application();
    conditional_application
        .environment()
        .active_profile("scheduled")
        .expect("active scheduled profile");
    conditional_application
        .register_conditional(conditional)
        .expect("conditional scheduled task");
    let conditional_context = conditional_application
        .build()
        .expect("conditional task context");
    conditional_context
        .refresh()
        .await
        .expect("conditional task refresh");
    conditional_context
        .start()
        .await
        .expect("conditional task start");
    conditional_probe.wait_for_executions(1).await;
    assert_eq!(
        conditional_context
            .startup_report()
            .await
            .condition_evaluations()[0]
            .scheduled_task_count(),
        1
    );
    conditional_context
        .close()
        .await
        .expect("conditional task close");

    let qualified_probe = Arc::new(ScheduledTaskProbe::default());
    let qualifier = Qualifier::new("security-cleanup").expect("valid task qualifier");
    let mut qualified_application = application();
    qualified_application
        .register(
            ComponentDefinition::shared_arc(Arc::new(FixedDelayProbeTask::new(
                Arc::clone(&qualified_probe),
                TaskSchedule::fixed_delay(Duration::from_secs(1))
                    .expect("valid qualified schedule"),
                Duration::ZERO,
            )))
            .qualified(qualifier.clone()),
        )
        .expect("qualified task definition");
    qualified_application.scheduled_task_qualified::<FixedDelayProbeTask>(qualifier);
    let qualified_context = qualified_application
        .build()
        .expect("qualified task context");
    qualified_context
        .refresh()
        .await
        .expect("qualified task refresh");
    qualified_context
        .start()
        .await
        .expect("qualified task start");
    qualified_probe.wait_for_executions(1).await;
    qualified_context
        .close()
        .await
        .expect("qualified task close");
}
