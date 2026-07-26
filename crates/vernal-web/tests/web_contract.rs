//! Vernal Web 请求上下文、请求 Scope、AOP 桥接和错误模型合同测试。

use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;
use vernal_beans::{ComponentDefinition, RegistryBuilder, ResolveError, ScopeKey};
use vernal_context::{ApplicationContextBuilder, ScopeCleanupPolicy, VernalApplicationBuilder};
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
        .expect("first resolution");
    let second = scope
        .get_or_insert_with(|| RequestScopedValue(99))
        .expect("cached resolution");

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.0, 0);
    assert_eq!(constructions.load(Ordering::Relaxed), 1);
    scope.close().await.expect("scope close");
    assert_eq!(scope.state(), ScopeState::Closed);
    assert!(matches!(
        scope.get_or_insert_with(|| RequestScopedValue(2)),
        Err(ScopeError::InvalidState {
            state: ScopeState::Closed,
            ..
        })
    ));
}

#[tokio::test]
async fn web_scope_resolves_ioc_request_components_and_isolates_sibling_requests() {
    let constructions = Arc::new(AtomicUsize::new(0));
    let factory_constructions = Arc::clone(&constructions);
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<
            RequestScopedValue,
            WebRequestScope,
            _,
        >(move |_| {
            RequestScopedValue(factory_constructions.fetch_add(1, Ordering::SeqCst))
        }))
        .expect("request component registration");
    let context = Arc::new(
        ApplicationContextBuilder::new(registry.build().expect("registry build"))
            .build()
            .expect("context build"),
    );
    context.refresh().await.expect("context refresh");
    context.start().await.expect("context start");

    // 同一请求两次解析必须命中 IoC ScopeContext 的同一个 OnceLock；兄弟请求则
    // 拥有彼此隔离的缓存和取消令牌。
    let first_scope = Arc::new(WebRequestScope::from_application_context(Arc::clone(
        &context,
    )));
    let first = first_scope.resolve::<RequestScopedValue>().expect("first");
    let repeated = first_scope
        .resolve::<RequestScopedValue>()
        .expect("repeated");
    assert!(Arc::ptr_eq(&first, &repeated));
    assert_eq!(first.0, 0);
    assert_eq!(first_scope.key(), ScopeKey::of::<WebRequestScope>());

    let sibling_scope = Arc::new(WebRequestScope::from_application_context(Arc::clone(
        &context,
    )));
    let sibling = sibling_scope
        .resolve::<RequestScopedValue>()
        .expect("sibling");
    assert!(!Arc::ptr_eq(&first, &sibling));
    assert_eq!(sibling.0, 1);
    assert_eq!(constructions.load(Ordering::SeqCst), 2);

    first_scope.close().await.expect("first scope close");
    assert!(matches!(
        first_scope.resolve::<RequestScopedValue>(),
        Err(ResolveError::ScopeUnavailable { .. })
    ));

    // 应用关闭取消整个作用域树，但兄弟 Scope 仍由响应所有者显式 close，以确保
    // 已登记的清理钩子不会因为父级取消而被跳过。
    context.close().await.expect("application close");
    assert!(sibling_scope.cancellation().is_cancelled());
    assert!(matches!(
        sibling_scope.resolve::<RequestScopedValue>(),
        Err(ResolveError::ScopeUnavailable {
            cancelled: true,
            ..
        })
    ));
    sibling_scope.close().await.expect("sibling scope close");
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
        .expect("first hook");
    let second_events = Arc::clone(&events);
    scope
        .on_close(move || async move {
            second_events.lock().await.push("second");
            Ok::<_, io::Error>(())
        })
        .expect("second hook");

    scope.close().await.expect("first close");
    scope.close().await.expect("idempotent close");
    assert!(cancellation.is_cancelled());
    assert_eq!(*events.lock().await, ["second", "first"]);
}

#[tokio::test]
async fn application_bound_scope_records_redacted_cleanup_failure() {
    let context = Arc::new(
        ApplicationContextBuilder::new(
            RegistryBuilder::new()
                .build()
                .expect("empty registry build"),
        )
        .build()
        .expect("context build"),
    );
    context.refresh().await.expect("context refresh");
    context.start().await.expect("context start");
    let scope = WebRequestScope::from_application_context(Arc::clone(&context));
    scope
        .on_close(|| async {
            Err::<(), _>(io::Error::other(
                "database password and request token must stay private",
            ))
        })
        .expect("failing close hook");

    let error = scope.close().await.expect_err("close hook must fail");
    assert!(matches!(error, ScopeError::CloseHook { .. }));
    assert_eq!(scope.state(), ScopeState::Closed);

    let report = context.startup_report().await;
    assert_eq!(
        report.warnings(),
        ["web.request-scope.cleanup-failed"],
        "application diagnostics must retain only a stable warning code"
    );
    assert!(
        report
            .warnings()
            .iter()
            .all(|warning| !warning.contains("password") && !warning.contains("token"))
    );
}

#[tokio::test]
async fn web_scope_inherits_timeout_without_cancelling_background_cleanup() {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder.scope_cleanup_policy(ScopeCleanupPolicy::bounded(Duration::from_millis(1)));
    let context = Arc::new(builder.build().expect("context build"));
    context.refresh().await.expect("context refresh");
    context.start().await.expect("context start");
    let scope = WebRequestScope::from_application_context(Arc::clone(&context));
    let release = Arc::new(Notify::new());
    let hook_release = Arc::clone(&release);
    scope
        .on_close(move || async move {
            hook_release.notified().await;
            Ok::<_, io::Error>(())
        })
        .expect("blocking hook");

    assert!(matches!(
        scope.close().await,
        Err(ScopeError::CloseTimeout { .. })
    ));
    assert_eq!(scope.state(), ScopeState::Closing);
    assert_eq!(
        context.startup_report().await.warnings(),
        ["web.request-scope.cleanup-failed"]
    );

    // 超时只结束 Web 调用者的等待；底层协调器继续持有并执行钩子。释放钩子后，
    // 使用无界内核入口加入同一关闭结果，证明没有启动第二次清理。
    release.notify_one();
    scope
        .scope_context()
        .close()
        .await
        .expect("background cleanup completion");
    assert_eq!(scope.state(), ScopeState::Closed);
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
