//! 契约验证测试 - 覆盖剩余未覆盖代码
//!
//! 这些测试验证：
//! - VernalApplicationBuilder 的未覆盖方法
//! - 条件装配的边界情况

use std::sync::Arc;
use std::time::Duration;

use vernal_beans::{Component, ComponentDefinition, Qualifier};
use vernal_context::{
    ApplicationEnvironment, ApplicationRunner, ConfigurationProperties,
    ConfigurationPropertiesError, Lifecycle, ScheduledTask, TaskSchedule, VernalApplicationBuilder,
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

struct TestConfig;
impl ConfigurationProperties for TestConfig {
    const PREFIX: &'static str = "test.config";
    fn bind_with_prefix(
        _env: &ApplicationEnvironment,
        _prefix: &str,
    ) -> Result<Self, ConfigurationPropertiesError> {
        Ok(TestConfig)
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

// ════════════════════════════════════════════════════════════════════
// 测试: 配置属性注册
// ════════════════════════════════════════════════════════════════════

/// 验证 configuration_properties 注册类型安全配置对象
#[tokio::test]
async fn test_configuration_properties_registration() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let result = builder.configuration_properties::<TestConfig>();
    assert!(result.is_ok(), "configuration_properties should succeed");
}

// ════════════════════════════════════════════════════════════════════
// 测试: 批量注册
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

// ════════════════════════════════════════════════════════════════════
// 测试: 限定符组件注册
// ════════════════════════════════════════════════════════════════════

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

// ════════════════════════════════════════════════════════════════════
// 测试: 组件注册
// ════════════════════════════════════════════════════════════════════

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
