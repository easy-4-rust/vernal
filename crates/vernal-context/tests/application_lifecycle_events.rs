//! Context 刷新与就绪强类型事件合同测试。

mod application_lifecycle_event_support;

use std::{error::Error, sync::Arc, time::Duration};

use application_lifecycle_event_support::{
    ApplicationPhaseListener, ApplicationPhaseProbe, FailingInitializeLifecycle,
    FailingStartLifecycle, ReadyFailureListener,
};
use tokio::sync::broadcast;
use vernal_context::{
    ApplicationReadyEvent, ApplicationRefreshedEvent, ContextError, VernalApplicationBuilder,
};
use vernal_beans::ComponentDefinition;

/// 创建绑定当前 Tokio Runtime 的测试应用建造器。
fn application() -> VernalApplicationBuilder {
    VernalApplicationBuilder::new(tokio::runtime::Handle::current())
}

#[tokio::test]
async fn context_publishes_refreshed_then_ready_after_each_state_commit() {
    let probe = Arc::new(ApplicationPhaseProbe::default());
    let mut application = application();
    application
        .register(ComponentDefinition::shared_arc(Arc::new(
            ApplicationPhaseListener::new(Arc::clone(&probe)),
        )))
        .expect("application phase listener definition");
    application
        .event_listener::<ApplicationRefreshedEvent, ApplicationPhaseListener>()
        .event_listener::<ApplicationReadyEvent, ApplicationPhaseListener>();
    let context = application.build().expect("application phase context");

    context.refresh().await.expect("context refresh");
    probe.wait_for("refreshed").await;
    assert_eq!(probe.snapshot(), ["refreshed"]);

    context.start().await.expect("context start");
    probe.wait_for("ready").await;
    assert_eq!(probe.snapshot(), ["refreshed", "ready"]);
    context.close().await.expect("phase listener shutdown");
}

#[tokio::test]
async fn failed_refresh_never_publishes_a_false_refreshed_fact() {
    let mut application = application();
    application
        .register(ComponentDefinition::shared_value(
            FailingInitializeLifecycle,
        ))
        .expect("failing lifecycle definition");
    application.lifecycle::<FailingInitializeLifecycle>();
    let context = application.build().expect("failing refresh context");
    let mut refreshed = context
        .events()
        .subscribe::<ApplicationRefreshedEvent>()
        .await;

    assert!(context.refresh().await.is_err());
    assert!(matches!(
        refreshed.try_recv(),
        Err(broadcast::error::TryRecvError::Empty)
    ));
}

#[tokio::test]
async fn failed_start_never_publishes_a_false_ready_fact() {
    let mut application = application();
    application
        .register(ComponentDefinition::shared_value(FailingStartLifecycle))
        .expect("failing start lifecycle definition");
    application.lifecycle::<FailingStartLifecycle>();
    let context = application.build().expect("failing start context");
    let mut ready = context.events().subscribe::<ApplicationReadyEvent>().await;

    context.refresh().await.expect("context refresh");
    assert!(context.start().await.is_err());
    assert!(matches!(
        ready.try_recv(),
        Err(broadcast::error::TryRecvError::Empty)
    ));
}

#[tokio::test]
async fn ready_listener_failure_uses_managed_task_cancellation_and_redaction() {
    let mut application = application();
    application
        .register(ComponentDefinition::shared_value(ReadyFailureListener))
        .expect("ready failure listener definition");
    application.event_listener::<ApplicationReadyEvent, ReadyFailureListener>();
    let context = application.build().expect("ready listener context");

    context.refresh().await.expect("context refresh");
    context
        .start()
        .await
        .expect("ready state commits asynchronously");
    tokio::time::timeout(
        Duration::from_secs(2),
        context.cancellation_token().cancelled(),
    )
    .await
    .expect("ready listener failure should cancel application");

    let error = context
        .close()
        .await
        .expect_err("listener failure must surface");
    assert!(matches!(error, ContextError::ManagedTask { .. }));
    assert!(!error.to_string().contains("secret-ready-listener-response"));
    assert!(!format!("{error:?}").contains("secret-ready-listener-response"));
    assert!(
        error
            .source()
            .and_then(Error::source)
            .and_then(Error::source)
            .expect("explicit ready listener source")
            .to_string()
            .contains("secret-ready-listener-response")
    );
}
