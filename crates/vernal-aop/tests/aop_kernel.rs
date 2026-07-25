//! Vernal AOP 内核的顺序、短路、改写、取消和并发契约测试。

#[path = "aop_support/borrowed_target.rs"]
mod borrowed_target;

use std::{
    future::pending,
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use borrowed_target::BorrowedTarget;
use tokio::{sync::Mutex, task::JoinSet, time::Instant};
use tokio_util::sync::CancellationToken;
use vernal_aop::{
    Advisor, BorrowedInvocationFutureTarget, BorrowedInvocationTarget, Interceptor, Invocation,
    InvocationError, InvocationFuture, InvocationPlanBuilder, InvocationResult, InvocationTarget,
    InvocationValue, Next, Operation,
};

struct RecordingInterceptor {
    name: &'static str,
    events: Arc<Mutex<Vec<String>>>,
}

impl Interceptor for RecordingInterceptor {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            self.events
                .lock()
                .await
                .push(format!("{}:before", self.name));
            let result = next.run(invocation).await;
            self.events
                .lock()
                .await
                .push(format!("{}:after", self.name));
            result
        })
    }
}

struct ShortCircuitInterceptor;

impl Interceptor for ShortCircuitInterceptor {
    fn intercept<'a>(
        &'a self,
        _invocation: Arc<Invocation>,
        _next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async { Ok(Box::new(401_u16) as InvocationValue) })
    }
}

struct TransformInterceptor;

impl Interceptor for TransformInterceptor {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            let value = next.run(invocation).await?;
            let value = value.downcast::<i32>().map_err(|_| {
                InvocationError::target(io::Error::other("expected i32 in transform"))
            })?;
            Ok(Box::new(*value + 2) as InvocationValue)
        })
    }
}

struct RecoveryInterceptor;

impl Interceptor for RecoveryInterceptor {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            match next.run(invocation).await {
                Ok(value) => Ok(value),
                Err(InvocationError::Target { .. }) => {
                    Ok(Box::new(String::from("fallback")) as InvocationValue)
                }
                Err(error) => Err(error),
            }
        })
    }
}

struct CountingInterceptor {
    count: Arc<AtomicUsize>,
}

impl Interceptor for CountingInterceptor {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        self.count.fetch_add(1, Ordering::Relaxed);
        next.run(invocation)
    }
}

fn always() -> impl Fn(&Operation) -> bool {
    |_| true
}

#[tokio::test]
async fn borrowed_send_target_stays_inside_current_call_lifetime() {
    let operation = Operation::new("SalvoEndpoint", "GET");
    let plan = InvocationPlanBuilder::new().build(operation.clone());
    let mut events = Vec::new();
    let mut target = BorrowedTarget::new(&mut events);

    let value = plan
        .invoke_borrowed(
            Invocation::new(operation).shared(),
            &mut target as &mut dyn BorrowedInvocationTarget,
        )
        .await
        .expect("borrowed Send target");
    let value = value.downcast::<String>().expect("string result");

    assert_eq!(value.as_str(), "borrowed-result");
    assert_eq!(events, ["borrowed-target:before", "borrowed-target:after"]);
}

#[tokio::test]
async fn borrowed_future_target_owns_arguments_and_rejects_second_execution() {
    let operation = Operation::new("BorrowedService", "execute");
    let plan = InvocationPlanBuilder::new().build(operation.clone());
    let borrowed_value = String::from("current-call");
    let future: InvocationFuture<'_> =
        Box::pin(async { Ok(Box::new(borrowed_value.len()) as InvocationValue) });
    let mut target = BorrowedInvocationFutureTarget::new(operation.clone(), future);

    let value = plan
        .invoke_borrowed(Invocation::new(operation.clone()).shared(), &mut target)
        .await
        .expect("borrowed future target");
    assert_eq!(*value.downcast::<usize>().expect("usize result"), 12);

    let error = plan
        .invoke_borrowed(Invocation::new(operation.clone()).shared(), &mut target)
        .await
        .expect_err("borrowed future target must execute once");
    assert!(matches!(
        error,
        InvocationError::TargetAlreadyInvoked { operation: actual }
            if actual == operation
    ));
}

#[test]
fn catalog_precompiles_matching_advisors_and_coalesces_duplicate_operations() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut builder = InvocationPlanBuilder::new();
    builder.register(Advisor::new(
        always(),
        CountingInterceptor {
            count: Arc::clone(&count),
        },
        0,
    ));
    let operation = Operation::new("CatalogService", "execute");

    let catalog = builder.build_catalog([operation.clone(), operation.clone()]);

    assert_eq!(catalog.len(), 1);
    assert_eq!(
        catalog
            .get(&operation)
            .expect("catalog plan should exist")
            .len(),
        1
    );
}

#[tokio::test]
async fn lower_order_enters_first_and_exits_last() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut builder = InvocationPlanBuilder::new();
    builder
        .register(Advisor::new(
            always(),
            RecordingInterceptor {
                name: "late",
                events: Arc::clone(&events),
            },
            20,
        ))
        .register(Advisor::new(
            always(),
            RecordingInterceptor {
                name: "early",
                events: Arc::clone(&events),
            },
            10,
        ));

    let operation = Operation::new("OrderService", "create");
    let plan = builder.build(operation.clone());
    let target_events = Arc::clone(&events);
    let target: Arc<InvocationTarget> = Arc::new(move |_| {
        let target_events = Arc::clone(&target_events);
        Box::pin(async move {
            target_events.lock().await.push(String::from("target"));
            Ok(Box::new(40_i32) as InvocationValue)
        })
    });

    let result = plan
        .invoke(Invocation::new(operation).shared(), target)
        .await
        .expect("ordered chain should succeed");
    assert_eq!(*result.downcast::<i32>().expect("i32 result"), 40);
    assert_eq!(
        *events.lock().await,
        [
            "early:before",
            "late:before",
            "target",
            "late:after",
            "early:after"
        ]
    );
}

#[tokio::test]
async fn interceptor_can_short_circuit_without_running_target() {
    let called = Arc::new(AtomicBool::new(false));
    let mut builder = InvocationPlanBuilder::new();
    builder.register(Advisor::new(always(), ShortCircuitInterceptor, 0));
    let operation = Operation::new("AuthService", "check");
    let plan = builder.build(operation.clone());
    let target_called = Arc::clone(&called);
    let target: Arc<InvocationTarget> = Arc::new(move |_| {
        target_called.store(true, Ordering::Relaxed);
        Box::pin(async { Ok(Box::new(200_u16) as InvocationValue) })
    });

    let result = plan
        .invoke(Invocation::new(operation).shared(), target)
        .await
        .expect("short circuit should return a value");
    assert_eq!(*result.downcast::<u16>().expect("u16 result"), 401);
    assert!(!called.load(Ordering::Relaxed));
}

#[tokio::test]
async fn interceptor_can_transform_success_and_recover_target_error() {
    let operation = Operation::new("PriceService", "quote");
    let mut transform_builder = InvocationPlanBuilder::new();
    transform_builder.register(Advisor::new(always(), TransformInterceptor, 0));
    let transform_plan = transform_builder.build(operation.clone());
    let success_target: Arc<InvocationTarget> =
        Arc::new(|_| Box::pin(async { Ok(Box::new(40_i32) as InvocationValue) }));
    let transformed = transform_plan
        .invoke(Invocation::new(operation).shared(), success_target)
        .await
        .expect("transform should succeed");
    assert_eq!(*transformed.downcast::<i32>().expect("i32 result"), 42);

    let recovery_operation = Operation::new("RemoteService", "load");
    let mut recovery_builder = InvocationPlanBuilder::new();
    recovery_builder.register(Advisor::new(always(), RecoveryInterceptor, 0));
    let recovery_plan = recovery_builder.build(recovery_operation.clone());
    let failing_target: Arc<InvocationTarget> = Arc::new(|_| {
        Box::pin(async {
            Err(InvocationError::target(io::Error::other(
                "remote unavailable",
            )))
        })
    });
    let recovered = recovery_plan
        .invoke(Invocation::new(recovery_operation).shared(), failing_target)
        .await
        .expect("recovery should replace target error");
    assert_eq!(
        *recovered.downcast::<String>().expect("string fallback"),
        "fallback"
    );
}

#[tokio::test]
async fn typed_context_is_available_across_await_boundaries() {
    let operation = Operation::new("SessionService", "current_user");
    let plan = InvocationPlanBuilder::new().build(operation.clone());
    let invocation = Invocation::new(operation).shared();
    invocation
        .context()
        .insert::<String>(String::from("user-42"))
        .await;

    let target: Arc<InvocationTarget> = Arc::new(|invocation| {
        Box::pin(async move {
            tokio::task::yield_now().await;
            let subject = invocation
                .context()
                .get::<String>()
                .await
                .expect("subject should survive await");
            Ok(Box::new(subject) as InvocationValue)
        })
    });

    let result = plan
        .invoke(invocation, target)
        .await
        .expect("context read should succeed");
    assert_eq!(
        *result.downcast::<String>().expect("string subject"),
        "user-42"
    );
}

#[test]
fn deferred_catalog_is_visible_to_existing_clones_after_exactly_one_initialization() {
    let operation = Operation::new("ManagedService", "execute");
    let deferred = vernal_aop::InvocationPlanCatalog::deferred();
    let injected_clone = deferred.clone();
    assert!(injected_clone.is_empty());

    let compiled = InvocationPlanBuilder::new().build_catalog([operation.clone()]);
    deferred
        .initialize_from(&compiled)
        .expect("first initialization");

    assert!(
        injected_clone.get(&operation).is_some(),
        "a clone injected before IoC advisor resolution must observe the sealed plans"
    );
    assert!(
        deferred.initialize_from(&compiled).is_err(),
        "runtime replacement of an already published plan catalog must be rejected"
    );
}

#[tokio::test]
async fn cancellation_and_deadline_stop_pending_chain() {
    let operation = Operation::new("JobService", "run");
    let plan = Arc::new(InvocationPlanBuilder::new().build(operation.clone()));
    let pending_target: Arc<InvocationTarget> =
        Arc::new(|_| Box::pin(async { pending::<InvocationResult>().await }));

    let cancellation = CancellationToken::new();
    let invocation = Invocation::new(operation.clone())
        .with_cancellation(cancellation.clone())
        .shared();
    let cancel_plan = Arc::clone(&plan);
    let cancel_target = Arc::clone(&pending_target);
    let task = tokio::spawn(async move { cancel_plan.invoke(invocation, cancel_target).await });
    tokio::task::yield_now().await;
    cancellation.cancel();
    assert!(matches!(
        task.await.expect("task should join"),
        Err(InvocationError::Cancelled)
    ));

    let deadline_invocation = Invocation::new(operation)
        .with_deadline(Instant::now() + Duration::from_millis(10))
        .shared();
    assert!(matches!(
        plan.invoke(deadline_invocation, pending_target).await,
        Err(InvocationError::DeadlineExceeded)
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn immutable_plan_is_safe_under_concurrent_tokio_tasks() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut builder = InvocationPlanBuilder::new();
    builder.register(Advisor::new(
        always(),
        CountingInterceptor {
            count: Arc::clone(&count),
        },
        0,
    ));
    let operation = Operation::new("ConcurrentService", "execute");
    let plan = Arc::new(builder.build(operation.clone()));
    let target: Arc<InvocationTarget> = Arc::new(|invocation| {
        Box::pin(async move { Ok(Box::new(invocation.id().get()) as InvocationValue) })
    });

    let mut tasks = JoinSet::new();
    for _ in 0..64 {
        let plan = Arc::clone(&plan);
        let target = Arc::clone(&target);
        let invocation = Invocation::new(operation.clone()).shared();
        tasks.spawn(async move { plan.invoke(invocation, target).await });
    }

    let mut completed = 0;
    while let Some(result) = tasks.join_next().await {
        result
            .expect("task should join")
            .expect("invocation should succeed");
        completed += 1;
    }
    assert_eq!(completed, 64);
    assert_eq!(count.load(Ordering::Relaxed), 64);
}

#[tokio::test]
async fn pointcut_filters_advisors_and_plan_rejects_wrong_operation() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut builder = InvocationPlanBuilder::new();
    builder.register(Advisor::new(
        |operation: &Operation| operation.component() == "MatchedService",
        CountingInterceptor {
            count: Arc::clone(&count),
        },
        0,
    ));

    let matched = Operation::new("MatchedService", "run");
    let plan = builder.build(matched.clone());
    assert_eq!(plan.len(), 1);
    let target: Arc<InvocationTarget> =
        Arc::new(|_| Box::pin(async { Ok(Box::new(()) as InvocationValue) }));
    assert!(matches!(
        plan.invoke(
            Invocation::new(Operation::new("OtherService", "run")).shared(),
            target
        )
        .await,
        Err(InvocationError::PlanMismatch { .. })
    ));
    assert_eq!(count.load(Ordering::Relaxed), 0);
}
