//! Vernal 高层统一启动、失败诊断与取消安全合同测试。

mod application_launch_support;

use std::sync::Arc;

use application_launch_support::{
    BlockingLifecycle, LaunchProbe, RefreshFailureLifecycle, StartFailureLifecycle,
    SuccessfulLifecycle,
};
use vernal_beans::ComponentDefinition;
use vernal_context::{ApplicationLaunchError, ContextState, VernalApplicationBuilder};

/// 创建绑定当前 Tokio Runtime 的空应用建造器。
fn application_builder() -> VernalApplicationBuilder {
    VernalApplicationBuilder::current().expect("test runs inside Tokio")
}

#[tokio::test]
async fn launch_returns_shared_ready_context_and_close_remains_explicit() {
    let probe = LaunchProbe::new();
    let component_probe = Arc::clone(&probe);
    let mut builder = application_builder();
    builder
        .register(ComponentDefinition::singleton(move |_| {
            SuccessfulLifecycle::new(Arc::clone(&component_probe))
        }))
        .expect("successful component definition");
    builder.lifecycle::<SuccessfulLifecycle>();

    let context = builder.launch().await.expect("application should launch");

    assert_eq!(context.state().await, ContextState::Ready);
    assert!(context.container().resolve::<SuccessfulLifecycle>().is_ok());
    assert_eq!(
        probe.events().await,
        ["success:initialize", "success:start"]
    );

    context
        .close()
        .await
        .expect("explicit close should succeed");
    assert_eq!(context.state().await, ContextState::Closed);
    assert_eq!(
        probe.events().await,
        ["success:initialize", "success:start", "success:stop"]
    );
}

#[tokio::test]
async fn launch_preserves_build_failure_without_fabricating_startup_report() {
    let probe = LaunchProbe::new();
    let component_probe = Arc::clone(&probe);
    let mut builder = application_builder();
    builder
        .register(
            ComponentDefinition::singleton(move |_| {
                SuccessfulLifecycle::new(Arc::clone(&component_probe))
            })
            .depends_on::<String>(),
        )
        .expect("definition is structurally valid before graph freeze");

    let error = match builder.launch().await {
        Ok(_) => panic!("missing dependency should reject launch"),
        Err(error) => error,
    };

    assert!(matches!(error, ApplicationLaunchError::Build { .. }));
    assert_eq!(error.operation(), "build");
    assert!(error.startup_report().is_none());
    assert!(error.cleanup_error().is_none());
}

#[tokio::test]
async fn launch_refresh_failure_is_closed_and_reported_without_leaking_secret() {
    let probe = LaunchProbe::new();
    let component_probe = Arc::clone(&probe);
    let mut builder = application_builder();
    builder
        .register(ComponentDefinition::singleton(move |_| {
            RefreshFailureLifecycle::new(Arc::clone(&component_probe))
        }))
        .expect("refresh failure component definition");
    builder.lifecycle::<RefreshFailureLifecycle>();

    let error = match builder.launch().await {
        Ok(_) => panic!("initialize failure should reject launch"),
        Err(error) => error,
    };

    assert_eq!(error.operation(), "refresh");
    assert_eq!(
        error
            .startup_report()
            .expect("created context has a report")
            .context_state(),
        "closed"
    );
    assert!(error.cleanup_error().is_none());
    let report_json =
        serde_json::to_string(error.startup_report().expect("launch report")).expect("JSON report");
    assert!(!report_json.contains("refresh failure secret"));
    assert_eq!(probe.events().await, ["refresh-failure:initialize"]);
}

#[tokio::test]
async fn launch_start_failure_rolls_back_initialized_component_and_reports_closed() {
    let probe = LaunchProbe::new();
    let component_probe = Arc::clone(&probe);
    let mut builder = application_builder();
    builder
        .register(ComponentDefinition::singleton(move |_| {
            StartFailureLifecycle::new(Arc::clone(&component_probe))
        }))
        .expect("start failure component definition");
    builder.lifecycle::<StartFailureLifecycle>();

    let error = match builder.launch().await {
        Ok(_) => panic!("start failure should reject launch"),
        Err(error) => error,
    };

    assert_eq!(error.operation(), "start");
    assert_eq!(
        error
            .startup_report()
            .expect("created context has a report")
            .context_state(),
        "closed"
    );
    assert!(error.cleanup_error().is_none());
    let report_json =
        serde_json::to_string(error.startup_report().expect("launch report")).expect("JSON report");
    assert!(!report_json.contains("start failure secret"));
    assert_eq!(
        probe.events().await,
        [
            "start-failure:initialize",
            "start-failure:start",
            "start-failure:stop"
        ]
    );
}

#[tokio::test]
async fn cancelling_launch_waiter_finishes_startup_then_closes_orphan_context() {
    let probe = LaunchProbe::new();
    let component_probe = Arc::clone(&probe);
    let mut builder = application_builder();
    builder
        .register(ComponentDefinition::singleton(move |_| {
            BlockingLifecycle::new(Arc::clone(&component_probe))
        }))
        .expect("blocking component definition");
    builder.lifecycle::<BlockingLifecycle>();

    let mut launch = Box::pin(builder.launch());
    tokio::select! {
        _result = &mut launch => panic!("launch completed before initialize release"),
        () = probe.wait_for("blocking:initialize-enter") => {}
    }
    drop(launch);
    probe.release_initialize();
    probe.wait_for("blocking:stop").await;

    assert_eq!(
        probe.events().await,
        [
            "blocking:initialize-enter",
            "blocking:initialize-exit",
            "blocking:start",
            "blocking:stop"
        ]
    );
}
