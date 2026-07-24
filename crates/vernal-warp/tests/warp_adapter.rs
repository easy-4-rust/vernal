//! Warp Filter、Tower Service、IoC 与请求作用域集成测试。

#[path = "aop_support/policy_deny_interceptor.rs"]
mod policy_deny_interceptor;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use policy_deny_interceptor::PolicyDenyInterceptor;
use tokio::sync::Notify;
use tower::{Layer, ServiceExt};
use vernal_aop::{Advisor, Operation};
use vernal_context::{ApplicationContextBuilder, VernalApplicationBuilder};
use vernal_http::HttpRequestSnapshot;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_warp::{
    VernalWarpAopLayer, VernalWarpComponent, VernalWarpContext, VernalWarpLayer,
    VernalWarpRequestContext, VernalWarpRequestScope, WarpRejection,
};
use vernal_web::WebRequestScope;
use vernal_web_testkit::WebAdapterContract;
use warp::{Filter, Rejection, Reply, http::StatusCode};

struct Greeting(&'static str);

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<Greeting, WebRequestScope, _>(
            |_| Greeting("vernal-warp"),
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
async fn service_exposes_context_component_and_scope_until_body_finishes() {
    let context = ready_context().await;
    let expected = Arc::clone(&context);
    let closed = Arc::new(Notify::new());
    let handler_closed = Arc::clone(&closed);
    let route = VernalWarpContext::filter()
        .and(VernalWarpComponent::<Greeting>::filter())
        .and(VernalWarpRequestScope::filter())
        .and_then(
            move |actual: VernalWarpContext,
                  greeting: VernalWarpComponent<Greeting>,
                  scope: VernalWarpRequestScope| {
                let expected = Arc::clone(&expected);
                let handler_closed = Arc::clone(&handler_closed);
                async move {
                    let greeting = greeting.into_inner();
                    WebAdapterContract::assert_request_binding(
                        &expected, &actual.0, &scope.0, &greeting,
                    );
                    scope
                        .0
                        .on_close(move || async move {
                            handler_closed.notify_one();
                            Ok::<_, std::io::Error>(())
                        })
                        .expect("scope close hook");
                    Ok::<_, Rejection>(greeting.0)
                }
            },
        )
        .recover(WarpRejection::recover);
    let service = VernalWarpLayer::new(context).layer(warp::service(route));
    let request = warp::http::Request::builder()
        .uri("/hello")
        .body(Empty::<Bytes>::new())
        .expect("request");

    let response = service.oneshot(request).await.expect("service response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "vernal-warp");
    closed.notified().await;
}

#[tokio::test]
async fn missing_vernal_layer_returns_safe_rejection() {
    let route = VernalWarpComponent::<Greeting>::filter()
        .map(|_| "unreachable")
        .recover(WarpRejection::recover);
    let request = warp::http::Request::builder()
        .uri("/missing")
        .body(Empty::<Bytes>::new())
        .expect("request");

    let response = warp::service(route)
        .oneshot(request)
        .await
        .expect("service response");
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response
        .into_response()
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "Vernal application context is unavailable");
}

#[tokio::test]
async fn legacy_layer_does_not_fabricate_a_strict_request_context() {
    let context = ready_context().await;
    let route = VernalWarpRequestContext::filter()
        .map(|_| "unreachable")
        .recover(WarpRejection::recover);
    let service = VernalWarpLayer::new(context).layer(warp::service(route));
    let request = warp::http::Request::builder()
        .uri("/legacy")
        .body(Empty::<Bytes>::new())
        .expect("request");

    let response = service.oneshot(request).await.expect("legacy response");
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "Vernal request context is unavailable");
}

#[tokio::test]
async fn strict_aop_uses_declared_pattern_and_owned_snapshot() {
    let context = ready_aop_context(Operation::new("/orders/{id}", "GET"), None).await;
    let route = warp::path!("orders" / u64)
        .and(warp::get())
        .and(VernalWarpRequestContext::filter())
        .and_then(
            |_order_id: u64, request_context: VernalWarpRequestContext| async move {
                let snapshot = request_context
                    .0
                    .extensions()
                    .get::<HttpRequestSnapshot>()
                    .await
                    .expect("owned HTTP snapshot");

                assert_eq!(request_context.0.route().handler(), "/orders/{id}");
                assert_eq!(request_context.0.route().operation_name(), "GET");
                assert_eq!(request_context.0.route().path_template(), "/orders/{id}");
                assert_eq!(snapshot.uri().path(), "/orders/42");
                Ok::<_, Rejection>("woven")
            },
        );
    let service = VernalWarpAopLayer::new(context, "/orders/{id}").layer(warp::service(route));
    let request = warp::http::Request::builder()
        .uri("/orders/42")
        .body(Empty::<Bytes>::new())
        .expect("request");

    let response = service.oneshot(request).await.expect("strict AOP response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "woven");
}

#[tokio::test]
async fn strict_aop_maps_policy_failure_without_calling_filter() {
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
    let filter_called = Arc::clone(&called);
    let route = warp::path("protected").and(warp::get()).map(move || {
        filter_called.store(true, Ordering::SeqCst);
        "unreachable"
    });
    let service = VernalWarpAopLayer::new(context, "/protected").layer(warp::service(route));
    let request = warp::http::Request::builder()
        .uri("/protected")
        .body(Empty::<Bytes>::new())
        .expect("request");

    let response = service.oneshot(request).await.expect("policy response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "Authentication is required");
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn strict_aop_fails_closed_when_plan_is_missing() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(Operation::new("/different", "GET"), None).await;
    let filter_called = Arc::clone(&called);
    let route = warp::path("missing-plan").and(warp::get()).map(move || {
        filter_called.store(true, Ordering::SeqCst);
        "unreachable"
    });
    let service = VernalWarpAopLayer::new(context, "/missing-plan").layer(warp::service(route));
    let request = warp::http::Request::builder()
        .uri("/missing-plan")
        .body(Empty::<Bytes>::new())
        .expect("request");

    let response = service
        .oneshot(request)
        .await
        .expect("missing plan response");
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "Vernal AOP invocation failed");
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn strict_aop_fails_closed_when_pattern_is_empty() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(Operation::new("/empty", "GET"), None).await;
    let filter_called = Arc::clone(&called);
    let route = warp::path("empty").and(warp::get()).map(move || {
        filter_called.store(true, Ordering::SeqCst);
        "unreachable"
    });
    let service = VernalWarpAopLayer::new(context, "").layer(warp::service(route));
    let request = warp::http::Request::builder()
        .uri("/empty")
        .body(Empty::<Bytes>::new())
        .expect("request");

    let response = service
        .oneshot(request)
        .await
        .expect("missing route response");
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "Warp route pattern is unavailable");
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn strict_aop_preserves_native_warp_response() {
    let context = ready_aop_context(Operation::new("/native-error", "GET"), None).await;
    let route = warp::path("native-error").and(warp::get()).map(|| {
        warp::reply::with_status("native-not-found", StatusCode::NOT_FOUND).into_response()
    });
    let service = VernalWarpAopLayer::new(context, "/native-error").layer(warp::service(route));
    let request = warp::http::Request::builder()
        .uri("/native-error")
        .body(Empty::<Bytes>::new())
        .expect("request");

    let response = service.oneshot(request).await.expect("native response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "native-not-found");
}
