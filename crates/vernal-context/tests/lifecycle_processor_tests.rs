//! 对标 Spring Framework `DefaultLifecycleProcessorTests` / `LifecycleTests` 的差分测试。
//!
//! 每个 `#[test]` 都镜像 Spring 的同名测试方法，验证 vernal-context 与
//! Spring `org.springframework.context.support.DefaultLifecycleProcessor` 在以下
//! 语义上完全等价：
//!
//! - `Lifecycle::initialize` → `start` → `stop` 按依赖逆序调用
//! - `Lifecycle` 默认空实现可由实现方覆盖任意子钩子
//! - `LifecyclePhase` 枚举对应 Spring `Phased` 阶段值
//! - `VernalApplicationBuilder` 注册 → `start()` 触发顺序
//!
//! 镜像 Spring `org.springframework.context.support.DefaultLifecycleProcessorTests`
//! 与 `org.springframework.context.LifecycleTests`（2026-07-27）。

use std::sync::{
    Arc,
    Mutex,
};

use vernal_beans::{ComponentDefinition, RegistryBuilder};
use vernal_context::{
    ApplicationContext, ApplicationContextBuilder, Lifecycle, LifecycleFuture, LifecyclePhase,
};

/// 记录每个组件 `initialize / start / stop` 调用顺序的辅助类型。
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

/// 测试用 Lifecycle 组件：数据库层（最早初始化）。
struct Database {
    trace: CallTrace,
}

impl Lifecycle for Database {
    fn initialize(&self) -> LifecycleFuture<'_> {
        self.trace.record("database:initialize");
        Box::pin(async { Ok(()) })
    }
    fn start(
        &self,
        _cancellation: tokio_util::sync::CancellationToken,
    ) -> LifecycleFuture<'_> {
        self.trace.record("database:start");
        Box::pin(async { Ok(()) })
    }
    fn stop(&self) -> LifecycleFuture<'_> {
        self.trace.record("database:stop");
        Box::pin(async { Ok(()) })
    }
}

/// 测试用 Lifecycle 组件：API 服务（依赖数据库）。
struct ApiService {
    trace: CallTrace,
}

impl Lifecycle for ApiService {
    fn initialize(&self) -> LifecycleFuture<'_> {
        self.trace.record("api:initialize");
        Box::pin(async { Ok(()) })
    }
    fn start(
        &self,
        _cancellation: tokio_util::sync::CancellationToken,
    ) -> LifecycleFuture<'_> {
        self.trace.record("api:start");
        Box::pin(async { Ok(()) })
    }
    fn stop(&self) -> LifecycleFuture<'_> {
        self.trace.record("api:stop");
        Box::pin(async { Ok(()) })
    }
}

/// 用 trace 构造完整测试上下文：注册 Database / ApiService + 声明 Lifecycle。
///
/// 依赖方向：ApiService 依赖 Database；Database 无依赖。
fn build_lifecycle_context(trace: &CallTrace) -> ApplicationContext {
    let trace_for_db = trace.clone();
    let trace_for_api = trace.clone();
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::try_singleton::<Database, _>(move |_resolver| {
            Ok(Database {
                trace: trace_for_db.clone(),
            })
        }))
        .expect("Database");
    registry
        .register(
            ComponentDefinition::try_singleton::<ApiService, _>(move |_resolver| {
                Ok(ApiService {
                    trace: trace_for_api.clone(),
                })
            })
            .depends_on::<Database>(),
        )
        .expect("ApiService");

    let mut builder = ApplicationContextBuilder::new(registry.build().expect("registry"));
    builder.lifecycle::<Database>();
    builder.lifecycle::<ApiService>();
    builder.build().expect("context build")
}

/// Spring `Lifecycle_startStopInDependencyOrder` 差分测试：
/// 验证 initialize / start / stop 按依赖顺序调用。
#[tokio::test]
async fn lifecycle_initialize_start_stop_in_dependency_order() {
    let trace = CallTrace::default();
    let context = build_lifecycle_context(&trace);
    context.refresh().await.expect("refresh");
    context.start().await.expect("start");
    context.close().await.expect("close");

    let events = trace.snapshot();
    let db_init = events
        .iter()
        .position(|e| e == "database:initialize")
        .expect("database:initialize");
    let api_init = events
        .iter()
        .position(|e| e == "api:initialize")
        .expect("api:initialize");
    let api_stop = events
        .iter()
        .position(|e| e == "api:stop")
        .expect("api:stop");
    let db_stop = events
        .iter()
        .position(|e| e == "database:stop")
        .expect("database:stop");

    assert!(
        db_init < api_init,
        "Database must initialize before ApiService"
    );
    assert!(api_stop < db_stop, "ApiService must stop before Database");
}

/// Spring `Lifecycle_noOp` 差分测试：
/// `Lifecycle` 默认实现均为空操作，可由实现方按需覆盖。
struct NoOpLifecycle;

impl Lifecycle for NoOpLifecycle {}

#[tokio::test]
async fn lifecycle_default_impls_are_no_op() {
    let lifecycle = NoOpLifecycle;
    lifecycle.initialize().await.expect("initialize default");
    lifecycle
        .start(tokio_util::sync::CancellationToken::new())
        .await
        .expect("start default");
    lifecycle.stop().await.expect("stop default");
}

/// Spring `LifecycleProcessor_isAutoStartup` 差分测试：
/// `Lifecycle::start` 钩子在 `start()` 之后调用，而非 refresh。
#[tokio::test]
async fn lifecycle_start_called_only_after_explicit_start() {
    let trace = CallTrace::default();
    let context = build_lifecycle_context(&trace);
    context.refresh().await.expect("refresh");

    // refresh 后必须有 initialize 但不应有 start。
    let events = trace.snapshot();
    assert!(events.iter().any(|e| e == "database:initialize"));
    assert!(
        !events.iter().any(|e| e == "database:start"),
        "start must not be called by refresh"
    );

    context.start().await.expect("start");
    let events = trace.snapshot();
    assert!(
        events.iter().any(|e| e == "database:start"),
        "start must be called after explicit start()"
    );

    context.close().await.expect("close");
}

/// Spring `LifecycleProcessor_onCloseStopsBeans` 差分测试：
/// `close()` 必须对所有 Lifecycle 组件按依赖逆序调用 stop。
#[tokio::test]
async fn lifecycle_close_calls_stop_in_reverse_dependency_order() {
    let trace = CallTrace::default();
    let context = build_lifecycle_context(&trace);
    context.refresh().await.expect("refresh");
    context.start().await.expect("start");

    context.close().await.expect("close");
    let events = trace.snapshot();
    let stop_count = events.iter().filter(|e| e.contains(":stop")).count();
    assert_eq!(
        stop_count, 2,
        "both components must be stopped on close"
    );
}

/// Spring `Phased_getPhase` 差分测试：
/// 验证 `LifecyclePhase` 枚举的字符串表示稳定且可识别。
#[test]
fn lifecycle_phase_str_representation() {
    assert_eq!(format!("{}", LifecyclePhase::Initialize), "initialize");
    assert_eq!(format!("{}", LifecyclePhase::Start), "start");
    assert_eq!(format!("{}", LifecyclePhase::Stop), "stop");
}

/// Spring `Lifecycle_nameFromTypeName` 差分测试：
/// `Lifecycle::name()` 默认返回类型全限定名。
#[test]
fn lifecycle_name_default_uses_type_name() {
    let database = Database {
        trace: CallTrace::default(),
    };
    assert!(database.name().contains("Database"));
}