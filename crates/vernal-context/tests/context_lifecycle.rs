//! Vernal 应用上下文的状态、顺序、回滚与并发关闭合同测试。

#[path = "lifecycle_support/cancellation_safe_phase_lifecycle.rs"]
mod cancellation_safe_phase_lifecycle;

use std::{io, sync::Arc};

use cancellation_safe_phase_lifecycle::CancellationSafePhaseLifecycle;
use tokio::{
    sync::{Mutex, Notify},
    task::yield_now,
    time::{Duration, timeout},
};
use vernal_context::{
    ApplicationContextBuilder, ContextError, ContextState, Lifecycle, LifecycleFuture,
    LifecyclePhase,
};
use vernal_core::BoxError;
use vernal_beans::{ComponentDefinition, RegistryBuilder};

type Events = Arc<Mutex<Vec<&'static str>>>;

struct Database {
    events: Events,
}

impl Lifecycle for Database {
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().await.push("database:initialize");
            Ok(())
        })
    }

    fn start(&self, _cancellation: tokio_util::sync::CancellationToken) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().await.push("database:start");
            Ok(())
        })
    }

    fn stop(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().await.push("database:stop");
            Ok(())
        })
    }
}

struct ApiService {
    events: Events,
    _database: Arc<Database>,
}

impl Lifecycle for ApiService {
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().await.push("api:initialize");
            Ok(())
        })
    }

    fn start(&self, _cancellation: tokio_util::sync::CancellationToken) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().await.push("api:start");
            Ok(())
        })
    }

    fn stop(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().await.push("api:stop");
            Ok(())
        })
    }
}

struct InitializeFailure {
    events: Events,
}

impl Lifecycle for InitializeFailure {
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().await.push("failure:initialize");
            Err(Box::new(io::Error::other("initialize failed")) as BoxError)
        })
    }

    fn stop(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().await.push("failure:stop");
            Ok(())
        })
    }
}

struct StartFailure {
    events: Events,
}

impl Lifecycle for StartFailure {
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().await.push("failure:initialize");
            Ok(())
        })
    }

    fn start(&self, _cancellation: tokio_util::sync::CancellationToken) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().await.push("failure:start");
            Err(Box::new(io::Error::other("start failed")) as BoxError)
        })
    }

    fn stop(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().await.push("failure:stop");
            Ok(())
        })
    }
}

fn happy_context(events: &Events) -> vernal_context::ApplicationContext {
    let mut registry = RegistryBuilder::new();
    let database_events = Arc::clone(events);
    registry
        .register(ComponentDefinition::singleton(move |_| Database {
            events: Arc::clone(&database_events),
        }))
        .expect("database definition");
    let api_events = Arc::clone(events);
    registry
        .register(
            ComponentDefinition::singleton(move |resolver| ApiService {
                events: Arc::clone(&api_events),
                _database: resolver.resolve().expect("declared database dependency"),
            })
            .depends_on::<Database>(),
        )
        .expect("api definition");

    let mut builder =
        ApplicationContextBuilder::new(registry.build().expect("valid dependency graph"));
    builder.lifecycle::<ApiService>().lifecycle::<Database>();
    builder.build().expect("valid lifecycle bindings")
}

#[tokio::test]
async fn lifecycle_follows_dependency_order_and_reverse_shutdown() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let context = happy_context(&events);
    assert_eq!(context.state().await, ContextState::Created);

    context.refresh().await.expect("refresh should succeed");
    assert_eq!(context.state().await, ContextState::Refreshed);
    assert_eq!(
        *events.lock().await,
        ["database:initialize", "api:initialize"]
    );

    context.start().await.expect("start should succeed");
    assert_eq!(context.state().await, ContextState::Ready);
    let cancellation = context.cancellation_token();
    assert!(!cancellation.is_cancelled());

    context.close().await.expect("close should succeed");
    assert!(cancellation.is_cancelled());
    assert_eq!(context.state().await, ContextState::Closed);
    assert_eq!(
        *events.lock().await,
        [
            "database:initialize",
            "api:initialize",
            "database:start",
            "api:start",
            "api:stop",
            "database:stop"
        ]
    );

    context.close().await.expect("close should be idempotent");
    assert_eq!(events.lock().await.len(), 6);
}

#[tokio::test]
async fn initialize_failure_rolls_back_partial_state() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut registry = RegistryBuilder::new();
    let database_events = Arc::clone(&events);
    registry
        .register(ComponentDefinition::singleton(move |_| Database {
            events: Arc::clone(&database_events),
        }))
        .expect("database definition");
    let failure_events = Arc::clone(&events);
    registry
        .register(
            ComponentDefinition::singleton(move |_| InitializeFailure {
                events: Arc::clone(&failure_events),
            })
            .depends_on::<Database>(),
        )
        .expect("failure definition");

    let mut builder =
        ApplicationContextBuilder::new(registry.build().expect("valid dependency graph"));
    builder
        .lifecycle::<InitializeFailure>()
        .lifecycle::<Database>();
    let context = builder.build().expect("valid lifecycle bindings");

    let error = context.refresh().await.expect_err("refresh should fail");
    assert!(matches!(
        error,
        ContextError::Lifecycle {
            phase: LifecyclePhase::Initialize,
            ..
        }
    ));
    assert_eq!(context.state().await, ContextState::Closed);
    assert_eq!(
        *events.lock().await,
        [
            "database:initialize",
            "failure:initialize",
            "failure:stop",
            "database:stop"
        ]
    );
}

#[tokio::test]
async fn start_failure_cancels_and_stops_all_initialized_components() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut registry = RegistryBuilder::new();
    let database_events = Arc::clone(&events);
    registry
        .register(ComponentDefinition::singleton(move |_| Database {
            events: Arc::clone(&database_events),
        }))
        .expect("database definition");
    let failure_events = Arc::clone(&events);
    registry
        .register(
            ComponentDefinition::singleton(move |_| StartFailure {
                events: Arc::clone(&failure_events),
            })
            .depends_on::<Database>(),
        )
        .expect("failure definition");

    let mut builder =
        ApplicationContextBuilder::new(registry.build().expect("valid dependency graph"));
    builder.lifecycle::<StartFailure>().lifecycle::<Database>();
    let context = builder.build().expect("valid lifecycle bindings");
    let cancellation = context.cancellation_token();
    context.refresh().await.expect("refresh should succeed");

    let error = context.start().await.expect_err("start should fail");
    assert!(matches!(
        error,
        ContextError::Lifecycle {
            phase: LifecyclePhase::Start,
            ..
        }
    ));
    assert!(cancellation.is_cancelled());
    assert_eq!(context.state().await, ContextState::Closed);
    assert_eq!(
        *events.lock().await,
        [
            "database:initialize",
            "failure:initialize",
            "database:start",
            "failure:start",
            "failure:stop",
            "database:stop"
        ]
    );
}

#[tokio::test]
async fn invalid_transition_is_rejected_without_mutating_state() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let context = happy_context(&events);
    assert!(matches!(
        context.start().await,
        Err(ContextError::InvalidState {
            state: ContextState::Created,
            ..
        })
    ));
    assert_eq!(context.state().await, ContextState::Created);
}

#[tokio::test]
async fn cancellation_before_start_rolls_back_refreshed_components() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let context = happy_context(&events);
    context.refresh().await.expect("refresh should succeed");
    context.cancellation_token().cancel();

    assert!(matches!(
        context.start().await,
        Err(ContextError::LifecycleCancelled { operation: "start" })
    ));
    assert_eq!(context.state().await, ContextState::Closed);
    assert_eq!(
        events.lock().await.as_slice(),
        [
            "database:initialize",
            "api:initialize",
            "api:stop",
            "database:stop"
        ]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_close_is_serialized_and_stops_components_once() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let context = Arc::new(happy_context(&events));
    context.refresh().await.expect("refresh should succeed");
    context.start().await.expect("start should succeed");

    let mut tasks = Vec::new();
    for _ in 0..16 {
        let context = Arc::clone(&context);
        tasks.push(tokio::spawn(async move { context.close().await }));
    }
    for task in tasks {
        task.await
            .expect("close task should join")
            .expect("close should succeed");
    }

    let events = events.lock().await;
    assert_eq!(
        events.iter().filter(|event| **event == "api:stop").count(),
        1
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| **event == "database:stop")
            .count(),
        1
    );
}

#[test]
fn builder_rejects_lifecycle_type_without_ioc_definition() {
    let registry = RegistryBuilder::new()
        .build()
        .expect("empty graph is valid");
    let mut builder = ApplicationContextBuilder::new(registry);
    builder.lifecycle::<Database>();
    assert!(matches!(
        builder.build(),
        Err(ContextError::LifecycleDefinitionNotFound { .. })
    ));
}

#[tokio::test]
async fn typed_events_are_broadcast_and_isolated_per_context() {
    #[derive(Debug, PartialEq, Eq)]
    struct UserCreated(u64);

    let first_events = Arc::new(Mutex::new(Vec::new()));
    let first = happy_context(&first_events);
    let second_events = Arc::new(Mutex::new(Vec::new()));
    let second = happy_context(&second_events);

    let mut first_receiver = first.events().subscribe::<UserCreated>().await;
    let mut second_receiver = second.events().subscribe::<UserCreated>().await;
    assert_eq!(first.events().publish(UserCreated(42)).await, 1);

    let received = first_receiver.recv().await.expect("first context event");
    assert_eq!(*received, UserCreated(42));
    assert!(second_receiver.try_recv().is_err());
}

#[tokio::test]
async fn cancelling_refresh_waiter_does_not_abandon_initialize_rollback() {
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let component = Arc::new(CancellationSafePhaseLifecycle::failing_initialize(
        Arc::clone(&entered),
        Arc::clone(&release),
    ));
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::shared_arc(Arc::clone(&component)))
        .expect("phase lifecycle definition");
    let mut builder =
        ApplicationContextBuilder::new(registry.build().expect("valid lifecycle graph"));
    builder.lifecycle::<CancellationSafePhaseLifecycle>();
    let context = Arc::new(builder.build().expect("context build"));

    let waiter = {
        let context = Arc::clone(&context);
        tokio::spawn(async move { context.refresh().await })
    };
    entered.notified().await;
    waiter.abort();
    assert!(
        waiter
            .await
            .expect_err("refresh waiter should be cancelled")
            .is_cancelled()
    );

    release.notify_one();
    timeout(Duration::from_secs(1), async {
        while context.state().await != ContextState::Closed {
            yield_now().await;
        }
    })
    .await
    .expect("background refresh rollback should reach Closed");
    assert!(component.stopped());
    assert!(context.cancellation_token().is_cancelled());
}

#[tokio::test]
async fn cancelling_start_waiter_does_not_abandon_start_rollback() {
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let component = Arc::new(CancellationSafePhaseLifecycle::failing_start(
        Arc::clone(&entered),
        Arc::clone(&release),
    ));
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::shared_arc(Arc::clone(&component)))
        .expect("phase lifecycle definition");
    let mut builder =
        ApplicationContextBuilder::new(registry.build().expect("valid lifecycle graph"));
    builder.lifecycle::<CancellationSafePhaseLifecycle>();
    let context = Arc::new(builder.build().expect("context build"));
    context.refresh().await.expect("refresh should succeed");

    let waiter = {
        let context = Arc::clone(&context);
        tokio::spawn(async move { context.start().await })
    };
    entered.notified().await;
    waiter.abort();
    assert!(
        waiter
            .await
            .expect_err("start waiter should be cancelled")
            .is_cancelled()
    );

    release.notify_one();
    timeout(Duration::from_secs(1), async {
        while context.state().await != ContextState::Closed {
            yield_now().await;
        }
    })
    .await
    .expect("background start rollback should reach Closed");
    assert!(component.stopped());
    assert!(context.cancellation_token().is_cancelled());
}
