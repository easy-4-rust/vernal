//! Axum Router、Extractor、IoC 组件和请求作用域集成测试。

use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    routing::get,
};
use tokio::sync::Notify;
use tower::ServiceExt;
use vernal_axum::{VernalComponent, VernalContext, VernalRequestScope, VernalRouterExt};
use vernal_context::ApplicationContextBuilder;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};

struct Greeting(&'static str);

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::singleton::<Greeting, _>(|_| {
            Greeting("vernal")
        }))
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
                        assert!(Arc::ptr_eq(&actual, &expected));
                        assert!(!scope.cancellation().is_cancelled());
                        scope
                            .on_close(move || async move {
                                handler_closed.notify_one();
                                Ok::<_, std::io::Error>(())
                            })
                            .await
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
