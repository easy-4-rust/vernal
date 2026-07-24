//! Context 可序列化、只读、脱敏启动报告合同测试。

mod diagnostic_support;

use std::sync::Arc;

use diagnostic_support::{
    FailingLifecycle, HealthyLifecycle, PassThroughInterceptor, PassThroughLocalInterceptor,
};
use vernal_aop::{Advisor, LocalAdvisor, Operation};
use vernal_context::{
    ContextState, DiagnosticOutcome, DiagnosticState, MapPropertySource, VernalApplicationBuilder,
};
use vernal_ioc::ComponentDefinition;

#[tokio::test]
async fn report_tracks_registry_aop_subsystems_and_lifecycle_without_mutability() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder
        .environment()
        .active_profile("production")
        .expect("active profile")
        .add_last(Arc::new(
            MapPropertySource::new(
                "application",
                [
                    ("database.url", "postgres://secret-user:secret-password@db"),
                    ("service.port", "8080"),
                ],
            )
            .expect("property source"),
        ))
        .expect("environment source");
    builder
        .register(ComponentDefinition::singleton(|_| HealthyLifecycle))
        .expect("healthy lifecycle definition");
    builder
        .lifecycle::<HealthyLifecycle>()
        .advisor(Advisor::new(
            |_: &Operation| true,
            PassThroughInterceptor,
            10,
        ))
        .local_advisor(LocalAdvisor::new(
            |_: &Operation| true,
            PassThroughLocalInterceptor,
            10,
        ))
        .operation(Operation::new("diagnostic", "run"))
        .diagnostic_feature("tokio-runtime")
        .adapter_status("axum", DiagnosticState::Available)
        .external_dependency_status("redis-session", DiagnosticState::Degraded)
        .warning_code("preview-api");

    let context = builder.build().expect("valid application");
    context.refresh().await.expect("refresh");
    context.start().await.expect("start");

    let ready = context.startup_report().await;
    assert_eq!(ready.context_state(), ContextState::Ready.as_str());
    assert_eq!(ready.framework_version(), env!("CARGO_PKG_VERSION"));
    assert_eq!(ready.minimum_rust_version(), "1.85.0");
    assert_eq!(ready.environment().property_sources(), ["application"]);
    assert_eq!(ready.environment().active_profiles(), ["production"]);
    assert_eq!(ready.environment().default_profiles(), ["default"]);
    assert_eq!(ready.environment().effective_profiles(), ["production"]);
    assert_eq!(ready.aop_plan_count(), 1);
    assert_eq!(ready.aop_interceptor_count(), 1);
    assert_eq!(ready.local_aop_plan_count(), 1);
    assert_eq!(ready.local_aop_interceptor_count(), 1);
    assert_eq!(ready.enabled_features(), ["tokio-runtime"]);
    assert_eq!(ready.adapters()[0].name(), "axum");
    assert_eq!(
        ready.external_dependencies()[0].state(),
        DiagnosticState::Degraded
    );
    assert_eq!(ready.warnings(), ["preview-api"]);
    assert!(ready.unused_definitions().is_empty());
    assert_eq!(ready.registry().summary().definition_count(), 12);
    assert_eq!(ready.observations().len(), 4);
    assert!(
        ready
            .observations()
            .iter()
            .all(|observation| observation.outcome() == DiagnosticOutcome::Succeeded)
    );

    // 运行期诊断入口只接收静态代码，并在同一 Context 内排序、去重；已经取得的
    // ready 快照仍保持只读，不会被后续告警反向修改。
    context
        .record_runtime_warning("web.request-scope.cleanup-failed")
        .await;
    context.record_runtime_warning("preview-api").await;
    let warned = context.startup_report().await;
    assert_eq!(
        warned.warnings(),
        ["preview-api", "web.request-scope.cleanup-failed"]
    );
    assert_eq!(ready.warnings(), ["preview-api"]);

    let json = serde_json::to_string(&ready).expect("serializable startup report");
    assert!(json.contains("\"context_state\":\"ready\""));
    assert!(json.contains("\"state\":\"degraded\""));
    assert!(json.contains("\"property_sources\":[\"application\"]"));
    assert!(!json.contains("0x"));
    assert!(!json.contains("database.url"));
    assert!(!json.contains("secret-user"));
    assert!(!json.contains("secret-password"));

    context.close().await.expect("close");
    let closed = context.startup_report().await;
    assert_eq!(closed.context_state(), ContextState::Closed.as_str());
    assert_eq!(closed.observations().len(), 5);
    assert_eq!(
        closed.warnings(),
        ["preview-api", "web.request-scope.cleanup-failed"]
    );

    // 先前取得的快照拥有独立值，Context 后续关闭不会反向修改它。
    assert_eq!(ready.context_state(), ContextState::Ready.as_str());
    assert_eq!(ready.observations().len(), 4);
}

#[tokio::test]
async fn failed_lifecycle_report_never_serializes_business_error_text() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder
        .register(ComponentDefinition::singleton(|_| FailingLifecycle))
        .expect("failing lifecycle definition");
    builder.lifecycle::<FailingLifecycle>();
    let context = builder.build().expect("valid application");

    context.refresh().await.expect_err("initialize must fail");
    let report = context.startup_report().await;
    assert_eq!(report.context_state(), ContextState::Closed.as_str());
    assert!(
        report
            .observations()
            .iter()
            .any(|observation| observation.outcome() == DiagnosticOutcome::Failed)
    );

    let json = serde_json::to_string(&report).expect("serializable failure report");
    assert!(json.contains("\"outcome\":\"failed\""));
    assert!(json.contains("FailingLifecycle"));
    assert!(!json.contains("super-secret"));
    assert!(!json.contains("postgres://"));
    assert!(!json.contains("initialize failed"));
}
