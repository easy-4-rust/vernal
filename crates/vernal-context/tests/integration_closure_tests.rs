//! 集成测试 - 通过 ApplicationContext::build() 路径执行闭包体
//!
//! 这些测试通过完整的 build() 路径执行 Box<dyn FnOnce> 闭包体，
//! 确保 LLVM coverage 能够跟踪闭包执行。

use std::sync::Arc;
use std::time::Duration;

use vernal_aop::{
    Interceptor, Invocation, InvocationFuture, LocalInterceptor, LocalInvocationFuture, LocalNext,
    Next, Operation, Pointcut,
};
use vernal_beans::{Component, ComponentDefinition};
use vernal_context::{
    ApplicationRunner, Lifecycle, ScheduledTask, TaskSchedule, VernalApplicationBuilder,
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
impl vernal_context::ApplicationEventListener<TestEvent> for TestEventListener {
    type Error = std::io::Error;
    async fn on_event(&self, _: Arc<TestEvent>) -> Result<(), Self::Error> {
        Ok(())
    }
}

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

struct AlwaysPointcut;
impl Pointcut for AlwaysPointcut {
    fn matches(&self, _: &Operation) -> bool {
        true
    }
}

// ════════════════════════════════════════════════════════════════════
// 测试: 通过 build() 路径执行 lifecycle 闭包
// ════════════════════════════════════════════════════════════════════

/// 验证 lifecycle() 闭包通过 build() 执行
#[tokio::test]
async fn test_lifecycle_closure_via_build() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let _ = builder.register(SimpleComponent::definition());
    let _ = builder.register(TestLifecycle::definition());
    builder.lifecycle::<TestLifecycle>();
    let result = builder.build();
    assert!(
        result.is_ok(),
        "build with lifecycle should succeed: {:?}",
        result.err()
    );
}

// ════════════════════════════════════════════════════════════════════
// 测试: 通过 build() 路径执行 event_listener 闭包
// ════════════════════════════════════════════════════════════════════

/// 验证 event_listener() 闭包通过 build() 执行
#[tokio::test]
async fn test_event_listener_closure_via_build() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let _ = builder.register(SimpleComponent::definition());
    let _ = builder.register(TestEventListener::definition());
    builder.event_listener::<TestEvent, TestEventListener>();
    let result = builder.build();
    assert!(
        result.is_ok(),
        "build with event_listener should succeed: {:?}",
        result.err()
    );
}

// ════════════════════════════════════════════════════════════════════
// 测试: 通过 build() 路径执行 application_runner 闭包
// ════════════════════════════════════════════════════════════════════

/// 验证 application_runner() 闭包通过 build() 执行
#[tokio::test]
async fn test_application_runner_closure_via_build() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let _ = builder.register(SimpleComponent::definition());
    let _ = builder.register(TestRunner::definition());
    builder.application_runner::<TestRunner>();
    let result = builder.build();
    assert!(
        result.is_ok(),
        "build with application_runner should succeed: {:?}",
        result.err()
    );
}

// ════════════════════════════════════════════════════════════════════
// 测试: 通过 build() 路径执行 scheduled_task 闭包
// ════════════════════════════════════════════════════════════════════

/// 验证 scheduled_task() 闭包通过 build() 执行
#[tokio::test]
async fn test_scheduled_task_closure_via_build() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let _ = builder.register(SimpleComponent::definition());
    let _ = builder.register(TestTask::definition());
    builder.scheduled_task::<TestTask>();
    let result = builder.build();
    assert!(
        result.is_ok(),
        "build with scheduled_task should succeed: {:?}",
        result.err()
    );
}

// ════════════════════════════════════════════════════════════════════
// 测试: 通过 build() 路径执行 AOP 顾问
// ════════════════════════════════════════════════════════════════════

/// 验证 advisor_component() 通过 build() 执行
#[tokio::test]
async fn test_advisor_component_via_build() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let _ = builder.register(SimpleComponent::definition());
    let _ = builder.register(NopInterceptor::definition());
    builder.advisor_component::<NopInterceptor, AlwaysPointcut>(AlwaysPointcut, 0);
    let result = builder.build();
    assert!(
        result.is_ok(),
        "build with advisor_component should succeed: {:?}",
        result.err()
    );
}

/// 验证 local_advisor_component() 通过 build() 执行
#[tokio::test]
async fn test_local_advisor_component_via_build() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let _ = builder.register(SimpleComponent::definition());
    let _ = builder.register(NopLocalInterceptor::definition());
    builder.local_advisor_component::<NopLocalInterceptor, AlwaysPointcut>(AlwaysPointcut, 0);
    let result = builder.build();
    assert!(
        result.is_ok(),
        "build with local_advisor_component should succeed: {:?}",
        result.err()
    );
}

// ════════════════════════════════════════════════════════════════════
// 测试: 组合场景 - 多种组件类型同时注册
// ════════════════════════════════════════════════════════════════════

/// 验证多种组件类型同时注册并通过 build() 执行
#[tokio::test]
async fn test_combined_registration_via_build() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());

    // 注册组件
    let _ = builder.register(SimpleComponent::definition());
    let _ = builder.register(TestEventListener::definition());
    let _ = builder.register(TestRunner::definition());
    let _ = builder.register(TestTask::definition());
    let _ = builder.register(TestLifecycle::definition());
    let _ = builder.register(NopInterceptor::definition());

    // 注册生命周期
    builder.lifecycle::<TestLifecycle>();

    // 注册事件监听器
    builder.event_listener::<TestEvent, TestEventListener>();

    // 注册 Runner
    builder.application_runner::<TestRunner>();

    // 注册周期任务
    builder.scheduled_task::<TestTask>();

    // 注册 AOP 顾问
    builder.advisor_component::<NopInterceptor, AlwaysPointcut>(AlwaysPointcut, 0);

    // 构建上下文
    let result = builder.build();
    assert!(
        result.is_ok(),
        "combined registration should succeed: {:?}",
        result.err()
    );
}
