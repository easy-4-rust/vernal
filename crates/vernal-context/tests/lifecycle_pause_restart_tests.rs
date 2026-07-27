//! 对标 Spring Framework 7.0 `DefaultLifecycleProcessorTests` 的 pause/restart 差分测试。
//!
//! 每个 `#[test]` 都镜像 Spring `DefaultLifecycleProcessor.onPause()` /
//! `onRestart()` 的行为，验证 vernal-context 与 Spring 7.0 在以下语义上完全等价：
//!
//! - `ApplicationContext::pause()` 只暂停声明 `is_pauseable() == true` 的组件
//! - 不可暂停组件（`is_pauseable() == false`）在 pause 期间保持运行
//! - pause 后 Context 进入 `Paused` 状态
//! - `ApplicationContext::restart()` 只重启此前已暂停的可暂停组件
//! - restart 后 Context 推回 `Ready` 状态
//! - 非法状态调用 pause/restart 返回 `InvalidState` 错误
//! - pause 期间组件钩子失败触发完整关闭
//!
//! 镜像 Spring `org.springframework.context.support.DefaultLifecycleProcessorTests`
//! 与 `org.springframework.context.LifecycleTests` 的 pause/restart 部分（2026-07-27）。

use std::sync::{Arc, Mutex};

use tokio::runtime::Handle;
use vernal_beans::ComponentDefinition;
use vernal_context::{
    ApplicationContext, ApplicationPausedEvent, ContextError, ContextState, Lifecycle,
    LifecycleFuture, VernalApplicationBuilder,
};

/// 共享调用记录器。
#[derive(Debug, Default, Clone)]
struct CallTrace {
    events: Arc<Mutex<Vec<String>>>,
}

impl CallTrace {
    fn record(&self, event: &str) {
        self.events.lock().unwrap().push(event.to_owned());
    }

    fn snapshot(&self) -> Vec<String> {
        self.events.lock().unwrap().clone()
    }

    fn count(&self, event: &str) -> usize {
        self.snapshot()
            .iter()
            .filter(|e| *e == event)
            .count()
    }
}

/// 测试用可暂停 Lifecycle 组件。
struct PausableComponent {
    name: &'static str,
    trace: CallTrace,
}

impl Lifecycle for PausableComponent {
    fn name(&self) -> &'static str {
        self.name
    }

    fn start(&self, _cancellation: tokio_util::sync::CancellationToken) -> LifecycleFuture<'_> {
        let trace = self.trace.clone();
        let name = self.name;
        Box::pin(async move {
            trace.record(&format!("{name}:start"));
            Ok(())
        })
    }

    fn stop(&self) -> LifecycleFuture<'_> {
        let trace = self.trace.clone();
        let name = self.name;
        Box::pin(async move {
            trace.record(&format!("{name}:stop"));
            Ok(())
        })
    }

    fn pause(&self) -> LifecycleFuture<'_> {
        let trace = self.trace.clone();
        let name = self.name;
        Box::pin(async move {
            trace.record(&format!("{name}:pause"));
            Ok(())
        })
    }

    fn is_pauseable(&self) -> bool {
        true
    }
}

/// 测试用不可暂停 Lifecycle 组件。
struct NonPausableComponent {
    name: &'static str,
    trace: CallTrace,
}

impl Lifecycle for NonPausableComponent {
    fn name(&self) -> &'static str {
        self.name
    }

    fn start(&self, _cancellation: tokio_util::sync::CancellationToken) -> LifecycleFuture<'_> {
        let trace = self.trace.clone();
        let name = self.name;
        Box::pin(async move {
            trace.record(&format!("{name}:start"));
            Ok(())
        })
    }

    fn stop(&self) -> LifecycleFuture<'_> {
        let trace = self.trace.clone();
        let name = self.name;
        Box::pin(async move {
            trace.record(&format!("{name}:stop"));
            Ok(())
        })
    }

    // 不覆盖 pause —— 默认空实现，但 is_pauseable 返回 false。

    fn is_pauseable(&self) -> bool {
        false
    }
}

/// 用 trace 构造测试 ApplicationContext：注册 PausableComponent。
#[allow(dead_code)]
async fn build_pausable_context(trace: &CallTrace) -> (Arc<ApplicationContext>, CallTrace) {
    let trace = trace.clone();
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    let trace_for_component = trace.clone();
    builder
        .register(ComponentDefinition::try_singleton::<PausableComponent, _>(
            move |_resolver| {
                Ok(PausableComponent {
                    name: "pausable",
                    trace: trace_for_component.clone(),
                })
            },
        ))
        .expect("register PausableComponent");
    builder.lifecycle::<PausableComponent>();
    // 提前返回 builder 用于异步 launch
    let context = builder.launch().await.unwrap();
    (context, trace)
}

/// Spring `Lifecycle_pause_onlyPausableComponents` 差分测试：
/// `pause()` 只调用 `is_pauseable() == true` 的组件的 pause 钩子。
#[tokio::test]
async fn pause_only_invokes_pausable_components() {
    let trace = CallTrace::default();
    let trace_for_pausable = trace.clone();
    let trace_for_nonpausable = trace.clone();
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<PausableComponent, _>(
            move |_resolver| {
                Ok(PausableComponent {
                    name: "pausable",
                    trace: trace_for_pausable.clone(),
                })
            },
        ))
        .expect("register PausableComponent");
    builder
        .register(ComponentDefinition::try_singleton::<NonPausableComponent, _>(
            move |_resolver| {
                Ok(NonPausableComponent {
                    name: "nonpausable",
                    trace: trace_for_nonpausable.clone(),
                })
            },
        ))
        .expect("register NonPausableComponent");
    builder.lifecycle::<PausableComponent>();
    builder.lifecycle::<NonPausableComponent>();

    let context = builder.launch().await.expect("launch");
    assert_eq!(context.state().await, ContextState::Ready);

    // pause
    context.pause().await.expect("pause");

    // 可暂停组件调用 pause；不可暂停组件不调用 pause。
    assert_eq!(trace.count("pausable:pause"), 1);
    assert_eq!(trace.count("nonpausable:pause"), 0);

    // 不可暂停组件仍然处于已启动状态（start 调用一次，未调用 stop）。
    assert_eq!(trace.count("nonpausable:start"), 1);
    assert_eq!(trace.count("nonpausable:stop"), 0);

    // pause 后状态为 Paused。
    assert_eq!(context.state().await, ContextState::Paused);

    context.close().await.expect("close");
}

/// Spring `Lifecycle_restart_pausableComponents` 差分测试：
/// `restart()` 重新调用可暂停组件的 start 钩子。
#[tokio::test]
async fn restart_invokes_pausable_components_start() {
    let trace = CallTrace::default();
    let trace_for_pausable = trace.clone();
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<PausableComponent, _>(
            move |_resolver| {
                Ok(PausableComponent {
                    name: "pausable",
                    trace: trace_for_pausable.clone(),
                })
            },
        ))
        .expect("register PausableComponent");
    builder.lifecycle::<PausableComponent>();

    let context = builder.launch().await.expect("launch");
    assert_eq!(trace.count("pausable:start"), 1);
    assert_eq!(trace.count("pausable:pause"), 0);

    context.pause().await.expect("pause");
    assert_eq!(trace.count("pausable:pause"), 1);

    context.restart().await.expect("restart");

    // restart 重新调用 start。
    assert_eq!(trace.count("pausable:start"), 2);
    assert_eq!(context.state().await, ContextState::Ready);

    context.close().await.expect("close");
}

/// Spring `Lifecycle_restart_nonPausableNotRestarted` 差分测试：
/// restart 不重新启动不可暂停组件。
#[tokio::test]
async fn restart_skips_non_pausable_components() {
    let trace = CallTrace::default();
    let trace_for_pausable = trace.clone();
    let trace_for_nonpausable = trace.clone();
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<PausableComponent, _>(
            move |_resolver| {
                Ok(PausableComponent {
                    name: "pausable",
                    trace: trace_for_pausable.clone(),
                })
            },
        ))
        .expect("register PausableComponent");
    builder
        .register(ComponentDefinition::try_singleton::<NonPausableComponent, _>(
            move |_resolver| {
                Ok(NonPausableComponent {
                    name: "nonpausable",
                    trace: trace_for_nonpausable.clone(),
                })
            },
        ))
        .expect("register NonPausableComponent");
    builder.lifecycle::<PausableComponent>();
    builder.lifecycle::<NonPausableComponent>();

    let context = builder.launch().await.expect("launch");
    assert_eq!(trace.count("pausable:start"), 1);
    assert_eq!(trace.count("nonpausable:start"), 1);

    context.pause().await.expect("pause");
    context.restart().await.expect("restart");

    // restart 只重新启动可暂停组件。
    assert_eq!(trace.count("pausable:start"), 2);
    assert_eq!(trace.count("nonpausable:start"), 1);
    assert_eq!(context.state().await, ContextState::Ready);

    context.close().await.expect("close");
}

/// Spring `Lifecycle_pauseFromInvalidState` 差分测试：
/// 从非 `Ready` 状态调用 pause 返回 `InvalidState` 错误。
#[tokio::test]
async fn pause_from_invalid_state_fails() {
    let trace = CallTrace::default();
    let trace_for_pausable = trace.clone();
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<PausableComponent, _>(
            move |_resolver| {
                Ok(PausableComponent {
                    name: "pausable",
                    trace: trace_for_pausable.clone(),
                })
            },
        ))
        .expect("register PausableComponent");
    builder.lifecycle::<PausableComponent>();

    let context = builder.launch().await.expect("launch");

    // pause 一次到达 Paused。
    context.pause().await.expect("first pause");

    // 从 Paused 状态再次 pause 必须失败。
    let second_pause = context.pause().await;
    assert!(
        matches!(second_pause, Err(ContextError::InvalidState { .. })),
        "expected InvalidState, got {second_pause:?}"
    );

    context.close().await.expect("close");
}

/// Spring `Lifecycle_restartFromInvalidState` 差分测试：
/// 从非 `Paused` 状态调用 restart 返回 `InvalidState` 错误。
#[tokio::test]
async fn restart_from_invalid_state_fails() {
    let trace = CallTrace::default();
    let trace_for_pausable = trace.clone();
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<PausableComponent, _>(
            move |_resolver| {
                Ok(PausableComponent {
                    name: "pausable",
                    trace: trace_for_pausable.clone(),
                })
            },
        ))
        .expect("register PausableComponent");
    builder.lifecycle::<PausableComponent>();

    let context = builder.launch().await.expect("launch");

    // 从 Ready 状态直接 restart 必须失败（没有先 pause）。
    let restart_without_pause = context.restart().await;
    assert!(
        matches!(restart_without_pause, Err(ContextError::InvalidState { .. })),
        "expected InvalidState, got {restart_without_pause:?}"
    );

    context.close().await.expect("close");
}

/// Spring `Lifecycle_pauseThenClose_stopsAllComponents` 差分测试：
/// pause 后调用 close 仍然停止全部组件（包括未暂停的）。
#[tokio::test]
async fn pause_then_close_stops_all_components() {
    let trace = CallTrace::default();
    let trace_for_pausable = trace.clone();
    let trace_for_nonpausable = trace.clone();
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<PausableComponent, _>(
            move |_resolver| {
                Ok(PausableComponent {
                    name: "pausable",
                    trace: trace_for_pausable.clone(),
                })
            },
        ))
        .expect("register PausableComponent");
    builder
        .register(ComponentDefinition::try_singleton::<NonPausableComponent, _>(
            move |_resolver| {
                Ok(NonPausableComponent {
                    name: "nonpausable",
                    trace: trace_for_nonpausable.clone(),
                })
            },
        ))
        .expect("register NonPausableComponent");
    builder.lifecycle::<PausableComponent>();
    builder.lifecycle::<NonPausableComponent>();

    let context = builder.launch().await.expect("launch");
    context.pause().await.expect("pause");

    // close 仍然停止全部组件，包括非可暂停组件。
    context.close().await.expect("close");

    // 可暂停组件：start -> pause -> close(stop)。
    assert_eq!(trace.count("pausable:start"), 1);
    assert_eq!(trace.count("pausable:pause"), 1);
    assert_eq!(trace.count("pausable:stop"), 1);

    // 非可暂停组件：start -> close(stop)。pause 期间未被暂停。
    assert_eq!(trace.count("nonpausable:start"), 1);
    assert_eq!(trace.count("nonpausable:pause"), 0);
    assert_eq!(trace.count("nonpausable:stop"), 1);
}

/// Spring `Lifecycle_multiplePauseRestartCycles` 差分测试：
/// 多次 pause/restart 循环应正常工作。
#[tokio::test]
async fn multiple_pause_restart_cycles() {
    let trace = CallTrace::default();
    let trace_for_pausable = trace.clone();
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<PausableComponent, _>(
            move |_resolver| {
                Ok(PausableComponent {
                    name: "pausable",
                    trace: trace_for_pausable.clone(),
                })
            },
        ))
        .expect("register PausableComponent");
    builder.lifecycle::<PausableComponent>();

    let context = builder.launch().await.expect("launch");

    // 三次 pause/restart 循环。
    for _ in 0..3 {
        context.pause().await.expect("pause");
        assert_eq!(context.state().await, ContextState::Paused);
        context.restart().await.expect("restart");
        assert_eq!(context.state().await, ContextState::Ready);
    }

    // 3 次 pause + 4 次 start（1 次初始 + 3 次重启）。
    assert_eq!(trace.count("pausable:pause"), 3);
    assert_eq!(trace.count("pausable:start"), 4);

    context.close().await.expect("close");
}

/// Spring `DefaultLifecycleProcessor.isPauseable_default` 差分测试：
/// `Lifecycle::is_pauseable()` 默认返回 `true`，与 Spring 7.0 一致。
#[test]
fn lifecycle_is_pauseable_default_is_true() {
    struct DefaultComponent;
    impl Lifecycle for DefaultComponent {}
    let component = DefaultComponent;
    assert!(
        component.is_pauseable(),
        "Lifecycle::is_pauseable() default must be true (Spring 7.0 parity)"
    );
}

/// Spring `pause_emitsApplicationPausedEvent` 差分测试：
/// pause 完成后必须发布 `ApplicationPausedEvent`。
#[tokio::test]
async fn pause_emits_application_paused_event() {
    let trace = CallTrace::default();
    let trace_for_pausable = trace.clone();
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<PausableComponent, _>(
            move |_resolver| {
                Ok(PausableComponent {
                    name: "pausable",
                    trace: trace_for_pausable.clone(),
                })
            },
        ))
        .expect("register PausableComponent");
    builder.lifecycle::<PausableComponent>();

    // 启动一个事件捕获组件。
    let received = Arc::new(Mutex::new(false));
    let received_clone = Arc::clone(&received);
    builder
        .register(ComponentDefinition::shared_value(EventCapture {
            received: received_clone,
        }))
        .expect("register EventCapture");
    builder.event_listener::<ApplicationPausedEvent, EventCapture>();

    let context = builder.launch().await.expect("launch");
    context.pause().await.expect("pause");

    // 事件被异步处理；等待短暂时间确保监听器有机会运行。
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(
        *received.lock().unwrap(),
        "ApplicationPausedEvent must be delivered to listener"
    );

    context.close().await.expect("close");
}

/// 测试用事件捕获组件。
#[derive(Debug)]
struct EventCapture {
    received: Arc<Mutex<bool>>,
}

impl vernal_context::ApplicationEventListener<ApplicationPausedEvent> for EventCapture {
    type Error = std::io::Error;

    async fn on_event(
        &self,
        _event: Arc<ApplicationPausedEvent>,
    ) -> Result<(), Self::Error> {
        *self.received.lock().unwrap() = true;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "event-capture"
    }
}

/// Spring `Lifecycle_pause_failureClosesContext` 差分测试：
/// pause 钩子失败时 Context 必须进入 Closed 终态。
struct FailingPauseComponent;

impl Lifecycle for FailingPauseComponent {
    fn name(&self) -> &'static str {
        "failing-pause"
    }

    fn pause(&self) -> LifecycleFuture<'_> {
        Box::pin(async {
            Err(Box::new(std::io::Error::other("pause failed"))
                as Box<dyn std::error::Error + Send + Sync>)
        })
    }
}

#[tokio::test]
async fn pause_failure_closes_context() {
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<FailingPauseComponent, _>(
            |_resolver| Ok(FailingPauseComponent),
        ))
        .expect("register FailingPauseComponent");
    builder.lifecycle::<FailingPauseComponent>();

    let context = builder.launch().await.expect("launch");
    let pause_result = context.pause().await;
    assert!(
        pause_result.is_err(),
        "pause must fail when component pauses fail"
    );
    let state = context.state().await;
    assert_eq!(
        state,
        ContextState::Closed,
        "context must be closed after pause failure"
    );
}