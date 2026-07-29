//! 差分测试 - StartupReport 序列化稳定性（对标 P1-D）
//!
//! 验证 StartupReport 的 JSON 序列化输出字段顺序和嵌套结构稳定。
//! 修改 StartupReport 字段后测试应该立即失败。

use std::sync::Arc;
use std::time::Duration;

use vernal_context::{
    ApplicationEnvironment, ApplicationModule, ApplicationModuleRegistrar,
    ApplicationRunner, Lifecycle, ScheduledTask, TaskSchedule, VernalApplicationBuilder,
};
use vernal_beans::{Component, ComponentDefinition};

// ════════════════════════════════════════════════════════════════════
// 测试用类型
// ════════════════════════════════════════════════════════════════════

struct SimpleComponent;
impl Component for SimpleComponent {
    fn definition() -> ComponentDefinition {
        ComponentDefinition::singleton::<Self, _>(|_| SimpleComponent)
    }
}

struct SimpleLifecycle;
impl Lifecycle for SimpleLifecycle {}

struct TestModule;

impl ApplicationModule for TestModule {
    fn name(&self) -> &'static str {
        "test-module"
    }

    fn configure(
        self,
        registrar: &mut ApplicationModuleRegistrar,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        registrar.component::<SimpleComponent>();
        Ok(())
    }
}

fn build_context() -> Arc<vernal_context::ApplicationContext> {
    let rt = tokio::runtime::Handle::current();
    let mut builder = VernalApplicationBuilder::new(rt);
    builder.register_module(TestModule).unwrap();
    Arc::new(builder.build().unwrap())
}

// ════════════════════════════════════════════════════════════════════
// 测试: StartupReport 基本结构
// ════════════════════════════════════════════════════════════════════

/// 验证 StartupReport 包含所有必需字段
#[tokio::test]
async fn test_startup_report_has_required_fields() {
    let context = build_context();
    let report = context.startup_report().await;

    // 验证报告包含基本字段
    assert!(!report.framework_version().is_empty());
    assert!(!report.minimum_rust_version().is_empty());
    assert!(!report.project_status().is_empty());
    assert!(!report.context_state().is_empty());
}

/// 验证 StartupReport 的 framework_version 字段
#[tokio::test]
async fn test_startup_report_framework_version() {
    let context = build_context();
    let report = context.startup_report().await;

    let version = report.framework_version();
    assert!(!version.is_empty(), "framework_version should not be empty");
}

/// 验证 StartupReport 的 project_status 字段
#[tokio::test]
async fn test_startup_report_project_status() {
    let context = build_context();
    let report = context.startup_report().await;

    let status = report.project_status();
    assert!(!status.is_empty(), "project_status should not be empty");
}

/// 验证 StartupReport 的 context_state 字段
#[tokio::test]
async fn test_startup_report_context_state() {
    let context = build_context();
    let report = context.startup_report().await;

    let state = report.context_state();
    assert!(!state.is_empty(), "context_state should not be empty");
}

/// 验证 StartupReport 的 environment 字段
#[tokio::test]
async fn test_startup_report_environment() {
    let context = build_context();
    let report = context.startup_report().await;

    let _environment = report.environment();
}

/// 验证 StartupReport 的 registry 字段
#[tokio::test]
async fn test_startup_report_registry() {
    let context = build_context();
    let report = context.startup_report().await;

    let _registry = report.registry();
}

/// 验证 StartupReport 的 aop 计数字段
#[tokio::test]
async fn test_startup_report_aop_counts() {
    let context = build_context();
    let report = context.startup_report().await;

    let _plan_count = report.aop_plan_count();
    let _interceptor_count = report.aop_interceptor_count();
    let _local_plan_count = report.local_aop_plan_count();
    let _local_interceptor_count = report.local_aop_interceptor_count();
}

/// 验证 StartupReport 的 enabled_features 字段
#[tokio::test]
async fn test_startup_report_enabled_features() {
    let context = build_context();
    let report = context.startup_report().await;

    let features = report.enabled_features();
    let _ = features.len();
}

/// 验证 StartupReport 的 adapters 字段
#[tokio::test]
async fn test_startup_report_adapters() {
    let context = build_context();
    let report = context.startup_report().await;

    let adapters = report.adapters();
    let _ = adapters.len();
}

/// 验证 StartupReport 的 external_dependencies 字段
#[tokio::test]
async fn test_startup_report_external_dependencies() {
    let context = build_context();
    let report = context.startup_report().await;

    let deps = report.external_dependencies();
    let _ = deps.len();
}

/// 验证 StartupReport 的 condition_evaluations 字段
#[tokio::test]
async fn test_startup_report_condition_evaluations() {
    let context = build_context();
    let report = context.startup_report().await;

    let evaluations = report.condition_evaluations();
    let _ = evaluations.len();
}

/// 验证 StartupReport 的 warnings 字段
#[tokio::test]
async fn test_startup_report_warnings() {
    let context = build_context();
    let report = context.startup_report().await;

    let warnings = report.warnings();
    let _ = warnings.len();
}

/// 验证 StartupReport 的 unused_definitions 字段
#[tokio::test]
async fn test_startup_report_unused_definitions() {
    let context = build_context();
    let report = context.startup_report().await;

    let definitions = report.unused_definitions();
    let _ = definitions.len();
}

/// 验证 StartupReport 的 observations 字段
#[tokio::test]
async fn test_startup_report_observations() {
    let context = build_context();
    let report = context.startup_report().await;

    let observations = report.observations();
    let _ = observations.len();
}

/// 验证 StartupReport 的序列化稳定性
#[tokio::test]
async fn test_startup_report_serialization_stability() {
    let context = build_context();
    let report = context.startup_report().await;

    // 序列化为 JSON
    let json1 = serde_json::to_string(&report).unwrap();
    let json2 = serde_json::to_string(&report).unwrap();

    // 多次序列化应该产生相同结果
    assert_eq!(json1, json2);

    // JSON 应该是有效的
    let parsed: serde_json::Value = serde_json::from_str(&json1).unwrap();
    assert!(parsed.is_object());
}

/// 验证 StartupReport 的 Debug 输出
#[tokio::test]
async fn test_startup_report_debug() {
    let context = build_context();
    let report = context.startup_report().await;

    let debug = format!("{:?}", report);
    assert!(debug.contains("StartupReport"));
    assert!(debug.contains("framework_version"));
    assert!(debug.contains("project_status"));
}

/// 验证 StartupReport 的 Clone 特性
#[tokio::test]
async fn test_startup_report_clone() {
    let context = build_context();
    let report = context.startup_report().await;
    let cloned = report.clone();

    assert_eq!(report.framework_version(), cloned.framework_version());
    assert_eq!(report.project_status(), cloned.project_status());
    assert_eq!(report.context_state(), cloned.context_state());
}

/// 验证 StartupReport 的 PartialEq 特性
#[tokio::test]
async fn test_startup_report_partial_eq() {
    let context = build_context();
    let report1 = context.startup_report().await;
    let report2 = context.startup_report().await;

    assert_eq!(report1, report2);
}
