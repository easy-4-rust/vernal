//! 覆盖率提升测试 - 覆盖 bind, bind_all, register_bundle 方法

use std::sync::Arc;
use std::time::Duration;

use vernal_context::{
    ApplicationRunner, Lifecycle, ScheduledTask, TaskSchedule,
    VernalApplicationBuilder, ApplicationContext,
};
use vernal_beans::{Component, ComponentDefinition, Qualifier, TraitBinding};

// ════════════════════════════════════════════════════════════════════
// 测试用类型
// ════════════════════════════════════════════════════════════════════

struct SimpleComponent;
impl Component for SimpleComponent {
    fn definition() -> ComponentDefinition {
        ComponentDefinition::singleton::<Self, _>(|_| SimpleComponent)
    }
}

struct TestLifecycle;
impl Lifecycle for TestLifecycle {}

struct TestRunner;
impl Component for TestRunner {
    fn definition() -> ComponentDefinition {
        ComponentDefinition::singleton::<Self, _>(|_| TestRunner)
    }
}
impl ApplicationRunner for TestRunner {
    type Error = std::io::Error;
    async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct TestTask;
impl Component for TestTask {
    fn definition() -> ComponentDefinition {
        ComponentDefinition::singleton::<Self, _>(|_| TestTask)
    }
}
impl ScheduledTask for TestTask {
    type Error = std::io::Error;
    fn schedule(&self) -> TaskSchedule {
        TaskSchedule::fixed_rate(Duration::from_secs(60)).unwrap()
    }
    async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct TestEvent;
struct TestEventListener;
impl Component for TestEventListener {
    fn definition() -> ComponentDefinition {
        ComponentDefinition::singleton::<Self, _>(|_| TestEventListener)
    }
}
impl vernal_context::ApplicationEventListener<TestEvent> for TestEventListener {
    type Error = std::io::Error;
    async fn on_event(&self, _: Arc<TestEvent>) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// 创建测试用 ApplicationContext
fn build_context() -> Arc<ApplicationContext> {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let _ = builder.register(SimpleComponent::definition());
    Arc::new(builder.build().unwrap())
}

// ════════════════════════════════════════════════════════════════════
// 测试: bind()
// ════════════════════════════════════════════════════════════════════

/// 验证 bind() 注册 Trait 绑定
#[tokio::test]
async fn test_bind() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let binding = TraitBinding::new::<dyn std::any::Any + Send + Sync, SimpleComponent, _>(|c| c);
    let result = builder.bind(binding);
    assert!(result.is_ok(), "bind should succeed");
}

// ════════════════════════════════════════════════════════════════════
// 测试: bind_all()
// ════════════════════════════════════════════════════════════════════

/// 验证 bind_all() 批量注册 Trait 绑定
#[tokio::test]
async fn test_bind_all() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let bindings = vec![
        TraitBinding::new::<dyn std::any::Any + Send + Sync, SimpleComponent, _>(|c| c),
    ];
    let result = builder.bind_all(bindings);
    assert!(result.is_ok(), "bind_all should succeed");
}

/// 验证 bind_all() 空迭代器
#[tokio::test]
async fn test_bind_all_empty() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let bindings: Vec<TraitBinding> = vec![];
    let result = builder.bind_all(bindings);
    assert!(result.is_ok(), "bind_all with empty iterator should succeed");
}

// ════════════════════════════════════════════════════════════════════
// 测试: register_bundle()
// ════════════════════════════════════════════════════════════════════

/// 验证 register_bundle() 原子注册组件和绑定
#[tokio::test]
async fn test_register_bundle() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let defs = vec![ComponentDefinition::singleton::<SimpleComponent, _>(|_| SimpleComponent)];
    let bindings = vec![
        TraitBinding::new::<dyn std::any::Any + Send + Sync, SimpleComponent, _>(|c| c),
    ];
    let result = builder.register_bundle(defs, bindings);
    assert!(result.is_ok(), "register_bundle should succeed");
}

/// 验证 register_bundle() 空迭代器
#[tokio::test]
async fn test_register_bundle_empty() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let defs: Vec<ComponentDefinition> = vec![];
    let bindings: Vec<TraitBinding> = vec![];
    let result = builder.register_bundle(defs, bindings);
    assert!(result.is_ok(), "register_bundle with empty iterators should succeed");
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext 元数据方法
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_context_id_not_empty() {
    let context = build_context();
    let id = context.id();
    assert!(!id.is_empty(), "id should not be empty");
}

#[tokio::test]
async fn test_context_id_contains_prefix() {
    let context = build_context();
    let id = context.id();
    assert!(id.contains("vernal-context"), "id should contain 'vernal-context' prefix, got: {}", id);
}

#[tokio::test]
async fn test_context_id_stable() {
    let context = build_context();
    let id1 = context.id();
    let id2 = context.id();
    assert_eq!(id1, id2, "id should be stable across calls");
}

#[tokio::test]
async fn test_context_set_id() {
    let context = build_context();
    let original_id = context.id();
    context.set_id("custom-id".to_string());
    let new_id = context.id();
    assert_eq!(new_id, "custom-id", "set_id should change the id");
    assert_ne!(new_id, original_id, "new id should differ from original");
}

#[tokio::test]
async fn test_context_set_id_empty() {
    let context = build_context();
    context.set_id(String::new());
    let id = context.id();
    assert!(id.is_empty(), "set_id with empty string should work");
}

#[tokio::test]
async fn test_context_application_name_default() {
    let context = build_context();
    let name = context.application_name();
    let _ = name.len();
}

#[tokio::test]
async fn test_context_application_name_stable() {
    let context = build_context();
    let name1 = context.application_name();
    let name2 = context.application_name();
    assert_eq!(name1, name2, "application_name should be stable");
}

#[tokio::test]
async fn test_context_set_application_name() {
    let context = build_context();
    context.set_application_name("my-app".to_string());
    let name = context.application_name();
    assert_eq!(name, "my-app", "set_application_name should change the name");
}

#[tokio::test]
async fn test_context_set_application_name_empty() {
    let context = build_context();
    context.set_application_name(String::new());
    let name = context.application_name();
    assert!(name.is_empty(), "set_application_name with empty string should work");
}

#[tokio::test]
async fn test_context_display_name_default() {
    let context = build_context();
    let name = context.display_name();
    assert!(!name.is_empty(), "display_name should not be empty by default");
}

#[tokio::test]
async fn test_context_display_name_stable() {
    let context = build_context();
    let name1 = context.display_name();
    let name2 = context.display_name();
    assert_eq!(name1, name2, "display_name should be stable");
}

#[tokio::test]
async fn test_context_set_display_name() {
    let context = build_context();
    context.set_display_name("My Application".to_string());
    let name = context.display_name();
    assert_eq!(name, "My Application", "set_display_name should change the display name");
}

#[tokio::test]
async fn test_context_parent_default() {
    let context = build_context();
    let parent = context.parent();
    assert!(parent.is_none(), "parent should be None by default");
}

#[tokio::test]
async fn test_context_set_parent() {
    let context = build_context();
    let parent = build_context();
    context.set_parent(Some(parent.clone()));
    let retrieved = context.parent();
    assert!(retrieved.is_some(), "parent should be set after set_parent");
}

#[tokio::test]
async fn test_context_set_parent_none() {
    let context = build_context();
    let parent = build_context();
    context.set_parent(Some(parent.clone()));
    assert!(context.parent().is_some());
    context.set_parent(None);
    assert!(context.parent().is_none(), "parent should be None after set_parent(None)");
}

#[tokio::test]
async fn test_context_startup_date() {
    let context = build_context();
    let date = context.startup_date();
    assert!(date > 0, "startup_date should be greater than 0, got: {}", date);
}

#[tokio::test]
async fn test_context_startup_date_stable() {
    let context = build_context();
    let date1 = context.startup_date();
    let date2 = context.startup_date();
    assert_eq!(date1, date2, "startup_date should be stable");
}

#[tokio::test]
async fn test_context_is_active_after_build() {
    let context = build_context();
    let active = context.is_active().await;
    let _ = active;
}

#[tokio::test]
async fn test_context_is_active_stable() {
    let context = build_context();
    let active1 = context.is_active().await;
    let active2 = context.is_active().await;
    assert_eq!(active1, active2, "is_active should be stable");
}

#[tokio::test]
async fn test_context_is_closed_after_build() {
    let context = build_context();
    let closed = context.is_closed().await;
    assert!(!closed, "context should not be closed after build");
}

#[tokio::test]
async fn test_context_is_closed_stable() {
    let context = build_context();
    let closed1 = context.is_closed().await;
    let closed2 = context.is_closed().await;
    assert_eq!(closed1, closed2, "is_closed should be stable");
}

#[tokio::test]
async fn test_context_register_shutdown_hook() {
    let context = build_context();
    context.register_shutdown_hook();
    context.register_shutdown_hook();
}

// ════════════════════════════════════════════════════════════════════
// 测试: StartupReport 方法
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_project_status() {
    let context = build_context();
    let report = context.startup_report().await;
    let status = report.project_status();
    assert!(!status.is_empty(), "project_status should not be empty");
}

#[tokio::test]
async fn test_project_status_stable() {
    let context = build_context();
    let report1 = context.startup_report().await;
    let report2 = context.startup_report().await;
    assert_eq!(report1.project_status(), report2.project_status(), "project_status should be stable");
}

#[tokio::test]
async fn test_framework_version() {
    let context = build_context();
    let report = context.startup_report().await;
    let version = report.framework_version();
    assert!(!version.is_empty(), "framework_version should not be empty");
}

#[tokio::test]
async fn test_minimum_rust_version() {
    let context = build_context();
    let report = context.startup_report().await;
    let version = report.minimum_rust_version();
    assert!(!version.is_empty(), "minimum_rust_version should not be empty");
}

#[tokio::test]
async fn test_context_state() {
    let context = build_context();
    let report = context.startup_report().await;
    let state = report.context_state();
    assert!(!state.is_empty(), "context_state should not be empty");
}

#[tokio::test]
async fn test_report_environment() {
    let context = build_context();
    let report = context.startup_report().await;
    let _environment = report.environment();
}

#[tokio::test]
async fn test_report_registry() {
    let context = build_context();
    let report = context.startup_report().await;
    let _registry = report.registry();
}

#[tokio::test]
async fn test_report_enabled_features() {
    let context = build_context();
    let report = context.startup_report().await;
    let features = report.enabled_features();
    let _ = features.len();
}

#[tokio::test]
async fn test_report_adapters() {
    let context = build_context();
    let report = context.startup_report().await;
    let adapters = report.adapters();
    let _ = adapters.len();
}

#[tokio::test]
async fn test_report_external_dependencies() {
    let context = build_context();
    let report = context.startup_report().await;
    let deps = report.external_dependencies();
    let _ = deps.len();
}

#[tokio::test]
async fn test_report_condition_evaluations() {
    let context = build_context();
    let report = context.startup_report().await;
    let evaluations = report.condition_evaluations();
    let _ = evaluations.len();
}

#[tokio::test]
async fn test_report_warnings() {
    let context = build_context();
    let report = context.startup_report().await;
    let warnings = report.warnings();
    let _ = warnings.len();
}

#[tokio::test]
async fn test_report_unused_definitions() {
    let context = build_context();
    let report = context.startup_report().await;
    let definitions = report.unused_definitions();
    let _ = definitions.len();
}

#[tokio::test]
async fn test_report_observations() {
    let context = build_context();
    let report = context.startup_report().await;
    let observations = report.observations();
    let _ = observations.len();
}

// ════════════════════════════════════════════════════════════════════
// 测试: VernalApplicationBuilder 方法
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_register_all() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let defs = vec![ComponentDefinition::singleton::<SimpleComponent, _>(|_| SimpleComponent)];
    let result = builder.register_all(defs);
    assert!(result.is_ok(), "register_all should succeed");
}

#[tokio::test]
async fn test_register_all_empty() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let defs: Vec<ComponentDefinition> = vec![];
    let result = builder.register_all(defs);
    assert!(result.is_ok(), "register_all with empty iterator should succeed");
}

#[tokio::test]
async fn test_lifecycle_qualified() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let q = Qualifier::new("test-q").unwrap();
    builder.lifecycle_qualified::<TestLifecycle>(q);
}

#[tokio::test]
async fn test_application_runner_qualified() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let q = Qualifier::new("runner-q").unwrap();
    builder.application_runner_qualified::<TestRunner>(q);
}

#[tokio::test]
async fn test_scheduled_task_qualified() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let q = Qualifier::new("task-q").unwrap();
    builder.scheduled_task_qualified::<TestTask>(q);
}

#[tokio::test]
async fn test_event_listener_qualified() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let q = Qualifier::new("listener-q").unwrap();
    builder.event_listener_qualified::<TestEvent, TestEventListener>(q);
}

#[tokio::test]
async fn test_register_event_listener_component() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let result = builder.register_event_listener_component::<TestEvent, TestEventListener>();
    assert!(result.is_ok(), "register_event_listener_component should succeed");
}

#[tokio::test]
async fn test_register_application_runner() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let result = builder.register_application_runner::<TestRunner>();
    assert!(result.is_ok(), "register_application_runner should succeed");
}

#[tokio::test]
async fn test_register_scheduled_task() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let result = builder.register_scheduled_task::<TestTask>();
    assert!(result.is_ok(), "register_scheduled_task should succeed");
}
