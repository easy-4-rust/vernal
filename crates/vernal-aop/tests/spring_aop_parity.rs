//! spring-aop 语义对齐测试套件。
//!
//! 对照 spring-aop 208 类的核心语义，验证 vernal-aop 的 Rust 实现行为等价。

use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use std::error::Error;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use vernal_aop::{
    Advisor, AnyPointcut, Aspect, AspectAdapter, AspectError, ComponentPointcut,
    DefaultPointcutAdvisor, Interceptor, IntroductionAdvisor, IntroductionInfo, Invocation,
    InvocationError, InvocationFuture, InvocationPlanBuilder, InvocationResult, InvocationValue,
    MethodPointcut, Next, Operation, OperationMetadata, OperationPointcut, Pointcut,
    PointcutAdvisor, PointcutExt, QualifierPointcut, TagPointcut,
};

// ============================================================================
// Test fixtures
// ============================================================================

struct CountingInterceptor {
    count: Arc<AtomicUsize>,
}

impl Interceptor for CountingInterceptor {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        self.count.fetch_add(1, Ordering::SeqCst);
        next.run(invocation)
    }
}

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

fn always() -> impl Fn(&Operation) -> bool {
    |_| true
}

fn noop_target() -> Arc<vernal_aop::InvocationTarget> {
    Arc::new(|_| Box::pin(async { Ok(Box::new(()) as InvocationValue) }))
}

// ============================================================================
// Aspect 四段式异步测试
// (对应 spring-aop: MethodBeforeAdvice / AfterReturningAdvice / ThrowsAdvice / MethodInterceptor)
// ============================================================================

#[tokio::test]
async fn aspect_before_runs_before_target() {
    let before_called = Arc::new(AtomicBool::new(false));
    let before_flag = Arc::clone(&before_called);

    struct BeforeAspect(Arc<AtomicBool>);

    impl Aspect for BeforeAspect {
        fn before<'a>(
            &'a self,
            _inv: &'a Invocation,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), InvocationError>> + Send + 'a>,
        > {
            self.0.store(true, Ordering::SeqCst);
            Box::pin(async { Ok(()) })
        }
    }

    let operation = Operation::new("Svc", "method");
    let mut builder = InvocationPlanBuilder::new();
    builder.register(Advisor::new(
        always(),
        AspectAdapter::new(BeforeAspect(before_flag)),
        0,
    ));
    let plan = builder.build(operation.clone());
    let target: Arc<vernal_aop::InvocationTarget> =
        Arc::new(|_| Box::pin(async { Ok(Box::new(42_i32) as InvocationValue) }));

    let result = plan
        .invoke(Invocation::new(operation).shared(), target)
        .await
        .unwrap();
    assert_eq!(*result.downcast::<i32>().unwrap(), 42);
    assert!(before_called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn aspect_after_runs_after_successful_target() {
    let after_called = Arc::new(AtomicBool::new(false));
    let after_flag = Arc::clone(&after_called);

    struct AfterAspect(Arc<AtomicBool>);

    impl Aspect for AfterAspect {
        fn after<'a>(
            &'a self,
            _inv: &'a Invocation,
            _result: &'a InvocationValue,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
            self.0.store(true, Ordering::SeqCst);
            Box::pin(async {})
        }
    }

    let operation = Operation::new("Svc", "method");
    let mut builder = InvocationPlanBuilder::new();
    builder.register(Advisor::new(
        always(),
        AspectAdapter::new(AfterAspect(after_flag)),
        0,
    ));
    let plan = builder.build(operation.clone());

    let target: Arc<vernal_aop::InvocationTarget> =
        Arc::new(|_| Box::pin(async { Ok(Box::new(42_i32) as InvocationValue) }));
    let _result = plan
        .invoke(Invocation::new(operation).shared(), target)
        .await
        .unwrap();
    assert!(after_called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn aspect_after_error_runs_on_target_failure() {
    let error_called = Arc::new(AtomicBool::new(false));
    let error_flag = Arc::clone(&error_called);

    struct ErrorAspect(Arc<AtomicBool>);

    impl Aspect for ErrorAspect {
        fn after_error<'a>(
            &'a self,
            _inv: &'a Invocation,
            _error: &'a InvocationError,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
            self.0.store(true, Ordering::SeqCst);
            Box::pin(async {})
        }
    }

    let operation = Operation::new("Svc", "method");
    let mut builder = InvocationPlanBuilder::new();
    builder.register(Advisor::new(
        always(),
        AspectAdapter::new(ErrorAspect(error_flag)),
        0,
    ));
    let plan = builder.build(operation.clone());
    let target: Arc<vernal_aop::InvocationTarget> = Arc::new(|_| {
        Box::pin(async { Err(InvocationError::target(io::Error::other("target failed"))) })
    });

    let _result = plan
        .invoke(Invocation::new(operation).shared(), target)
        .await;
    assert!(error_called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn aspect_around_default_implements_before_proceed_after_pattern() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = Arc::clone(&events);

    struct PatternAspect(Arc<Mutex<Vec<String>>>);

    impl Aspect for PatternAspect {
        fn before<'a>(
            &'a self,
            _inv: &'a Invocation,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), InvocationError>> + Send + 'a>,
        > {
            let events = Arc::clone(&self.0);
            Box::pin(async move {
                events.lock().await.push("before".to_string());
                Ok(())
            })
        }

        fn after<'a>(
            &'a self,
            _inv: &'a Invocation,
            _result: &'a InvocationValue,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
            let events = Arc::clone(&self.0);
            Box::pin(async move {
                events.lock().await.push("after".to_string());
            })
        }
    }

    let operation = Operation::new("Svc", "method");
    let mut builder = InvocationPlanBuilder::new();
    builder.register(Advisor::new(
        always(),
        AspectAdapter::new(PatternAspect(events_clone)),
        0,
    ));
    let plan = builder.build(operation.clone());
    let target: Arc<vernal_aop::InvocationTarget> =
        Arc::new(|_| Box::pin(async { Ok(Box::new(String::from("result")) as InvocationValue) }));

    let _result = plan
        .invoke(Invocation::new(operation).shared(), target)
        .await
        .unwrap();
    assert_eq!(*events.lock().await, ["before", "after"]);
}

#[tokio::test]
async fn aspect_around_default_before_error_short_circuits() {
    let after_called = Arc::new(AtomicBool::new(false));
    let after_flag = Arc::clone(&after_called);

    struct FailingBeforeAspect(Arc<AtomicBool>);

    impl Aspect for FailingBeforeAspect {
        fn before<'a>(
            &'a self,
            _inv: &'a Invocation,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), InvocationError>> + Send + 'a>,
        > {
            Box::pin(async { Err(InvocationError::Cancelled) })
        }

        fn after<'a>(
            &'a self,
            _inv: &'a Invocation,
            _result: &'a InvocationValue,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
            self.0.store(true, Ordering::SeqCst);
            Box::pin(async {})
        }
    }

    let operation = Operation::new("Svc", "method");
    let mut builder = InvocationPlanBuilder::new();
    builder.register(Advisor::new(
        always(),
        AspectAdapter::new(FailingBeforeAspect(after_flag)),
        0,
    ));
    let plan = builder.build(operation.clone());
    let target_called = Arc::new(AtomicBool::new(false));
    let target_flag = Arc::clone(&target_called);
    let target: Arc<vernal_aop::InvocationTarget> = Arc::new(move |_: Arc<Invocation>| {
        target_flag.store(true, Ordering::SeqCst);
        Box::pin(async { Ok(Box::new(()) as InvocationValue) })
    });

    let result = plan
        .invoke(Invocation::new(operation).shared(), target)
        .await;
    assert!(result.is_err());
    assert!(!target_called.load(Ordering::SeqCst));
    assert!(!after_called.load(Ordering::SeqCst));
}

// ============================================================================
// Advisor 链排序与 Pointcut 过滤
// ============================================================================

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
    let target: Arc<vernal_aop::InvocationTarget> = Arc::new(move |_: Arc<Invocation>| {
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
    assert_eq!(*result.downcast::<i32>().unwrap(), 40);
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

    assert!(matches!(
        plan.invoke(
            Invocation::new(Operation::new("OtherService", "run")).shared(),
            noop_target()
        )
        .await,
        Err(InvocationError::PlanMismatch { .. })
    ));
    assert_eq!(count.load(Ordering::Relaxed), 0);
}

// ============================================================================
// Interceptor 短路、转换和恢复
// ============================================================================

#[tokio::test]
async fn interceptor_can_short_circuit_without_running_target() {
    let called = Arc::new(AtomicBool::new(false));
    let mut builder = InvocationPlanBuilder::new();
    builder.register(Advisor::new(always(), ShortCircuitInterceptor, 0));
    let operation = Operation::new("AuthService", "check");
    let plan = builder.build(operation.clone());
    let target_called = Arc::clone(&called);
    let target: Arc<vernal_aop::InvocationTarget> = Arc::new(move |_: Arc<Invocation>| {
        target_called.store(true, Ordering::Relaxed);
        Box::pin(async { Ok(Box::new(200_u16) as InvocationValue) })
    });

    let result = plan
        .invoke(Invocation::new(operation).shared(), target)
        .await
        .expect("short circuit should return a value");
    assert_eq!(*result.downcast::<u16>().unwrap(), 401);
    assert!(!called.load(Ordering::Relaxed));
}

#[tokio::test]
async fn interceptor_can_transform_success_and_recover_target_error() {
    let operation = Operation::new("PriceService", "quote");
    let mut transform_builder = InvocationPlanBuilder::new();
    transform_builder.register(Advisor::new(always(), TransformInterceptor, 0));
    let transform_plan = transform_builder.build(operation.clone());
    let success_target: Arc<vernal_aop::InvocationTarget> =
        Arc::new(|_| Box::pin(async { Ok(Box::new(40_i32) as InvocationValue) }));
    let transformed = transform_plan
        .invoke(Invocation::new(operation).shared(), success_target)
        .await
        .expect("transform should succeed");
    assert_eq!(*transformed.downcast::<i32>().unwrap(), 42);

    let recovery_operation = Operation::new("RemoteService", "load");
    let mut recovery_builder = InvocationPlanBuilder::new();
    recovery_builder.register(Advisor::new(always(), RecoveryInterceptor, 0));
    let recovery_plan = recovery_builder.build(recovery_operation.clone());
    let failing_target: Arc<vernal_aop::InvocationTarget> = Arc::new(|_| {
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
    assert_eq!(*recovered.downcast::<String>().unwrap(), "fallback");
}

// ============================================================================
// 取消和截止时间
// ============================================================================

#[tokio::test]
async fn cancellation_and_deadline_stop_pending_chain() {
    let operation = Operation::new("JobService", "run");
    let plan = Arc::new(InvocationPlanBuilder::new().build(operation.clone()));
    let pending_target: Arc<vernal_aop::InvocationTarget> =
        Arc::new(|_| Box::pin(async { std::future::pending::<InvocationResult>().await }));

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
        .with_deadline(tokio::time::Instant::now() + Duration::from_millis(10))
        .shared();
    assert!(matches!(
        plan.invoke(deadline_invocation, pending_target).await,
        Err(InvocationError::DeadlineExceeded)
    ));
}

// ============================================================================
// 并发安全
// ============================================================================

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
    let target: Arc<vernal_aop::InvocationTarget> = Arc::new(|invocation: Arc<Invocation>| {
        Box::pin(async move { Ok(Box::new(invocation.id().get()) as InvocationValue) })
    });

    let mut tasks = tokio::task::JoinSet::new();
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

// ============================================================================
// DefaultPointcutAdvisor (对应 spring-aop: DefaultPointcutAdvisor)
// ============================================================================

#[test]
fn default_pointcut_advisor_matches_all_with_any_pointcut() {
    let advisor = DefaultPointcutAdvisor::new(AnyPointcut::new(), ShortCircuitInterceptor);
    let op = Operation::new("Svc", "method");
    assert!(advisor.pointcut().matches(&op));
}

#[test]
fn default_pointcut_advisor_respects_order() {
    let advisor =
        DefaultPointcutAdvisor::with_order(AnyPointcut::new(), ShortCircuitInterceptor, 42);
    assert_eq!(advisor.order(), 42);
}

#[test]
fn default_pointcut_advisor_with_component_pointcut() {
    let advisor = DefaultPointcutAdvisor::new(
        ComponentPointcut::new("OrderService"),
        ShortCircuitInterceptor,
    );

    let op1 = Operation::new("OrderService", "create");
    let op2 = Operation::new("UserService", "create");
    assert!(advisor.pointcut().matches(&op1));
    assert!(!advisor.pointcut().matches(&op2));
}

// ============================================================================
// Pointcut 代数 (对应 spring-aop: ComposablePointcut)
// ============================================================================

#[test]
fn and_pointcut_requires_both_sides() {
    let combined = ComponentPointcut::new("OrderService").and(MethodPointcut::new("create"));

    assert!(combined.matches(&Operation::new("OrderService", "create")));
    assert!(!combined.matches(&Operation::new("OrderService", "cancel")));
    assert!(!combined.matches(&Operation::new("UserService", "create")));
}

#[test]
fn or_pointcut_requires_either_side() {
    let combined = MethodPointcut::new("save").or(MethodPointcut::new("update"));

    assert!(combined.matches(&Operation::new("Svc", "save")));
    assert!(combined.matches(&Operation::new("Svc", "update")));
    assert!(!combined.matches(&Operation::new("Svc", "delete")));
}

#[test]
fn not_pointcut_inverts_match() {
    let negated = ComponentPointcut::new("SystemService").not();

    assert!(!negated.matches(&Operation::new("SystemService", "health")));
    assert!(negated.matches(&Operation::new("UserService", "create")));
}

#[test]
fn complex_pointcut_algebra() {
    let order_mutations = ComponentPointcut::new("OrderService")
        .and(MethodPointcut::new("create").or(MethodPointcut::new("cancel")));
    let not_health = OperationPointcut::new(Operation::new("System", "health")).not();
    let guarded = order_mutations.and(not_health);

    assert!(guarded.matches(&Operation::new("OrderService", "create")));
    assert!(guarded.matches(&Operation::new("OrderService", "cancel")));
    assert!(!guarded.matches(&Operation::new("OrderService", "find")));
    assert!(!guarded.matches(&Operation::new("System", "health")));
}

// ============================================================================
// DSL 切点表达式解析 (对应 spring-aop: AspectJExpressionPointcut)
// ============================================================================

use vernal_aop::pointcut::dsl::{
    NamePattern, PointcutExpr, PointcutMatcher, Visibility, parse_pointcut_expr,
};

#[test]
fn dsl_parse_execution_wildcard() {
    let pc = parse_pointcut_expr("execution(pub fn *(..))").unwrap();
    match pc {
        PointcutExpr::Execution(pattern) => {
            assert_eq!(pattern.visibility, Some(Visibility::Public));
            assert_eq!(pattern.name, NamePattern::Wildcard);
        }
        _ => panic!("Expected Execution"),
    }
}

#[test]
fn dsl_parse_within() {
    let pc = parse_pointcut_expr("within(crate::api)").unwrap();
    match pc {
        PointcutExpr::Within(pattern) => assert_eq!(pattern.path, "crate::api"),
        _ => panic!("Expected Within"),
    }
}

#[test]
fn dsl_parse_and_or_not() {
    let pc = parse_pointcut_expr("execution(pub fn *(..)) && within(crate::api)").unwrap();
    assert!(matches!(pc, PointcutExpr::And(_, _)));

    let pc = parse_pointcut_expr("!within(crate::internal)").unwrap();
    assert!(matches!(pc, PointcutExpr::Not(_)));
}

#[test]
fn dsl_parse_tag_and_qualifier() {
    let pc = parse_pointcut_expr("tag(secured)").unwrap();
    assert!(matches!(pc, PointcutExpr::Tag(_)));

    let pc = parse_pointcut_expr("qualifier(primary)").unwrap();
    assert!(matches!(pc, PointcutExpr::Qualifier(_)));
}

#[test]
fn dsl_matches_operation() {
    let pc = parse_pointcut_expr("within(OrderService)").unwrap();

    let op1 = Operation::new("OrderService", "create");
    assert!(pc.matches_operation(&op1));

    let op2 = Operation::new("UserService", "create");
    assert!(!pc.matches_operation(&op2));
}

// ============================================================================
// Tag 和 Qualifier 切点 (vernal 扩展能力)
// ============================================================================

#[test]
fn tag_pointcut_matches_operation_with_tag() {
    let pointcut = TagPointcut::new("secured").unwrap();

    let op = Operation::new("Svc", "method")
        .with_metadata(OperationMetadata::empty().with_tag("secured").unwrap());
    assert!(pointcut.matches(&op));

    let op2 = Operation::new("Svc", "method");
    assert!(!pointcut.matches(&op2));
}

#[test]
fn qualifier_pointcut_matches_operation_with_qualifier() {
    let pointcut = QualifierPointcut::new("primary").unwrap();

    let op = Operation::new("Svc", "method").with_metadata(
        OperationMetadata::empty()
            .with_qualifier("primary")
            .unwrap(),
    );
    assert!(pointcut.matches(&op));

    let op2 = Operation::new("Svc", "method").with_metadata(
        OperationMetadata::empty()
            .with_qualifier("secondary")
            .unwrap(),
    );
    assert!(!pointcut.matches(&op2));
}

// ============================================================================
// AspectError (对应 spring-aop: AopInvocationException)
// ============================================================================

#[test]
fn aspect_error_execution_display() {
    let err = AspectError::execution("test error");
    assert_eq!(err.to_string(), "Execution error: test error");
}

#[test]
fn aspect_error_with_source_cause_chain() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
    let err = AspectError::execution_with_source("read failed", io_err);
    assert!(err.source().is_some());
}

#[test]
fn aspect_error_weaving_display() {
    let err = AspectError::weaving("bad pointcut");
    assert_eq!(err.to_string(), "Weaving error: bad pointcut");
}

#[test]
fn aspect_error_from_string_conversion() {
    let err: AspectError = "error".into();
    assert!(matches!(err, AspectError::ExecutionError { .. }));
}

// ============================================================================
// IntroductionInfo / IntroductionAdvisor (对应 spring-aop)
// ============================================================================

struct TestIntroductionInfo;

impl IntroductionInfo for TestIntroductionInfo {
    fn interface_names(&self) -> &[&'static str] {
        &["Lockable", "TimeStamped"]
    }
}

#[test]
fn introduction_info_returns_interface_names() {
    let info = TestIntroductionInfo;
    assert_eq!(info.interface_names(), &["Lockable", "TimeStamped"]);
}

struct StaticPointcutAlways;

impl Pointcut for StaticPointcutAlways {
    fn matches(&self, _operation: &Operation) -> bool {
        true
    }
}

struct TestIntroductionAdvisor(StaticPointcutAlways);

impl IntroductionInfo for TestIntroductionAdvisor {
    fn interface_names(&self) -> &[&'static str] {
        &["Lockable"]
    }
}

impl IntroductionAdvisor for TestIntroductionAdvisor {
    fn class_filter(&self) -> &dyn Fn(&str) -> bool {
        &|_| true
    }

    fn order(&self) -> i32 {
        0
    }

    fn pointcut(&self) -> &dyn Pointcut {
        &self.0
    }
}

#[test]
fn introduction_advisor_has_class_filter_and_order() {
    let advisor = TestIntroductionAdvisor(StaticPointcutAlways);
    assert!((advisor.class_filter())("AnyClass"));
    assert_eq!(advisor.order(), 0);
    assert!(advisor.pointcut().matches(&Operation::new("Any", "method")));
}

// ============================================================================
// Context 在拦截器间传递 (vernal Tokio-first 特有能力)
// ============================================================================

#[tokio::test]
async fn typed_context_is_available_across_await_boundaries() {
    let operation = Operation::new("SessionService", "current_user");
    let plan = InvocationPlanBuilder::new().build(operation.clone());
    let invocation = Invocation::new(operation).shared();
    invocation
        .context()
        .insert::<String>(String::from("user-42"))
        .await;

    let target: Arc<vernal_aop::InvocationTarget> = Arc::new(|invocation: Arc<Invocation>| {
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
    assert_eq!(*result.downcast::<String>().unwrap(), "user-42");
}

// ============================================================================
// Catalog 延迟初始化和密封
// ============================================================================

#[test]
fn deferred_catalog_is_visible_to_clones_after_initialization() {
    let operation = Operation::new("ManagedService", "execute");
    let deferred = vernal_aop::InvocationPlanCatalog::deferred();
    let injected_clone = deferred.clone();
    assert!(injected_clone.is_empty());

    let compiled = InvocationPlanBuilder::new()
        .build_catalog([operation.clone()])
        .expect("operation declaration should compile");
    deferred
        .initialize_from(&compiled)
        .expect("first initialization");

    assert!(injected_clone.get(&operation).is_some());
    assert!(deferred.initialize_from(&compiled).is_err());
}
