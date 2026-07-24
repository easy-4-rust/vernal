//! Axum Router、Extractor、IoC 组件和请求作用域集成测试。

#[path = "aop_support/policy_deny_interceptor.rs"]
mod policy_deny_interceptor;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    routing::get,
};
use policy_deny_interceptor::PolicyDenyInterceptor;
use tokio::sync::Notify;
use tower::ServiceExt;
use vernal_aop::{Advisor, Operation};
use vernal_axum::{
    VernalComponent, VernalContext, VernalRequestContext, VernalRequestScope, VernalRouterExt,
};
use vernal_context::{ApplicationContextBuilder, VernalApplicationBuilder};
use vernal_http::HttpRequestSnapshot;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_web::WebRequestScope;
use vernal_web_testkit::WebAdapterContract;

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
    let closed = Arc::new(Notify::new());
    let handler_closed = Arc::clone(&closed);
    let app = Router::new()
        .route(
            "/hello",
            get(
                move |VernalContext(actual): VernalContext,
                      VernalComponent(greeting): VernalComponent<Greeting>,
                      VernalRequestScope(scope): VernalRequestScope| {
                    let expected = Arc::clone(&expected);
                    let handler_closed = Arc::clone(&handler_closed);
                    async move {
                        WebAdapterContract::assert_request_binding(
                            &expected, &actual, &scope, &greeting,
                        );
                        scope
                            .on_close(move || async move {
                                handler_closed.notify_one();
                                Ok::<_, std::io::Error>(())
                            })
                            .expect("scope close hook");
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
    assert_eq!(
        to_bytes(response.into_body(), 64)
            .await
            .expect("response body"),
        "vernal"
    );
    closed.notified().await;
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
    let context = ready_aop_context(Operation::new("/orders/{id}", "GET"), None).await;
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
    let context = ready_aop_context(
        Operation::new("/protected", "GET"),
        Some(Advisor::new(
            |_: &Operation| true,
            PolicyDenyInterceptor,
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
}
