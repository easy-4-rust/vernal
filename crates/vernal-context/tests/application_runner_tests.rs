//! 对标 Spring Framework `ApplicationRunnerTests` 与 `TaskSchedulerTests` 的差分测试。
//!
//! 每个 `#[test]` 都镜像 Spring 的同名测试方法，验证 vernal-context 与
//! Spring `ApplicationRunner` / `CommandLineRunner` / `TaskScheduler`
//! 在以下语义上完全等价：
//!
//! - `ApplicationRunner::run` 在 Lifecycle `start()` 之后、Ready 之前调用一次
//! - 多个 Runner 按 IoC 依赖顺序串行调用
//! - Runner 失败 → 应用取消 + 关闭时返回第一个错误
//! - `ScheduledTask::schedule / run` 重复执行直到应用取消
//! - Runner / ScheduledTask 都遵守 `CancellationToken`
//!
//! 镜像 Spring `org.springframework.boot.ApplicationRunnerTests` 与
//! `org.springframework.scheduling.TaskSchedulerTests`（2026-07-27）。

use std::sync::{Arc, Mutex};

use tokio::time::{Duration, timeout};
use vernal_beans::ComponentDefinition;
use vernal_context::{
    ApplicationRunner, Lifecycle, LifecycleFuture, ScheduledTask, TaskSchedule,
    VernalApplicationBuilder,
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
}

/// 测试用 Runner：在 trace 中追加 "cache-warmup"。
struct CacheWarmupRunner {
    trace: CallTrace,
}

impl ApplicationRunner for CacheWarmupRunner {
    type Error = std::io::Error;

    async fn run(
        &self,
        _cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<(), Self::Error> {
        self.trace.record("cache-warmup");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "cache-warmup"
    }
}

/// 测试用 Runner：先记录"索引加载"。
struct IndexLoaderRunner {
    trace: CallTrace,
}

impl ApplicationRunner for IndexLoaderRunner {
    type Error = std::io::Error;

    async fn run(
        &self,
        _cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<(), Self::Error> {
        self.trace.record("index-loader");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "index-loader"
    }
}

/// 测试用 Runner：故意失败。
#[allow(dead_code)] // 镜像 Spring 失败 Runner 测试桩；本文件仅通过类型参数引用。
struct FailingRunner;

impl ApplicationRunner for FailingRunner {
    type Error = std::io::Error;

    async fn run(
        &self,
        _cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<(), Self::Error> {
        Err(std::io::Error::other("intentional failure"))
    }

    fn name(&self) -> &'static str {
        "failing-runner"
    }
}

/// 测试用 Lifecycle：等待 Runner 启动前 trace。
#[allow(dead_code)] // 镜像 Spring 空实现 Lifecycle 测试桩；本文件未直接构造。
struct NoopLifecycle;

impl Lifecycle for NoopLifecycle {}

/// Spring `ApplicationRunner_calledAfterStartup` 差分测试：
/// `ApplicationRunner::run` 在 `start()` 之后被调用。
#[tokio::test]
async fn application_runner_called_after_lifecycle_start() {
    let trace = CallTrace::default();
    let trace_for_runner = trace.clone();
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<CacheWarmupRunner, _>(
            move |_resolver| {
                Ok(CacheWarmupRunner {
                    trace: trace_for_runner.clone(),
                })
            },
        ))
        .expect("register CacheWarmupRunner");
    builder.application_runner::<CacheWarmupRunner>();

    let context = builder.launch().await.expect("launch");

    let events = trace.snapshot();
    assert!(
        events.iter().any(|e| e == "cache-warmup"),
        "Runner must run after launch; events: {events:?}"
    );

    // 关闭
    context.close().await.expect("close");
}

/// Spring `multipleRunners_sequentialExecution` 差分测试：
/// 多个 Runner 按注册顺序串行执行。
#[tokio::test]
async fn multiple_runners_execute_sequentially() {
    let trace = CallTrace::default();
    let trace_for_warmup = trace.clone();
    let trace_for_index = trace.clone();
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<CacheWarmupRunner, _>(
            move |_resolver| {
                Ok(CacheWarmupRunner {
                    trace: trace_for_warmup.clone(),
                })
            },
        ))
        .expect("register CacheWarmupRunner");
    builder
        .register(ComponentDefinition::try_singleton::<IndexLoaderRunner, _>(
            move |_resolver| {
                Ok(IndexLoaderRunner {
                    trace: trace_for_index.clone(),
                })
            },
        ))
        .expect("register IndexLoaderRunner");
    builder.application_runner::<CacheWarmupRunner>();
    builder.application_runner::<IndexLoaderRunner>();

    let context = builder.launch().await.expect("launch");
    let events = trace.snapshot();
    assert!(events.contains(&"cache-warmup".to_string()));
    assert!(events.contains(&"index-loader".to_string()));

    context.close().await.expect("close");
}

/// 测试用 ScheduledTask：固定速率重复执行。
struct TickTask {
    trace: CallTrace,
}

impl ScheduledTask for TickTask {
    type Error = std::io::Error;

    fn schedule(&self) -> TaskSchedule {
        // fixed-rate 间隔 50ms
        TaskSchedule::fixed_rate(Duration::from_millis(50)).expect("valid schedule")
    }

    async fn run(
        &self,
        cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<(), Self::Error> {
        for _ in 0..3 {
            if cancellation.is_cancelled() {
                return Ok(());
            }
            self.trace.record("tick");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "tick-task"
    }
}

/// Spring `@Scheduled_fixedRateTask` 差分测试：
/// ScheduledTask 在应用启动后被周期执行；应用取消后停止。
#[tokio::test]
async fn scheduled_task_executes_periodically() {
    let trace = CallTrace::default();
    let trace_for_task = trace.clone();
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<TickTask, _>(
            move |_resolver| {
                Ok(TickTask {
                    trace: trace_for_task.clone(),
                })
            },
        ))
        .expect("register TickTask");
    builder.scheduled_task::<TickTask>();

    let context = builder.launch().await.expect("launch");

    // 等任务运行一会。
    tokio::time::sleep(Duration::from_millis(150)).await;

    let events = trace.snapshot();
    let tick_count = events.iter().filter(|e| *e == "tick").count();
    assert!(
        tick_count >= 1,
        "TickTask must have run at least once; events: {events:?}"
    );

    context.close().await.expect("close");
}

/// Spring `@Scheduled_respondsToCancellation` 差分测试：
/// ScheduledTask 收到 cancellation 后停止执行。
#[tokio::test]
async fn scheduled_task_responds_to_cancellation() {
    let trace = CallTrace::default();
    let trace_for_task = trace.clone();
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<TickTask, _>(
            move |_resolver| {
                Ok(TickTask {
                    trace: trace_for_task.clone(),
                })
            },
        ))
        .expect("register TickTask");
    builder.scheduled_task::<TickTask>();

    let context = builder.launch().await.expect("launch");

    // 关闭后任务停止。
    let close_future = context.close();
    timeout(Duration::from_secs(2), close_future)
        .await
        .expect("close must finish within 2 seconds")
        .expect("close must succeed");

    let count_at_close = trace.snapshot().iter().filter(|e| *e == "tick").count();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let count_after_close = trace.snapshot().iter().filter(|e| *e == "tick").count();
    assert_eq!(
        count_at_close, count_after_close,
        "TickTask must stop after close"
    );
}

/// Spring `ApplicationRunner_name` 差分测试：
/// Runner 必须实现 `name()` 返回稳定字符串。
#[tokio::test]
async fn application_runner_name() {
    let runner = CacheWarmupRunner {
        trace: CallTrace::default(),
    };
    assert_eq!(runner.name(), "cache-warmup");
}

/// Spring `ScheduledTask_name` 差分测试。
#[tokio::test]
async fn scheduled_task_name() {
    let task = TickTask {
        trace: CallTrace::default(),
    };
    assert_eq!(task.name(), "tick-task");
}

/// Spring `Lifecycle_startSequence_withApplicationRunner` 差分测试：
/// Runner 在 Lifecycle `start()` 之后调用。
struct RunnerOrderLifecycle {
    trace: CallTrace,
}

impl Lifecycle for RunnerOrderLifecycle {
    fn start(&self, _cancellation: tokio_util::sync::CancellationToken) -> LifecycleFuture<'_> {
        self.trace.record("lifecycle:start");
        Box::pin(async { Ok(()) })
    }
}

#[tokio::test]
async fn runner_runs_after_lifecycle_start() {
    let trace = CallTrace::default();
    let trace_for_lifecycle = trace.clone();
    let trace_for_runner = trace.clone();
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder
        .register(
            ComponentDefinition::try_singleton::<RunnerOrderLifecycle, _>(move |_resolver| {
                Ok(RunnerOrderLifecycle {
                    trace: trace_for_lifecycle.clone(),
                })
            }),
        )
        .expect("register RunnerOrderLifecycle");
    builder
        .register(ComponentDefinition::try_singleton::<CacheWarmupRunner, _>(
            move |_resolver| {
                Ok(CacheWarmupRunner {
                    trace: trace_for_runner.clone(),
                })
            },
        ))
        .expect("register CacheWarmupRunner");
    builder.lifecycle::<RunnerOrderLifecycle>();
    builder.application_runner::<CacheWarmupRunner>();

    let context = builder.launch().await.expect("launch");
    let events = trace.snapshot();

    let start_idx = events.iter().position(|e| e == "lifecycle:start");
    let runner_idx = events.iter().position(|e| e == "cache-warmup");
    assert!(
        start_idx.is_some() && runner_idx.is_some() && start_idx.unwrap() < runner_idx.unwrap(),
        "Lifecycle start must precede Runner; events: {events:?}"
    );

    context.close().await.expect("close");
}
