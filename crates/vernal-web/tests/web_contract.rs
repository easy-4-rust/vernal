//! Vernal Web 请求上下文、请求 Scope、AOP 桥接和错误模型合同测试。

use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use vernal_web::{
    HandlerInvocation, ProblemDetails, ProblemKind, RequestContext, RouteMetadata, ScopeError,
    ScopeState, SecurityPrincipal, WebRequestScope,
};

struct RequestScopedValue(usize);

#[tokio::test]
async fn request_scope_caches_by_type_and_rejects_access_after_close() {
    let scope = WebRequestScope::new(CancellationToken::new());
    let constructions = Arc::new(AtomicUsize::new(0));

    let first_counter = Arc::clone(&constructions);
    let first = scope
        .get_or_insert_with(move || {
            RequestScopedValue(first_counter.fetch_add(1, Ordering::Relaxed))
        })
        .await
        .expect("first resolution");
    let second = scope
        .get_or_insert_with(|| RequestScopedValue(99))
        .await
        .expect("cached resolution");

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.0, 0);
    assert_eq!(constructions.load(Ordering::Relaxed), 1);
    scope.close().await.expect("scope close");
    assert_eq!(scope.state().await, ScopeState::Closed);
    assert!(matches!(
        scope.get_or_insert_with(|| RequestScopedValue(2)).await,
        Err(ScopeError::InvalidState {
            state: ScopeState::Closed,
            ..
        })
    ));
}

#[tokio::test]
async fn close_hooks_run_in_reverse_and_close_is_idempotent() {
    let cancellation = CancellationToken::new();
    let scope = WebRequestScope::new(cancellation.clone());
    let events = Arc::new(Mutex::new(Vec::new()));

    let first_events = Arc::clone(&events);
    scope
        .on_close(move || async move {
            first_events.lock().await.push("first");
            Ok::<_, io::Error>(())
        })
        .await
        .expect("first hook");
    let second_events = Arc::clone(&events);
    scope
        .on_close(move || async move {
            second_events.lock().await.push("second");
            Ok::<_, io::Error>(())
        })
        .await
        .expect("second hook");

    scope.close().await.expect("first close");
    scope.close().await.expect("idempotent close");
    assert!(cancellation.is_cancelled());
    assert_eq!(*events.lock().await, ["second", "first"]);
}

#[tokio::test]
async fn request_context_carries_principal_and_builds_aop_invocation() {
    let cancellation = CancellationToken::new();
    let context = Arc::new(RequestContext::new(
        RouteMetadata::new("OrderHandler", "create", "/orders"),
        cancellation.clone(),
    ));
    let principal = Arc::new(SecurityPrincipal::new("user-42", ["admin", "operator"]));
    context.set_principal(Some(Arc::clone(&principal))).await;
    let scope = Arc::new(WebRequestScope::new(cancellation.clone()));
    let handler = HandlerInvocation::new(Arc::clone(&context), Arc::clone(&scope));

    let invocation = handler.aop_invocation().await;
    assert_eq!(invocation.operation().to_string(), "OrderHandler::create");
    assert!(Arc::ptr_eq(
        &invocation
            .context()
            .get::<Arc<RequestContext>>()
            .await
            .expect("request context extension"),
        &context
    ));
    assert!(Arc::ptr_eq(
        &invocation
            .context()
            .get::<Arc<WebRequestScope>>()
            .await
            .expect("request scope extension"),
        &scope
    ));
    assert!(principal.has_role("admin"));
    assert_eq!(
        context.principal().await.expect("principal").subject(),
        "user-42"
    );

    cancellation.cancel();
    assert!(invocation.cancellation().is_cancelled());
}

#[test]
fn problem_details_keeps_stable_classification_and_safe_text() {
    let problem = ProblemDetails::new(ProblemKind::PolicyDenied, 403, "Access denied")
        .with_detail("The current principal cannot access this operation")
        .with_instance("request-42");
    assert_eq!(problem.kind(), ProblemKind::PolicyDenied);
    assert_eq!(problem.status(), 403);
    assert_eq!(problem.title(), "Access denied");
    assert_eq!(problem.instance(), Some("request-42"));
}
