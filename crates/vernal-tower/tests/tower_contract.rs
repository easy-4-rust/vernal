//! Vernal Tower Context 注入、Scope Body 生命周期和请求取消合同测试。

use std::{convert::Infallible, future::pending, io, sync::Arc, time::Duration};

use http::{Request, Response};
use http_body_util::BodyExt;
use tokio::sync::{Mutex, Notify};
use tower::{Layer, ServiceExt, service_fn};
use vernal_context::ApplicationContextBuilder;
use vernal_http::HttpBody;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_tower::{RequestScopeLayer, TowerBodyError, TowerError, VernalLayer};
use vernal_web::{ScopeState, WebRequestScope};
use vernal_web_testkit::{FailingHttpBody, ScopeCleanupTimeoutFixture, ScopeCloseProbe};

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

async fn ready_context_with_scoped_value() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<String, WebRequestScope, _>(
            |_| "request-value".to_owned(),
        ))
        .expect("scoped value registration");
    let registry = registry.build().expect("registry");
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
            .collect()
            .await
            .expect("body")
            .to_bytes(),
        &vernal_http::Bytes::from_static(b"ok")
    );
}

#[tokio::test]
async fn request_scope_layer_uses_one_context_for_extensions_and_scoped_components() {
    let context = ready_context_with_scoped_value().await;
    let expected = Arc::clone(&context);
    let service = service_fn(move |request: Request<()>| {
        let expected = Arc::clone(&expected);
        async move {
            let injected = request
                .extensions()
                .get::<Arc<vernal_context::ApplicationContext>>()
                .expect("context extension");
            let scope = request
                .extensions()
                .get::<Arc<WebRequestScope>>()
                .expect("request scope");
            let value = scope.resolve::<String>().expect("request component");

            // Layer 单独使用时也必须写入与 Scope Owner 完全相同的 Arc；
            // 这保证组件提取器不会因 Tower Layer 顺序而访问另一棵组件图。
            assert!(Arc::ptr_eq(injected, &expected));
            assert!(Arc::ptr_eq(
                scope
                    .application_context()
                    .expect("scope application owner"),
                &expected
            ));
            assert_eq!(value.as_str(), "request-value");
            Ok::<_, Infallible>(Response::new(HttpBody::full("ok")))
        }
    });

    let response = RequestScopeLayer::new(context)
        .layer(service)
        .oneshot(Request::new(()))
        .await
        .expect("scoped response");
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes(),
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
                .expect("close hook");
            Ok::<_, Infallible>(Response::new(HttpBody::full("stream")))
        }
    });

    let response = RequestScopeLayer::new(ready_context().await)
        .layer(service)
        .oneshot(Request::new(()))
        .await
        .expect("scoped response");
    let scope = captured_scope.lock().await.clone().expect("captured scope");
    assert_eq!(scope.state(), ScopeState::Open);
    let collected = response
        .into_body()
        .collect()
        .await
        .expect("scoped body should complete");
    assert_eq!(collected.to_bytes(), "stream");
    assert_eq!(scope.state(), ScopeState::Closed);
    assert_eq!(*events.lock().await, ["closed"]);
}

#[tokio::test]
async fn partial_response_consumption_then_disconnect_closes_request_scope() {
    let probe = Arc::new(ScopeCloseProbe::new());
    let service_probe = Arc::clone(&probe);
    let service = service_fn(move |request: Request<()>| {
        let service_probe = Arc::clone(&service_probe);
        async move {
            let scope = request
                .extensions()
                .get::<Arc<WebRequestScope>>()
                .expect("request scope")
                .clone();
            service_probe.observe(&scope);
            Ok::<_, Infallible>(Response::new(HttpBody::full("stream is not consumed")))
        }
    });

    let response = RequestScopeLayer::new(ready_context().await)
        .layer(service)
        .oneshot(Request::new(()))
        .await
        .expect("scoped response");
    let mut body = response.into_body();
    let data = body
        .frame()
        .await
        .expect("Tower response must emit one data frame")
        .expect("Tower data frame must succeed")
        .into_data()
        .expect("first Tower frame must contain data");
    assert_eq!(data, "stream is not consumed");
    probe.assert_open();
    drop(body);

    probe.assert_closed_within(Duration::from_secs(1)).await;
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
            .expect("close hook");
        Ok::<_, Infallible>(Response::new(HttpBody::full("body")))
    });

    let response = RequestScopeLayer::new(ready_context().await)
        .layer(service)
        .oneshot(Request::new(()))
        .await
        .expect("scoped response");
    let result = response.into_body().collect().await;
    assert!(matches!(result, Err(TowerBodyError::Scope(_))));
}

#[tokio::test]
async fn upstream_body_error_closes_scope_before_restoring_transport_failure() {
    let probe = Arc::new(ScopeCloseProbe::new());
    let service_probe = Arc::clone(&probe);
    let service = service_fn(move |request: Request<()>| {
        let service_probe = Arc::clone(&service_probe);
        async move {
            let scope = request
                .extensions()
                .get::<Arc<WebRequestScope>>()
                .expect("request scope")
                .clone();
            service_probe.observe(&scope);
            Ok::<_, Infallible>(Response::new(FailingHttpBody::new()))
        }
    });

    let response = RequestScopeLayer::new(ready_context().await)
        .layer(service)
        .oneshot(Request::new(()))
        .await
        .expect("scoped response");
    probe.assert_open();
    let error = response
        .into_body()
        .collect()
        .await
        .expect_err("upstream body failure must remain observable");
    assert!(matches!(
        error,
        TowerBodyError::Upstream(ref source)
            if source.to_string() == FailingHttpBody::error_message()
    ));
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn response_body_reports_cleanup_timeout_without_cancelling_background_close() {
    let fixture = ScopeCleanupTimeoutFixture::new(Duration::from_millis(10)).await;
    let scoped = vernal_tower::ScopedBody::new(
        FailingHttpBody::new(),
        fixture.scope(),
        fixture.scope().cancellation().clone(),
    );

    let error = scoped
        .collect()
        .await
        .expect_err("Tower body must report the scope cleanup timeout");
    assert!(
        error.to_string().contains("cleanup exceeded timeout"),
        "unexpected Tower cleanup error: {error}"
    );
    fixture.assert_cleanup_timed_out();
    fixture.assert_redacted_warning().await;
    fixture.release();
    fixture.assert_closed_within(Duration::from_secs(1)).await;
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

    let result = RequestScopeLayer::new(ready_context().await)
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
            .state(),
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
                .expect("close hook");
            service_started.notify_one();
            pending::<Result<Response<HttpBody>, Infallible>>().await
        }
    });

    let task = tokio::spawn(
        RequestScopeLayer::new(ready_context().await)
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
