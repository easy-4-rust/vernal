//! 集成测试 - 覆盖 event_bus.rs 和 application_module_registrar.rs 的未覆盖代码

use std::sync::Arc;
use std::time::Duration;

use vernal_context::{
    ApplicationRunner, Lifecycle, ScheduledTask, TaskSchedule,
    VernalApplicationBuilder, ApplicationEventListener,
    ApplicationModuleRegistrar, ApplicationEnvironment,
    ConfigurationPhase, ConditionalComponentModule, ComponentCondition,
};
use vernal_beans::{Component, ComponentDefinition, Qualifier};

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

struct AlwaysTrueCondition;
impl ComponentCondition for AlwaysTrueCondition {
    fn name(&self) -> &'static str { "always_true" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(true)
    }
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationModuleRegistrar 方法
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_registrar_register() {
    let mut registrar = ApplicationModuleRegistrar::new();
    registrar.register(SimpleComponent::definition());
}

#[test]
fn test_registrar_register_all() {
    let mut registrar = ApplicationModuleRegistrar::new();
    let defs = vec![ComponentDefinition::singleton::<SimpleComponent, _>(|_| SimpleComponent)];
    registrar.register_all(defs);
}

#[test]
fn test_registrar_component() {
    let mut registrar = ApplicationModuleRegistrar::new();
    registrar.component::<SimpleComponent>();
}

#[test]
fn test_registrar_lifecycle() {
    let mut registrar = ApplicationModuleRegistrar::new();
    registrar.lifecycle::<TestLifecycle>();
}

#[test]
fn test_registrar_lifecycle_qualified() {
    let mut registrar = ApplicationModuleRegistrar::new();
    let q = Qualifier::new("test-q").unwrap();
    registrar.lifecycle_qualified::<TestLifecycle>(q);
}

#[test]
fn test_registrar_event_listener() {
    let mut registrar = ApplicationModuleRegistrar::new();
    registrar.event_listener::<TestEvent, TestEventListener>();
}

#[test]
fn test_registrar_event_listener_qualified() {
    let mut registrar = ApplicationModuleRegistrar::new();
    let q = Qualifier::new("test-q").unwrap();
    registrar.event_listener_qualified::<TestEvent, TestEventListener>(q);
}

#[test]
fn test_registrar_application_runner() {
    let mut registrar = ApplicationModuleRegistrar::new();
    registrar.application_runner::<TestRunner>();
}

#[test]
fn test_registrar_application_runner_qualified() {
    let mut registrar = ApplicationModuleRegistrar::new();
    let q = Qualifier::new("test-q").unwrap();
    registrar.application_runner_qualified::<TestRunner>(q);
}

#[test]
fn test_registrar_scheduled_task() {
    let mut registrar = ApplicationModuleRegistrar::new();
    registrar.scheduled_task::<TestTask>();
}

#[test]
fn test_registrar_scheduled_task_qualified() {
    let mut registrar = ApplicationModuleRegistrar::new();
    let q = Qualifier::new("test-q").unwrap();
    registrar.scheduled_task_qualified::<TestTask>(q);
}

#[test]
fn test_registrar_register_event_listener_component() {
    let mut registrar = ApplicationModuleRegistrar::new();
    registrar.register_event_listener_component::<TestEvent, TestEventListener>();
}

#[test]
fn test_registrar_register_application_runner() {
    let mut registrar = ApplicationModuleRegistrar::new();
    registrar.register_application_runner::<TestRunner>();
}

#[test]
fn test_registrar_register_scheduled_task() {
    let mut registrar = ApplicationModuleRegistrar::new();
    registrar.register_scheduled_task::<TestTask>();
}

#[test]
fn test_registrar_conditional() {
    let mut registrar = ApplicationModuleRegistrar::new();
    let module = ConditionalComponentModule::new("test", AlwaysTrueCondition);
    registrar.conditional(module);
}

#[test]
fn test_registrar_conditionals() {
    let mut registrar = ApplicationModuleRegistrar::new();
    let modules = vec![
        ConditionalComponentModule::new("test1", AlwaysTrueCondition),
        ConditionalComponentModule::new("test2", AlwaysTrueCondition),
    ];
    registrar.conditionals(modules);
}

#[test]
fn test_registrar_into_parts() {
    let mut registrar = ApplicationModuleRegistrar::new();
    registrar.register(SimpleComponent::definition());
    let parts = registrar.into_parts();
    assert!(!parts.is_empty());
}

#[test]
fn test_registrar_bind() {
    let mut registrar = ApplicationModuleRegistrar::new();
    let binding = vernal_beans::TraitBinding::new::<dyn std::any::Any + Send + Sync, SimpleComponent, _>(|c| c);
    registrar.bind(binding);
}

#[test]
fn test_registrar_bind_all() {
    let mut registrar = ApplicationModuleRegistrar::new();
    let bindings = vec![
        vernal_beans::TraitBinding::new::<dyn std::any::Any + Send + Sync, SimpleComponent, _>(|c| c),
    ];
    registrar.bind_all(bindings);
}

// ════════════════════════════════════════════════════════════════════
// 测试: 通过 build() 路径执行所有闭包
// ════════════════════════════════════════════════════════════════════

/// 验证 lifecycle 闭包通过 build() 执行
#[tokio::test]
async fn test_lifecycle_via_build() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder.register(SimpleComponent::definition());
    builder.register(TestLifecycle::definition());
    builder.lifecycle::<TestLifecycle>();
    let result = builder.build();
    assert!(result.is_ok(), "build with lifecycle should succeed: {:?}", result.err());
}

/// 验证 event_listener 闭包通过 build() 执行
#[tokio::test]
async fn test_event_listener_via_build() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder.register(SimpleComponent::definition());
    builder.register(TestEventListener::definition());
    builder.event_listener::<TestEvent, TestEventListener>();
    let result = builder.build();
    assert!(result.is_ok(), "build with event_listener should succeed: {:?}", result.err());
}

/// 验证 application_runner 闭包通过 build() 执行
#[tokio::test]
async fn test_application_runner_via_build() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder.register(SimpleComponent::definition());
    builder.register(TestRunner::definition());
    builder.application_runner::<TestRunner>();
    let result = builder.build();
    assert!(result.is_ok(), "build with application_runner should succeed: {:?}", result.err());
}

/// 验证 scheduled_task 闭包通过 build() 执行
#[tokio::test]
async fn test_scheduled_task_via_build() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder.register(SimpleComponent::definition());
    builder.register(TestTask::definition());
    builder.scheduled_task::<TestTask>();
    let result = builder.build();
    assert!(result.is_ok(), "build with scheduled_task should succeed: {:?}", result.err());
}

/// 验证 conditional module 通过 build() 执行
#[tokio::test]
async fn test_conditional_module_via_build() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let mut module = ConditionalComponentModule::new("test-cond", AlwaysTrueCondition)
        .with_phase(ConfigurationPhase::ParseConfiguration);
    module.register(SimpleComponent::definition());
    builder.register_conditional(module).unwrap();
    let result = builder.build();
    assert!(result.is_ok(), "conditional module build should succeed: {:?}", result.err());
}
