//! Warp Filter、Tower Service、IoC 与请求作用域集成测试。

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use tower::{Layer, ServiceExt, service_fn};
use vernal_aop::{Advisor, Operation};
use vernal_context::{ApplicationContextBuilder, VernalApplicationBuilder};
use vernal_http::HttpRequestSnapshot;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_warp::{
    VernalWarpAopLayer, VernalWarpComponent, VernalWarpContext, VernalWarpLayer,
    VernalWarpRequestContext, VernalWarpRequestScope, WarpRejection,
};
use vernal_web::WebRequestScope;
use vernal_web_testkit::{
    FailingHttpBody, ScopeCleanupTimeoutFixture, ScopeCloseProbe, ScopeRejectingInterceptor,
    WebAdapterContract,
};
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
    let probe = Arc::new(ScopeCloseProbe::new());
    let handler_probe = Arc::clone(&probe);
    let route = VernalWarpContext::filter()
        .and(VernalWarpComponent::<Greeting>::filter())
        .and(VernalWarpRequestScope::filter())
        .and_then(
            move |actual: VernalWarpContext,
                  greeting: VernalWarpComponent<Greeting>,
                  scope: VernalWarpRequestScope| {
                let expected = Arc::clone(&expected);
                let handler_probe = Arc::clone(&handler_probe);
                async move {
                    let greeting = greeting.into_inner();
                    WebAdapterContract::assert_request_binding(
                        &expected, &actual.0, &scope.0, &greeting,
                    );
                    handler_probe.observe(&scope.0);
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
    probe.assert_open();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "vernal-warp");
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn partial_response_consumption_then_disconnect_closes_request_scope() {
    let context = ready_context().await;
    let probe = Arc::new(ScopeCloseProbe::new());
    let handler_probe = Arc::clone(&probe);
    let route = VernalWarpRequestScope::filter().and_then(move |scope: VernalWarpRequestScope| {
        let handler_probe = Arc::clone(&handler_probe);
        async move {
            handler_probe.observe(&scope.0);
            Ok::<_, Rejection>("stream is not consumed")
        }
    });
    let service = VernalWarpLayer::new(context).layer(warp::service(route));
    let request = warp::http::Request::builder()
        .uri("/drop")
        .body(Empty::<Bytes>::new())
        .expect("request");

    let response = service.oneshot(request).await.expect("service response");
    let mut body = response.into_body();
    let data = body
        .frame()
        .await
        .expect("Warp response must emit one data frame")
        .expect("Warp data frame must succeed")
        .into_data()
        .expect("first Warp frame must contain data");
    assert_eq!(data, "stream is not consumed");
    probe.assert_open();
    drop(body);

    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn warp_layer_body_error_closes_scope_before_restoring_upstream_error() {
    let context = ready_context().await;
    let probe = Arc::new(ScopeCloseProbe::new());
    let service_probe = Arc::clone(&probe);
    let inner = service_fn(move |request: warp::http::Request<Empty<Bytes>>| {
        let service_probe = Arc::clone(&service_probe);
        async move {
            let scope = request
                .extensions()
                .get::<Arc<WebRequestScope>>()
                .expect("request scope")
                .clone();
            service_probe.observe(&scope);
            Ok::<_, std::convert::Infallible>(warp::http::Response::new(FailingHttpBody::new()))
        }
    });
    let service = VernalWarpLayer::new(context).layer(inner);
    let request = warp::http::Request::builder()
        .uri("/failure")
        .body(Empty::<Bytes>::new())
        .expect("request");

    let response = service.oneshot(request).await.expect("service response");
    probe.assert_open();
    let error = response
        .into_body()
        .collect()
        .await
        .expect_err("synthetic Warp body must fail");
    assert!(matches!(
        error,
        vernal_tower::TowerBodyError::Upstream(ref source)
            if source.to_string() == FailingHttpBody::error_message()
    ));
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn warp_body_reports_cleanup_timeout_while_background_close_continues() {
    let fixture = ScopeCleanupTimeoutFixture::new(Duration::from_millis(10)).await;
    let scoped = vernal_tower::ScopedBody::new(
        FailingHttpBody::new(),
        fixture.scope(),
        fixture.scope().cancellation().clone(),
    );

    let error = scoped
        .collect()
        .await
        .expect_err("Warp body must report the scope cleanup timeout");
    assert!(
        error.to_string().contains("cleanup exceeded timeout"),
        "unexpected Warp cleanup error: {error}"
    );
    fixture.assert_cleanup_timed_out();
    fixture.assert_redacted_warning().await;
    fixture.release();
    fixture.assert_closed_within(Duration::from_secs(1)).await;
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
    probe.assert_closed_within(Duration::from_secs(1)).await;
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
