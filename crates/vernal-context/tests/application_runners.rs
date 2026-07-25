//! `IoC` 托管一次性应用 Runner 合同测试。

mod application_runner_support;

use std::{error::Error, sync::Arc, time::Duration};

use application_runner_support::{
    FailingRunner, FirstRunner, LaterRunner, NeverRunner, PanickingRunner, ReadyRecorder,
    RunnerApplicationModule, SecondRunner, StartedLifecycle, StartupProbe,
};
use tokio::sync::broadcast;
use vernal_context::{
    ApplicationBuildError, ApplicationReadyEvent, ConditionalComponentModule, ContextError,
    ContextState, DiagnosticOutcome, DiagnosticPhase, LifecycleExecutionPolicy, ProfileCondition,
    VernalApplicationBuilder,
};
use vernal_beans::{ComponentDefinition, Qualifier};

/// 创建绑定当前 Tokio Runtime 的测试应用建造器。
fn application() -> VernalApplicationBuilder {
    VernalApplicationBuilder::new(tokio::runtime::Handle::current())
}

#[tokio::test]
async fn runners_follow_dependency_order_between_lifecycle_start_and_ready_fact() {
    let probe = Arc::new(StartupProbe::default());
    let mut application = application();
    application
        .register(ComponentDefinition::shared_arc(Arc::new(
            StartedLifecycle::new(Arc::clone(&probe)),
        )))
        .expect("lifecycle definition");
    // 故意先登记依赖方和 Runner 声明，证明执行顺序来自冻结依赖图而非调用顺序。
    application
        .register(
            ComponentDefinition::shared_arc(Arc::new(SecondRunner::new(Arc::clone(&probe))))
                .depends_on::<FirstRunner>(),
        )
        .expect("second runner definition");
    application
        .register(ComponentDefinition::shared_arc(Arc::new(FirstRunner::new(
            Arc::clone(&probe),
        ))))
        .expect("first runner definition");
    application
        .register(ComponentDefinition::shared_arc(Arc::new(
            ReadyRecorder::new(Arc::clone(&probe)),
        )))
        .expect("ready recorder definition");
    application
        .lifecycle::<StartedLifecycle>()
        .application_runner::<SecondRunner>()
        .application_runner::<FirstRunner>()
        .event_listener::<ApplicationReadyEvent, ReadyRecorder>();
    let context = application.build().expect("runner context");

    context.refresh().await.expect("context refresh");
    context.start().await.expect("context start");
    probe.wait_for_count(4).await;
    assert_eq!(
        probe.snapshot(),
        [
            "lifecycle:start",
            "runner:first",
            "runner:second",
            "event:ready"
        ]
    );
    assert_eq!(context.state().await, ContextState::Ready);
    let report = context.startup_report().await;
    assert!(
        report.observations().iter().any(|observation| {
            observation.phase() == DiagnosticPhase::ApplicationRunner
                && observation.outcome() == DiagnosticOutcome::Succeeded
                && observation.subject().contains("FirstRunner")
        }),
        "startup report must expose a redacted successful runner observation"
    );
    context.close().await.expect("context close");
}

#[tokio::test]
async fn runner_failure_short_circuits_later_work_rolls_back_and_redacts_defaults() {
    let probe = Arc::new(StartupProbe::default());
    let mut application = application();
    application
        .register(ComponentDefinition::shared_arc(Arc::new(
            StartedLifecycle::new(Arc::clone(&probe)),
        )))
        .expect("lifecycle definition");
    application
        .register(ComponentDefinition::shared_value(FailingRunner))
        .expect("failing runner definition");
    application
        .register(
            ComponentDefinition::shared_arc(Arc::new(LaterRunner::new(Arc::clone(&probe))))
                .depends_on::<FailingRunner>(),
        )
        .expect("later runner definition");
    application
        .lifecycle::<StartedLifecycle>()
        .application_runner::<FailingRunner>()
        .application_runner::<LaterRunner>();
    let context = application.build().expect("failing runner context");
    let mut ready = context.events().subscribe::<ApplicationReadyEvent>().await;

    context.refresh().await.expect("context refresh");
    let error = context.start().await.expect_err("runner must fail");
    assert!(matches!(
        error,
        ContextError::ApplicationRunnerFailed { .. }
    ));
    assert_eq!(
        probe.snapshot(),
        ["lifecycle:start", "lifecycle:stop"],
        "later runner must not execute and rollback must stop initialized components"
    );
    assert_eq!(context.state().await, ContextState::Closed);
    assert!(matches!(
        ready.try_recv(),
        Err(broadcast::error::TryRecvError::Empty)
    ));
    assert!(
        !error
            .to_string()
            .contains("secret-runner-downstream-response")
    );
    assert!(!format!("{error:?}").contains("secret-runner-downstream-response"));
    assert!(
        error
            .source()
            .and_then(Error::source)
            .expect("explicit runner root cause")
            .to_string()
            .contains("secret-runner-downstream-response")
    );
    let report = context.startup_report().await;
    assert!(
        report
            .warnings()
            .contains(&"context.application-runner.failed".to_owned())
    );
    assert!(report.observations().iter().any(|observation| {
        observation.phase() == DiagnosticPhase::ApplicationRunner
            && observation.outcome() == DiagnosticOutcome::Failed
            && !observation
                .subject()
                .contains("secret-runner-downstream-response")
    }));
}

#[tokio::test]
async fn runner_timeout_uses_start_budget_aborts_and_closes_context() {
    let mut application = application();
    application
        .register(ComponentDefinition::shared_value(NeverRunner))
        .expect("never runner definition");
    application.application_runner::<NeverRunner>();
    application.lifecycle_execution_policy(LifecycleExecutionPolicy::new(
        Duration::from_secs(1),
        Duration::from_millis(25),
        Duration::from_secs(1),
        Duration::from_secs(1),
    ));
    let context = application.build().expect("timeout runner context");

    context.refresh().await.expect("context refresh");
    let error = context.start().await.expect_err("runner must time out");
    assert!(matches!(
        error,
        ContextError::ApplicationRunnerTimeout {
            timeout,
            abort_settled: true,
            ..
        } if timeout == Duration::from_millis(25)
    ));
    assert_eq!(context.state().await, ContextState::Closed);
}

#[tokio::test]
async fn runner_panic_is_captured_redacted_and_rolled_back() {
    let mut application = application();
    application
        .register(ComponentDefinition::shared_value(PanickingRunner))
        .expect("panicking runner definition");
    application.application_runner::<PanickingRunner>();
    let context = application.build().expect("panicking runner context");

    context.refresh().await.expect("context refresh");
    let error = context
        .start()
        .await
        .expect_err("runner panic must surface");
    assert!(matches!(
        error,
        ContextError::ApplicationRunnerFailed { .. }
    ));
    assert!(!error.to_string().contains("secret-runner-panic-payload"));
    assert!(!format!("{error:?}").contains("secret-runner-panic-payload"));
    assert_eq!(context.state().await, ContextState::Closed);
}

#[tokio::test]
async fn runner_declarations_require_existing_singleton_and_unique_component_identity() {
    let mut missing = application();
    missing.application_runner::<FirstRunner>();
    assert!(matches!(
        missing.build(),
        Err(ApplicationBuildError::Context {
            source: ContextError::ApplicationRunnerDefinitionNotFound { .. }
        })
    ));

    let mut transient = application();
    transient
        .register(ComponentDefinition::transient::<FailingRunner, _>(|_| {
            FailingRunner
        }))
        .expect("transient runner definition");
    transient.application_runner::<FailingRunner>();
    assert!(matches!(
        transient.build(),
        Err(ApplicationBuildError::Context {
            source: ContextError::ApplicationRunnerScope {
                scope: "transient",
                ..
            }
        })
    ));

    let mut duplicate = application();
    duplicate
        .register(ComponentDefinition::shared_value(FailingRunner))
        .expect("duplicate runner definition");
    duplicate
        .application_runner::<FailingRunner>()
        .application_runner::<FailingRunner>();
    assert!(matches!(
        duplicate.build(),
        Err(ApplicationBuildError::Context {
            source: ContextError::DuplicateApplicationRunner { .. }
        })
    ));
}

#[tokio::test]
async fn application_module_and_condition_module_contribute_the_same_runner_contract() {
    let module_probe = Arc::new(StartupProbe::default());
    let mut module_application = application();
    module_application
        .register_module(RunnerApplicationModule::new(Arc::new(FirstRunner::new(
            Arc::clone(&module_probe),
        ))))
        .expect("runner application module");
    let module_context = module_application.build().expect("module runner context");
    module_context.refresh().await.expect("module refresh");
    module_context.start().await.expect("module start");
    assert_eq!(module_probe.snapshot(), ["runner:first"]);
    module_context.close().await.expect("module close");

    let conditional_probe = Arc::new(StartupProbe::default());
    let mut conditional = ConditionalComponentModule::new(
        "test.conditional-runner",
        ProfileCondition::any(["runner"]).expect("valid runner profile"),
    );
    conditional
        .register(ComponentDefinition::shared_arc(Arc::new(FirstRunner::new(
            Arc::clone(&conditional_probe),
        ))))
        .application_runner::<FirstRunner>();
    let mut conditional_application = application();
    conditional_application
        .environment()
        .active_profile("runner")
        .expect("active runner profile");
    conditional_application
        .register_conditional(conditional)
        .expect("conditional runner module");
    let conditional_context = conditional_application
        .build()
        .expect("conditional runner context");
    conditional_context
        .refresh()
        .await
        .expect("conditional refresh");
    conditional_context
        .start()
        .await
        .expect("conditional start");
    assert_eq!(conditional_probe.snapshot(), ["runner:first"]);
    let report = conditional_context.startup_report().await;
    assert_eq!(
        report.condition_evaluations()[0].application_runner_count(),
        1
    );
    conditional_context
        .close()
        .await
        .expect("conditional close");
}

#[tokio::test]
async fn qualified_runner_resolves_only_its_exact_singleton_identity() {
    let probe = Arc::new(StartupProbe::default());
    let qualifier = Qualifier::new("security-warmup").expect("valid runner qualifier");
    let mut application = application();
    application
        .register(
            ComponentDefinition::shared_arc(Arc::new(FirstRunner::new(Arc::clone(&probe))))
                .qualified(qualifier.clone()),
        )
        .expect("qualified runner definition");
    application.application_runner_qualified::<FirstRunner>(qualifier);
    let context = application.build().expect("qualified runner context");

    context.refresh().await.expect("qualified refresh");
    context.start().await.expect("qualified start");
    assert_eq!(probe.snapshot(), ["runner:first"]);
    context.close().await.expect("qualified close");
}
