//! Axum Router、Extractor、IoC 组件和请求作用域集成测试。

use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, Response, StatusCode},
    routing::get,
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use vernal_aop::{Advisor, Operation};
use vernal_axum::{
    VernalComponent, VernalContext, VernalRequestContext, VernalRequestScope, VernalRouterExt,
};
use vernal_context::{ApplicationContextBuilder, VernalApplicationBuilder};
use vernal_http::HttpRequestSnapshot;
use vernal_beans::{ComponentDefinition, RegistryBuilder};
use vernal_web::WebRequestScope;
use vernal_web_testkit::{
    FailingHttpBody, ScopeCleanupTimeoutFixture, ScopeCloseProbe, ScopeRejectingInterceptor,
    SecurityContractInterceptor, WebAdapterContract,
};

struct Greeting(&'static str);

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<Greeting, WebRequestScope, _>(
            |_| Greeting("vernal"),
        ))
        .expect("component registration");
    let registry = registry.build().expect("registry build");
    let context = Arc::new(
        ApplicationContextBuilder::new(registry)
            .build()
            .expect("context build"),
    );
    context.refresh().await.expect("context refresh");
    context.start().await.expect("context start");
    context
}

async fn ready_aop_context(
    operation: Operation,
    advisor: Option<Advisor>,
) -> Arc<vernal_context::ApplicationContext> {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder.operation(operation);
    if let Some(advisor) = advisor {
        builder.advisor(advisor);
    }
    let context = Arc::new(builder.build().expect("AOP context build"));
    context.refresh().await.expect("AOP context refresh");
    context.start().await.expect("AOP context start");
    context
}

#[tokio::test]
async fn router_extracts_context_component_and_request_scope() {
    let context = ready_context().await;
    let expected = Arc::clone(&context);
    let probe = Arc::new(ScopeCloseProbe::new());
    let handler_probe = Arc::clone(&probe);
    let app = Router::new()
        .route(
            "/hello",
            get(
                move |VernalContext(actual): VernalContext,
                      VernalComponent(greeting): VernalComponent<Greeting>,
                      VernalRequestScope(scope): VernalRequestScope| {
                    let expected = Arc::clone(&expected);
                    let handler_probe = Arc::clone(&handler_probe);
                    async move {
                        WebAdapterContract::assert_request_binding(
                            &expected, &actual, &scope, &greeting,
                        );
                        handler_probe.observe(&scope);
                        greeting.0
                    }
                },
            ),
        )
        .with_vernal(Arc::clone(&context));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/hello")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("router response");
    assert_eq!(response.status(), StatusCode::OK);
    probe.assert_open();
    assert_eq!(
        to_bytes(response.into_body(), 64)
            .await
            .expect("response body"),
        "vernal"
    );
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn partial_response_consumption_then_disconnect_closes_scope_and_records_failure() {
    let context = ready_context().await;
    let probe = Arc::new(ScopeCloseProbe::new());
    let handler_probe = Arc::clone(&probe);
    let app = Router::new()
        .route(
            "/drop",
            get(move |VernalRequestScope(scope): VernalRequestScope| {
                let handler_probe = Arc::clone(&handler_probe);
                async move {
                    handler_probe.observe(&scope);
                    scope
                        .on_close(|| async {
                            Err::<(), _>(io::Error::other(
                                "private cleanup source must not enter diagnostics",
                            ))
                        })
                        .expect("failing close hook");
                    "stream is not consumed"
                }
            }),
        )
        .with_vernal(Arc::clone(&context));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/drop")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("router response");
    let mut body = response.into_body();
    let data = body
        .frame()
        .await
        .expect("Axum response must emit one data frame")
        .expect("Axum data frame must succeed")
        .into_data()
        .expect("first Axum frame must contain data");
    assert_eq!(data, "stream is not consumed");
    probe.assert_open();
    drop(body);

    probe.assert_closed_within(Duration::from_secs(1)).await;
    assert_eq!(
        context.startup_report().await.warnings(),
        ["web.request-scope.cleanup-failed"]
    );
}

#[tokio::test]
async fn axum_body_error_closes_scope_before_restoring_upstream_error() {
    let context = ready_context().await;
    let probe = Arc::new(ScopeCloseProbe::new());
    let handler_probe = Arc::clone(&probe);
    let app = Router::new()
        .route(
            "/failure",
            get(move |VernalRequestScope(scope): VernalRequestScope| {
                let handler_probe = Arc::clone(&handler_probe);
                async move {
                    handler_probe.observe(&scope);
                    Response::new(Body::new(FailingHttpBody::new()))
                }
            }),
        )
        .with_vernal(context);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/failure")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("router response");
    probe.assert_open();
    let error = to_bytes(response.into_body(), 64)
        .await
        .expect_err("synthetic Axum body must fail");
    assert!(error.to_string().contains(FailingHttpBody::error_message()));
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn axum_body_reports_cleanup_timeout_while_background_close_continues() {
    let fixture = ScopeCleanupTimeoutFixture::new(Duration::from_millis(10)).await;
    let body = Body::new(vernal_tower::ScopedBody::new(
        FailingHttpBody::new(),
        fixture.scope(),
        fixture.scope().cancellation().clone(),
    ));

    let error = to_bytes(body, 64)
        .await
        .expect_err("Axum body must report the scope cleanup timeout");
    assert!(
        error.to_string().contains("cleanup exceeded timeout"),
        "unexpected Axum cleanup error: {error}"
    );
    fixture.assert_cleanup_timed_out();
    fixture.assert_redacted_warning().await;
    fixture.release();
    fixture.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn missing_vernal_layer_returns_safe_rejection() {
    let app = Router::new().route(
        "/missing",
        get(|_: VernalComponent<Greeting>| async { "unreachable" }),
    );
    let response = app
        .oneshot(
            Request::builder()
                .uri("/missing")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("router response");
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        to_bytes(response.into_body(), 128)
            .await
            .expect("response body"),
        "Vernal application context is unavailable"
    );
}

#[tokio::test]
async fn strict_aop_router_uses_matched_path_and_propagates_owned_snapshot() {
    let context = ready_aop_context(
        Operation::new("/orders/{id}", "GET"),
        Some(Advisor::new(
            |_: &Operation| true,
            SecurityContractInterceptor::authenticated("operator-7", ["operator"]),
            -1000,
        )),
    )
    .await;
    let app = Router::new()
        .route(
            "/orders/{id}",
            get(
                |VernalRequestContext(context): VernalRequestContext| async move {
                    let snapshot = context
                        .extensions()
                        .get::<HttpRequestSnapshot>()
                        .await
                        .expect("owned HTTP snapshot");
                    assert_eq!(context.route().handler(), "/orders/{id}");
                    assert_eq!(context.route().operation_name(), "GET");
                    assert_eq!(context.route().path_template(), "/orders/{id}");
                    assert_eq!(snapshot.uri().path(), "/orders/42");
                    let principal = context.principal().await.expect("security principal");
                    assert_eq!(principal.subject(), "operator-7");
                    assert!(principal.has_role("operator"));
                    "woven"
                },
            ),
        )
        .with_vernal_aop(context);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/orders/42")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("strict AOP response");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        to_bytes(response.into_body(), 64)
            .await
            .expect("response body"),
        "woven"
    );
}

#[tokio::test]
async fn strict_aop_router_maps_policy_failure_without_calling_handler() {
    let called = Arc::new(AtomicBool::new(false));
    let probe = Arc::new(ScopeCloseProbe::new());
    let context = ready_aop_context(
        Operation::new("/protected", "GET"),
        Some(Advisor::new(
            |_: &Operation| true,
            ScopeRejectingInterceptor::new(Arc::clone(&probe)),
            -1000,
        )),
    )
    .await;
    let handler_called = Arc::clone(&called);
    let app = Router::new()
        .route(
            "/protected",
            get(move || {
                handler_called.store(true, Ordering::SeqCst);
                async { "unreachable" }
            }),
        )
        .with_vernal_aop(context);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("policy response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        to_bytes(response.into_body(), 128)
            .await
            .expect("response body"),
        "Authentication is required"
    );
    assert!(!called.load(Ordering::SeqCst));
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn strict_aop_router_maps_authenticated_forbidden_to_403() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(
        Operation::new("/protected", "GET"),
        Some(Advisor::new(
            |_: &Operation| true,
            SecurityContractInterceptor::forbidden("operator-7", ["operator"]),
            -1000,
        )),
    )
    .await;
    let handler_called = Arc::clone(&called);
    let app = Router::new()
        .route(
            "/protected",
            get(move || {
                handler_called.store(true, Ordering::SeqCst);
                async { "unreachable" }
            }),
        )
        .with_vernal_aop(context);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("forbidden response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        to_bytes(response.into_body(), 128)
            .await
            .expect("response body"),
        "Access is forbidden"
    );
    assert!(!called.load(Ordering::SeqCst));
}
