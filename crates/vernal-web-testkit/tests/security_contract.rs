//! 安全桥框架中立 Send/Local-AOP 合同测试。

use std::{
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use tokio_util::sync::CancellationToken;
use vernal_aop::{
    Advisor, AnyPointcut, Invocation, InvocationPlanBuilder, InvocationTarget, InvocationValue,
    LocalAdvisor, LocalInvocationError, LocalInvocationPlanBuilder, LocalInvocationTarget,
    LocalInvocationValue, Operation,
};
use vernal_web::{HandlerInvocation, RequestContext, RouteMetadata, WebFailure, WebRequestScope};
use vernal_web_testkit::SecurityContractInterceptor;

async fn invocation(operation: &Operation) -> (Arc<Invocation>, Arc<RequestContext>) {
    let request_context = Arc::new(RequestContext::new(
        RouteMetadata::new(
            operation.component(),
            operation.method(),
            "/security-contract",
        ),
        CancellationToken::new(),
    ));
    let scope = Arc::new(WebRequestScope::new(request_context.cancellation().clone()));
    let invocation = HandlerInvocation::new(Arc::clone(&request_context), scope)
        .aop_invocation()
        .await;
    (invocation, request_context)
}

#[tokio::test]
async fn authenticated_send_contract_projects_principal_before_target() {
    let operation = Operation::new("security_handler", "allowed");
    let mut builder = InvocationPlanBuilder::new();
    builder.register(Advisor::new(
        AnyPointcut::new(),
        SecurityContractInterceptor::authenticated("user-42", ["admin", "operator"]),
        -1000,
    ));
    let plan = builder.build(operation.clone());
    let (invocation, request_context) = invocation(&operation).await;
    let target: Arc<InvocationTarget> = Arc::new(|invocation| {
        Box::pin(async move {
            let context = invocation
                .context()
                .get::<Arc<RequestContext>>()
                .await
                .expect("security contract request context");
            let principal = context
                .principal()
                .await
                .expect("authenticated principal must reach target");
            Ok(Box::new(principal.subject().to_owned()) as InvocationValue)
        })
    });

    let value = plan
        .invoke(invocation, target)
        .await
        .expect("authenticated security contract");
    assert_eq!(
        *value.downcast::<String>().expect("target subject"),
        "user-42"
    );
    let principal = request_context
        .principal()
        .await
        .expect("request principal");
    assert!(principal.has_role("admin"));
    assert!(principal.has_role("operator"));
}

#[tokio::test]
async fn anonymous_send_contract_short_circuits_with_redacted_401() {
    let operation = Operation::new("security_handler", "authenticated_only");
    let mut builder = InvocationPlanBuilder::new();
    builder.register(Advisor::new(
        AnyPointcut::new(),
        SecurityContractInterceptor::unauthenticated(),
        -1000,
    ));
    let plan = builder.build(operation.clone());
    let (invocation, request_context) = invocation(&operation).await;
    let target_called = Arc::new(AtomicBool::new(false));
    let called = Arc::clone(&target_called);
    let target: Arc<InvocationTarget> = Arc::new(move |_invocation| {
        let called = Arc::clone(&called);
        Box::pin(async move {
            called.store(true, Ordering::SeqCst);
            Ok(Box::new(()) as InvocationValue)
        })
    });

    let error = plan
        .invoke(invocation, target)
        .await
        .expect_err("anonymous request must be rejected");
    let failure = error
        .into_target::<WebFailure>()
        .expect("security denial must remain a WebFailure");
    assert_eq!(failure.problem().status(), 401);
    assert_eq!(failure.problem().title(), "Authentication is required");
    assert!(!target_called.load(Ordering::SeqCst));
    assert!(request_context.principal().await.is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn authenticated_local_contract_retains_principal_and_short_circuits_with_403() {
    let operation = Operation::new("local_security_handler", "admin_only");
    let mut builder = LocalInvocationPlanBuilder::new();
    builder.register(LocalAdvisor::new(
        AnyPointcut::new(),
        SecurityContractInterceptor::forbidden("operator-7", ["operator"]),
        -1000,
    ));
    let plan = builder.build(operation.clone());
    let (invocation, request_context) = invocation(&operation).await;
    let target_called = Rc::new(AtomicBool::new(false));
    let called = Rc::clone(&target_called);
    let target: Rc<LocalInvocationTarget> = Rc::new(move |_invocation| {
        let called = Rc::clone(&called);
        Box::pin(async move {
            called.store(true, Ordering::SeqCst);
            Ok(Box::new(()) as LocalInvocationValue)
        })
    });

    let error = plan
        .invoke(invocation, target)
        .await
        .expect_err("authenticated user without permission must be rejected");
    let failure = match error {
        LocalInvocationError::Target { source } => source
            .downcast::<WebFailure>()
            .expect("local security denial must remain a WebFailure"),
        other => panic!("unexpected local security error: {other}"),
    };
    assert_eq!(failure.problem().status(), 403);
    assert_eq!(failure.problem().title(), "Access is forbidden");
    assert!(!target_called.load(Ordering::SeqCst));
    let principal = request_context
        .principal()
        .await
        .expect("forbidden authenticated principal");
    assert_eq!(principal.subject(), "operator-7");
    assert!(principal.has_role("operator"));
}
