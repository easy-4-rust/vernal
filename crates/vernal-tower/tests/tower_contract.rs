//! Vernal Tower Context 注入、Scope Body 生命周期和请求取消合同测试。

use std::{convert::Infallible, future::pending, io, sync::Arc, time::Duration};

use http::{Request, Response};
use http_body_util::BodyExt;
use tokio::sync::{Mutex, Notify};
use tower::{Layer, ServiceExt, service_fn};
use vernal_context::ApplicationContextBuilder;
use vernal_http::HttpBody;
use vernal_ioc::RegistryBuilder;
use vernal_tower::{RequestScopeLayer, TowerBodyError, TowerError, VernalLayer};
use vernal_web::{ScopeState, WebRequestScope};

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let registry = RegistryBuilder::new().build().expect("empty registry");
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
async fn vernal_layer_injects_explicit_application_context() {
    let context = ready_context().await;
    let expected = Arc::clone(&context);
    let service =
        VernalLayer::new(Arc::clone(&context)).layer(service_fn(move |request: Request<()>| {
            let expected = Arc::clone(&expected);
            async move {
                let injected = request
                    .extensions()
                    .get::<Arc<vernal_context::ApplicationContext>>()
                    .expect("context extension");
                assert!(Arc::ptr_eq(injected, &expected));
                Ok::<_, Infallible>(Response::new(HttpBody::full("ok")))
            }
        }));

    let response = service
        .oneshot(Request::new(()))
        .await
        .expect("service response");
    assert_eq!(
        response
            .into_body()
            .collect_limited(16)
            .await
            .expect("body")
            .bytes(),
        &vernal_http::Bytes::from_static(b"ok")
    );
}

#[tokio::test]
async fn response_body_completion_closes_scope_after_last_frame() {
    let captured_scope = Arc::new(Mutex::new(None));
    let events = Arc::new(Mutex::new(Vec::new()));
    let service_scope = Arc::clone(&captured_scope);
    let service_events = Arc::clone(&events);
    let service = service_fn(move |request: Request<()>| {
        let service_scope = Arc::clone(&service_scope);
        let service_events = Arc::clone(&service_events);
        async move {
            let scope = request
                .extensions()
                .get::<Arc<WebRequestScope>>()
                .expect("request scope")
                .clone();
            *service_scope.lock().await = Some(Arc::clone(&scope));
            scope
                .on_close(move || async move {
                    service_events.lock().await.push("closed");
                    Ok::<_, io::Error>(())
                })
                .await
                .expect("close hook");
            Ok::<_, Infallible>(Response::new(HttpBody::full("stream")))
        }
    });

    let response = RequestScopeLayer::new()
        .layer(service)
        .oneshot(Request::new(()))
        .await
        .expect("scoped response");
    let scope = captured_scope.lock().await.clone().expect("captured scope");
    assert_eq!(scope.state().await, ScopeState::Open);
    let collected = response
        .into_body()
        .collect()
        .await
        .expect("scoped body should complete");
    assert_eq!(collected.to_bytes(), "stream");
    assert_eq!(scope.state().await, ScopeState::Closed);
    assert_eq!(*events.lock().await, ["closed"]);
}

#[tokio::test]
async fn response_body_reports_explicit_scope_close_failure() {
    let service = service_fn(|request: Request<()>| async move {
        let scope = request
            .extensions()
            .get::<Arc<WebRequestScope>>()
            .expect("request scope")
            .clone();
        scope
            .on_close(|| async { Err::<(), _>(io::Error::other("release failed")) })
            .await
            .expect("close hook");
        Ok::<_, Infallible>(Response::new(HttpBody::full("body")))
    });

    let response = RequestScopeLayer::new()
        .layer(service)
        .oneshot(Request::new(()))
        .await
        .expect("scoped response");
    let result = response.into_body().collect().await;
    assert!(matches!(result, Err(TowerBodyError::Scope(_))));
}

#[tokio::test]
async fn upstream_service_error_still_closes_scope() {
    let captured_scope = Arc::new(Mutex::new(None));
    let service_scope = Arc::clone(&captured_scope);
    let service = service_fn(move |request: Request<()>| {
        let service_scope = Arc::clone(&service_scope);
        async move {
            let scope = request
                .extensions()
                .get::<Arc<WebRequestScope>>()
                .expect("request scope")
                .clone();
            *service_scope.lock().await = Some(scope);
            Err::<Response<HttpBody>, _>(io::Error::other("handler failed"))
        }
    });

    let result = RequestScopeLayer::new()
        .layer(service)
        .oneshot(Request::new(()))
        .await;
    let Err(error) = result else {
        panic!("upstream call should fail");
    };
    assert!(matches!(error, TowerError::Upstream(_)));
    assert_eq!(
        captured_scope
            .lock()
            .await
            .as_ref()
            .expect("captured scope")
            .state()
            .await,
        ScopeState::Closed
    );
}

#[tokio::test]
async fn dropping_request_future_signals_background_scope_cleanup() {
    let started = Arc::new(Notify::new());
    let closed = Arc::new(Notify::new());
    let service_started = Arc::clone(&started);
    let service_closed = Arc::clone(&closed);
    let service = service_fn(move |request: Request<()>| {
        let service_started = Arc::clone(&service_started);
        let service_closed = Arc::clone(&service_closed);
        async move {
            let scope = request
                .extensions()
                .get::<Arc<WebRequestScope>>()
                .expect("request scope")
                .clone();
            scope
                .on_close(move || async move {
                    service_closed.notify_one();
                    Ok::<_, io::Error>(())
                })
                .await
                .expect("close hook");
            service_started.notify_one();
            pending::<Result<Response<HttpBody>, Infallible>>().await
        }
    });

    let task = tokio::spawn(
        RequestScopeLayer::new()
            .layer(service)
            .oneshot(Request::new(())),
    );
    started.notified().await;
    task.abort();
    let _ = task.await;
    tokio::time::timeout(Duration::from_secs(1), closed.notified())
        .await
        .expect("scope cleanup should run after cancellation");
}
