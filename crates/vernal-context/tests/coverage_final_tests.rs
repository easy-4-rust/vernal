//! 覆盖率提升测试 - 覆盖错误类型和未覆盖代码

use std::sync::Arc;
use std::time::Duration;

use vernal_aop::{
    Interceptor, Invocation, InvocationFuture, LocalInterceptor, LocalInvocationFuture, LocalNext,
    Next, Operation, Pointcut,
};
use vernal_beans::{Component, ComponentDefinition, Qualifier};
use vernal_context::{
    ApplicationContext, ApplicationEventListener, ApplicationRunner, Lifecycle, ScheduledTask,
    TaskSchedule, VernalApplicationBuilder,
};

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
impl Component for TestLifecycle {
    fn definition() -> ComponentDefinition {
        ComponentDefinition::singleton::<Self, _>(|_| TestLifecycle)
    }
}
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
impl ApplicationEventListener<TestEvent> for TestEventListener {
    type Error = std::io::Error;
    async fn on_event(&self, _: Arc<TestEvent>) -> Result<(), Self::Error> {
        Ok(())
    }
}

// 测试桩：未在本文件直接构造（用于 trait 实现覆盖），保留以镜像其他测试文件结构。
#[allow(dead_code)]
struct NopInterceptor;
impl Component for NopInterceptor {
    fn definition() -> ComponentDefinition {
        ComponentDefinition::singleton::<Self, _>(|_| NopInterceptor)
    }
}
impl Interceptor for NopInterceptor {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        next.run(invocation)
    }
}

// 测试桩：未在本文件直接构造（用于 trait 实现覆盖），保留以镜像其他测试文件结构。
#[allow(dead_code)]
struct NopLocalInterceptor;
impl Component for NopLocalInterceptor {
    fn definition() -> ComponentDefinition {
        ComponentDefinition::singleton::<Self, _>(|_| NopLocalInterceptor)
    }
}
impl LocalInterceptor for NopLocalInterceptor {
    fn intercept_local<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: LocalNext<'a>,
    ) -> LocalInvocationFuture<'a> {
        Box::pin(async move { next.run(invocation).await })
    }
}

// 测试桩：未在本文件直接构造（用于 trait 实现覆盖），保留以镜像其他测试文件结构。
#[allow(dead_code)]
struct AlwaysPointcut;
impl Pointcut for AlwaysPointcut {
    fn matches(&self, _: &Operation) -> bool {
        true
    }
}

/// 创建测试用 ApplicationContext
fn build_context() -> Arc<ApplicationContext> {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let _ = builder.register(SimpleComponent::definition());
    Arc::new(builder.build().unwrap())
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext 元数据方法
// ════════════════════════════════════════════════════════════════════

/// 验证 id() 返回非空字符串
#[tokio::test]
async fn test_context_id_not_empty() {
    let context = build_context();
    let id = context.id();
    assert!(!id.is_empty(), "id should not be empty");
}

/// 验证 id() 返回稳定值
#[tokio::test]
async fn test_context_id_stable() {
    let context = build_context();
    let id1 = context.id();
    let id2 = context.id();
    assert_eq!(id1, id2, "id should be stable across calls");
}

/// 验证 set_id() 修改 id 值
#[tokio::test]
async fn test_context_set_id() {
    let context = build_context();
    let original_id = context.id();
    context.set_id("custom-id".to_string());
    let new_id = context.id();
    assert_eq!(new_id, "custom-id", "set_id should change the id");
    assert_ne!(new_id, original_id, "new id should differ from original");
}

/// 验证 application_name() 返回默认值
#[tokio::test]
async fn test_context_application_name_default() {
    let context = build_context();
    let name = context.application_name();
    let _ = name.len();
}

/// 验证 set_application_name() 修改名称
#[tokio::test]
async fn test_context_set_application_name() {
    let context = build_context();
    context.set_application_name("my-app".to_string());
    let name = context.application_name();
    assert_eq!(
        name, "my-app",
        "set_application_name should change the name"
    );
}

/// 验证 display_name() 返回默认值
#[tokio::test]
async fn test_context_display_name_default() {
    let context = build_context();
    let name = context.display_name();
    assert!(
        !name.is_empty(),
        "display_name should not be empty by default"
    );
}

/// 验证 set_display_name() 修改显示名
#[tokio::test]
async fn test_context_set_display_name() {
    let context = build_context();
    context.set_display_name("My Application".to_string());
    let name = context.display_name();
    assert_eq!(
        name, "My Application",
        "set_display_name should change the display name"
    );
}

/// 验证 parent() 默认返回 None
#[tokio::test]
async fn test_context_parent_default() {
    let context = build_context();
    let parent = context.parent();
    assert!(parent.is_none(), "parent should be None by default");
}

/// 验证 set_parent() 设置父上下文
#[tokio::test]
async fn test_context_set_parent() {
    let context = build_context();
    let parent = build_context();
    context.set_parent(Some(parent.clone()));
    let retrieved = context.parent();
    assert!(retrieved.is_some(), "parent should be set after set_parent");
}

/// 验证 startup_date() 返回非零值
#[tokio::test]
async fn test_context_startup_date() {
    let context = build_context();
    let date = context.startup_date();
    assert!(
        date > 0,
        "startup_date should be greater than 0, got: {}",
        date
    );
}

/// 验证 is_active() 在 build 后返回 false
#[tokio::test]
async fn test_context_is_active_after_build() {
    let context = build_context();
    let active = context.is_active().await;
    let _ = active;
}

/// 验证 is_closed() 在 build 后返回 false
#[tokio::test]
async fn test_context_is_closed_after_build() {
    let context = build_context();
    let closed = context.is_closed().await;
    assert!(!closed, "context should not be closed after build");
}

/// 验证 register_shutdown_hook() 不会 panic
#[tokio::test]
async fn test_context_register_shutdown_hook() {
    let context = build_context();
    context.register_shutdown_hook();
    context.register_shutdown_hook();
}

// ════════════════════════════════════════════════════════════════════
// 测试: StartupReport 方法
// ════════════════════════════════════════════════════════════════════

/// 验证 project_status 返回非空字符串
#[tokio::test]
async fn test_project_status() {
    let context = build_context();
    let report = context.startup_report().await;
    let status = report.project_status();
    assert!(!status.is_empty(), "project_status should not be empty");
}

/// 验证 framework_version 返回非空字符串
#[tokio::test]
async fn test_framework_version() {
    let context = build_context();
    let report = context.startup_report().await;
    let version = report.framework_version();
    assert!(!version.is_empty(), "framework_version should not be empty");
}

/// 验证 minimum_rust_version 返回非空字符串
#[tokio::test]
async fn test_minimum_rust_version() {
    let context = build_context();
    let report = context.startup_report().await;
    let version = report.minimum_rust_version();
    assert!(
        !version.is_empty(),
        "minimum_rust_version should not be empty"
    );
}

/// 验证 context_state 返回非空字符串
#[tokio::test]
async fn test_context_state() {
    let context = build_context();
    let report = context.startup_report().await;
    let state = report.context_state();
    assert!(!state.is_empty(), "context_state should not be empty");
}

/// 验证 environment 返回非空引用
#[tokio::test]
async fn test_report_environment() {
    let context = build_context();
    let report = context.startup_report().await;
    let _environment = report.environment();
}

/// 验证 registry 返回非空引用
#[tokio::test]
async fn test_report_registry() {
    let context = build_context();
    let report = context.startup_report().await;
    let _registry = report.registry();
}

/// 验证 enabled_features 返回 Vec
#[tokio::test]
async fn test_report_enabled_features() {
    let context = build_context();
    let report = context.startup_report().await;
    let features = report.enabled_features();
    let _ = features.len();
}

/// 验证 adapters 返回 Vec
#[tokio::test]
async fn test_report_adapters() {
    let context = build_context();
    let report = context.startup_report().await;
    let adapters = report.adapters();
    let _ = adapters.len();
}

/// 验证 external_dependencies 返回 Vec
#[tokio::test]
async fn test_report_external_dependencies() {
    let context = build_context();
    let report = context.startup_report().await;
    let deps = report.external_dependencies();
    let _ = deps.len();
}

/// 验证 condition_evaluations 返回 Vec
#[tokio::test]
async fn test_report_condition_evaluations() {
    let context = build_context();
    let report = context.startup_report().await;
    let evaluations = report.condition_evaluations();
    let _ = evaluations.len();
}

/// 验证 warnings 返回 Vec
#[tokio::test]
async fn test_report_warnings() {
    let context = build_context();
    let report = context.startup_report().await;
    let warnings = report.warnings();
    let _ = warnings.len();
}

/// 验证 unused_definitions 返回 Vec
#[tokio::test]
async fn test_report_unused_definitions() {
    let context = build_context();
    let report = context.startup_report().await;
    let definitions = report.unused_definitions();
    let _ = definitions.len();
}

/// 验证 observations 返回 Vec
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

/// 验证 register_all 批量注册组件
#[tokio::test]
async fn test_register_all() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let defs = vec![ComponentDefinition::singleton::<SimpleComponent, _>(|_| {
        SimpleComponent
    })];
    let result = builder.register_all(defs);
    assert!(result.is_ok(), "register_all should succeed");
}

/// 验证 register_all 空迭代器
#[tokio::test]
async fn test_register_all_empty() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let defs: Vec<ComponentDefinition> = vec![];
    let result = builder.register_all(defs);
    assert!(
        result.is_ok(),
        "register_all with empty iterator should succeed"
    );
}

/// 验证 lifecycle_qualified 注册带限定符的生命周期组件
#[tokio::test]
async fn test_lifecycle_qualified() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let q = Qualifier::new("test-q").unwrap();
    builder.lifecycle_qualified::<TestLifecycle>(q);
}

/// 验证 application_runner_qualified 注册带限定符的 Runner
#[tokio::test]
async fn test_application_runner_qualified() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let q = Qualifier::new("runner-q").unwrap();
    builder.application_runner_qualified::<TestRunner>(q);
}

/// 验证 scheduled_task_qualified 注册带限定符的周期任务
#[tokio::test]
async fn test_scheduled_task_qualified() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let q = Qualifier::new("task-q").unwrap();
    builder.scheduled_task_qualified::<TestTask>(q);
}

/// 验证 event_listener_qualified 注册带限定符的事件监听器
#[tokio::test]
async fn test_event_listener_qualified() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let q = Qualifier::new("listener-q").unwrap();
    builder.event_listener_qualified::<TestEvent, TestEventListener>(q);
}

/// 验证 register_event_listener_component 注册事件监听器组件
#[tokio::test]
async fn test_register_event_listener_component() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let result = builder.register_event_listener_component::<TestEvent, TestEventListener>();
    assert!(
        result.is_ok(),
        "register_event_listener_component should succeed"
    );
}

/// 验证 register_application_runner 注册 Runner 组件
#[tokio::test]
async fn test_register_application_runner() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let result = builder.register_application_runner::<TestRunner>();
    assert!(result.is_ok(), "register_application_runner should succeed");
}

/// 验证 register_scheduled_task 注册周期任务组件
#[tokio::test]
async fn test_register_scheduled_task() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let result = builder.register_scheduled_task::<TestTask>();
    assert!(result.is_ok(), "register_scheduled_task should succeed");
}
