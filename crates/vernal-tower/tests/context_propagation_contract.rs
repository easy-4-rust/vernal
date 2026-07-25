//! Vernal Tower 请求元数据、上下文复用与取消传播合同测试。

use std::{convert::Infallible, future::pending, sync::Arc, time::Duration};

use http::{Method, Request, Response, Version};
use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;
use tower::{Layer, ServiceExt, service_fn};
use vernal_context::ApplicationContextBuilder;
use vernal_http::{HttpBody, HttpRequestSnapshot};
use vernal_beans::Registry;
use vernal_tower::{ContextPropagationError, ContextPropagationLayer, RequestScopeLayer};
use vernal_web::{RequestContext, RouteMetadata, WebRequestScope};

fn route(path: &'static str) -> RouteMetadata {
    RouteMetadata::new(path, "GET", path)
}

/// 创建供 Tower 请求作用域绑定的最小可用应用上下文。
async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let context = Arc::new(
        ApplicationContextBuilder::new(Registry::empty())
            .build()
            .expect("context build"),
    );
    context.refresh().await.expect("context refresh");
    context.start().await.expect("context start");
    context
}

#[tokio::test]
async fn propagation_builds_owned_context_snapshot_and_cancellation_extension() {
    let service = service_fn(|request: Request<()>| async move {
        let context = request
            .extensions()
            .get::<Arc<RequestContext>>()
            .cloned()
            .expect("propagated request context");
        let cancellation = request
            .extensions()
            .get::<CancellationToken>()
            .cloned()
            .expect("propagated cancellation token");
        let snapshot = context
            .extensions()
            .get::<HttpRequestSnapshot>()
            .await
            .expect("owned HTTP snapshot");

        assert_eq!(context.route().path_template(), "/orders/{id}");
        assert_eq!(snapshot.method(), Method::POST);
        assert_eq!(snapshot.uri().path(), "/orders/42");
        assert_eq!(snapshot.version(), Version::HTTP_2);
        assert_eq!(snapshot.headers()["x-request-id"], "request-42");
        assert!(!cancellation.is_cancelled());
        assert!(!context.cancellation().is_cancelled());
        Ok::<_, Infallible>(Response::new(HttpBody::full("propagated")))
    });
    let service = RequestScopeLayer::new(ready_context().await)
        .layer(ContextPropagationLayer::from_extension().layer(service));
    let mut request = Request::builder()
        .method(Method::POST)
        .uri("/orders/42?dry_run=true")
        .version(Version::HTTP_2)
        .header("x-request-id", "request-42")
        .body(())
        .expect("valid request");
    request.extensions_mut().insert(route("/orders/{id}"));

    let response = service.oneshot(request).await.expect("propagated response");
    assert_eq!(response.status(), http::StatusCode::OK);
}

#[tokio::test]
async fn propagation_preserves_adapter_context_and_more_precise_snapshot() {
    let cancellation = CancellationToken::new();
    let scope = Arc::new(WebRequestScope::new(cancellation.clone()));
    let context = Arc::new(RequestContext::new(
        route("/mounted/orders/{id}"),
        cancellation,
    ));
    let precise_request = Request::builder()
        .method(Method::PATCH)
        .uri("/external/orders/42")
        .header("x-original-uri", "true")
        .body(())
        .expect("precise adapter request");
    context
        .extensions()
        .insert(HttpRequestSnapshot::capture(&precise_request))
        .await;

    let expected = Arc::clone(&context);
    let service = service_fn(move |request: Request<()>| {
        let expected = Arc::clone(&expected);
        async move {
            let actual = request
                .extensions()
                .get::<Arc<RequestContext>>()
                .cloned()
                .expect("existing request context");
            let snapshot = actual
                .extensions()
                .get::<HttpRequestSnapshot>()
                .await
                .expect("existing snapshot");
            assert!(Arc::ptr_eq(&actual, &expected));
            assert_eq!(actual.route().path_template(), "/mounted/orders/{id}");
            assert_eq!(snapshot.method(), Method::PATCH);
            assert_eq!(snapshot.uri().path(), "/external/orders/42");
            assert_eq!(snapshot.headers()["x-original-uri"], "true");
            Ok::<_, Infallible>(Response::new(()))
        }
    });
    let mut request = Request::builder()
        .method(Method::DELETE)
        .uri("/internal/orders/42")
        .body(())
        .expect("generic tower request");
    request.extensions_mut().insert(scope);
    request.extensions_mut().insert(Arc::clone(&context));
    request.extensions_mut().insert(route("/less-precise"));

    ContextPropagationLayer::from_extension()
        .layer(service)
        .oneshot(request)
        .await
        .expect("existing context must be preserved");
}

#[tokio::test]
async fn propagation_reports_missing_scope_and_route_before_calling_handler() {
    let service =
        || service_fn(|_request: Request<()>| async { Ok::<_, Infallible>(Response::new(())) });
    let mut missing_scope = Request::new(());
    missing_scope.extensions_mut().insert(route("/orders"));
    let error = ContextPropagationLayer::from_extension()
        .layer(service())
        .oneshot(missing_scope)
        .await
        .expect_err("scope is required");
    assert!(matches!(
        error,
        ContextPropagationError::MissingRequestScope
    ));

    let mut missing_route = Request::new(());
    missing_route
        .extensions_mut()
        .insert(Arc::new(WebRequestScope::new(CancellationToken::new())));
    let error = ContextPropagationLayer::from_extension()
        .layer(service())
        .oneshot(missing_route)
        .await
        .expect_err("route identity is required");
    assert!(matches!(
        error,
        ContextPropagationError::MissingRouteMetadata
    ));
}

#[tokio::test]
async fn dropping_request_future_cancels_the_propagated_context() {
    let started = Arc::new(Notify::new());
    let captured = Arc::new(Mutex::new(None));
    let service_started = Arc::clone(&started);
    let service_captured = Arc::clone(&captured);
    let service = service_fn(move |request: Request<()>| {
        let service_started = Arc::clone(&service_started);
        let service_captured = Arc::clone(&service_captured);
        async move {
            let context = request
                .extensions()
                .get::<Arc<RequestContext>>()
                .cloned()
                .expect("propagated request context");
            *service_captured.lock().await = Some(context);
            service_started.notify_one();
            pending::<Result<Response<HttpBody>, Infallible>>().await
        }
    });
    let service = RequestScopeLayer::new(ready_context().await)
        .layer(ContextPropagationLayer::from_extension().layer(service));
    let mut request = Request::new(());
    request.extensions_mut().insert(route("/pending"));

    let task = tokio::spawn(service.oneshot(request));
    started.notified().await;
    task.abort();
    let _ = task.await;

    let context = captured
        .lock()
        .await
        .clone()
        .expect("captured request context");
    tokio::time::timeout(Duration::from_secs(1), context.cancellation().cancelled())
        .await
        .expect("request cancellation must reach propagated context");
}
