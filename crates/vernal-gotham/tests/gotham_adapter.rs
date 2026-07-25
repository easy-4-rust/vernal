//! Gotham Middleware、State、IoC 与响应 Body 生命周期测试。

#[path = "aop_support/native_response_interceptor.rs"]
mod native_response_interceptor;

use std::{
    io,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bytes::Bytes;
use futures_util::stream;
use gotham::{
    handler::{HandlerError, HandlerResult, IntoBody},
    helpers::http::Body,
    middleware::Middleware,
    state::State,
};
use http::{HeaderMap, Request, Response, StatusCode};
use http_body::Frame;
use http_body_util::{BodyExt, Empty, StreamBody};
use native_response_interceptor::NativeResponseInterceptor;
use tokio_util::sync::CancellationToken;
use vernal_aop::{Advisor, Operation};
use vernal_context::{ApplicationContextBuilder, VernalApplicationBuilder};
use vernal_gotham::{GothamScopedBody, VernalGothamMiddleware, VernalGothamStateExt};
use vernal_http::HttpRequestSnapshot;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_web::WebRequestScope;
use vernal_web_testkit::{
    FailingHttpBody, ScopeCleanupTimeoutFixture, ScopeCloseProbe, ScopeRejectingInterceptor,
    WebAdapterContract,
};

struct Greeting(&'static str);

fn expect_response(result: HandlerResult) -> (State, Response<Body>) {
    match result {
        Ok(response) => response,
        Err((_state, error)) => panic!("unexpected Gotham handler error: {error:?}"),
    }
}

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<Greeting, WebRequestScope, _>(
            |_| Greeting("vernal-gotham"),
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

fn request_state(path: &str) -> State {
    let request = Request::builder()
        .uri(path)
        .body(Empty::<Bytes>::new())
        .expect("request build");
    State::from_request(request, SocketAddr::from(([127, 0, 0, 1], 3000)))
}

#[tokio::test]
async fn middleware_exposes_context_component_and_scope_until_body_finishes() {
    let context = ready_context().await;
    let expected = Arc::clone(&context);
    let probe = Arc::new(ScopeCloseProbe::new());
    let handler_probe = Arc::clone(&probe);
    let middleware = VernalGothamMiddleware::new(context);

    let result = middleware
        .call(request_state("/hello"), move |state| {
            Box::pin(async move {
                let actual = state.vernal_context().expect("Vernal context");
                let greeting = state
                    .vernal_component::<Greeting>()
                    .expect("Vernal component");
                let scope = state.vernal_request_scope().expect("request scope");
                WebAdapterContract::assert_request_binding(&expected, &actual, &scope, &greeting);
                handler_probe.observe(&scope);
                Ok((state, Response::new(greeting.0.into_body())))
            })
        })
        .await;
    let (_state, response) = expect_response(result);

    assert_eq!(response.status(), StatusCode::OK);
    probe.assert_open();
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .expect("response body")
            .to_bytes(),
        "vernal-gotham"
    );
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn missing_middleware_returns_safe_internal_server_error() {
    let state = request_state("/missing");
    let Err(rejection) = state.vernal_component::<Greeting>() else {
        panic!("missing context must reject");
    };
    let response = rejection.response(&state);

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .expect("rejection body")
            .to_bytes(),
        "Vernal application context is unavailable"
    );
}

#[tokio::test]
async fn partial_response_consumption_then_disconnect_closes_request_scope() {
    let context = ready_context().await;
    let probe = Arc::new(ScopeCloseProbe::new());
    let handler_probe = Arc::clone(&probe);
    let middleware = VernalGothamMiddleware::new(context);

    let result = middleware
        .call(request_state("/drop"), move |state| {
            Box::pin(async move {
                let scope = state.vernal_request_scope().expect("request scope");
                handler_probe.observe(&scope);
                Ok((state, Response::new("stream is not consumed".into_body())))
            })
        })
        .await;
    let (_state, response) = expect_response(result);
    let mut body = response.into_body();
    let data = body
        .frame()
        .await
        .expect("Gotham response must emit one data frame")
        .expect("Gotham data frame must succeed")
        .into_data()
        .expect("first Gotham frame must contain data");
    assert_eq!(data, "stream is not consumed");
    probe.assert_open();
    drop(body);

    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn gotham_body_preserves_data_trailers_and_backpressure() {
    let context = ready_context().await;
    let middleware = VernalGothamMiddleware::new(context);
    let mut trailers = HeaderMap::new();
    trailers.insert("x-vernal-trailer", "kept".parse().expect("header value"));

    let result = middleware
        .call(request_state("/frames"), move |state| {
            Box::pin(async move {
                let frames = stream::iter(vec![
                    Ok::<_, io::Error>(Frame::data(Bytes::from_static(b"data"))),
                    Ok(Frame::trailers(trailers)),
                ]);
                let body = StreamBody::new(frames).boxed_unsync();
                Ok((state, Response::new(body)))
            })
        })
        .await;
    let (_state, response) = expect_response(result);
    let mut body = response.into_body();

    let data = body
        .frame()
        .await
        .expect("data frame")
        .expect("data result")
        .into_data()
        .expect("data payload");
    assert_eq!(data, "data");
    let trailers = body
        .frame()
        .await
        .expect("trailer frame")
        .expect("trailer result")
        .into_trailers()
        .expect("trailer payload");
    assert_eq!(trailers["x-vernal-trailer"], "kept");
    assert!(body.frame().await.is_none());
}

#[tokio::test]
async fn gotham_body_error_closes_scope_before_restoring_upstream_error() {
    let scope = Arc::new(WebRequestScope::new(CancellationToken::new()));
    let scoped = GothamScopedBody::new(
        FailingHttpBody::new(),
        Arc::clone(&scope),
        CancellationToken::new(),
    );
    WebAdapterContract::assert_scope_open(&scope);

    let error = scoped
        .collect()
        .await
        .expect_err("synthetic Gotham body must fail");
    assert_eq!(error.to_string(), FailingHttpBody::error_message());
    WebAdapterContract::assert_scope_closed(&scope);
}

#[tokio::test]
async fn gotham_body_reports_cleanup_timeout_while_background_close_continues() {
    let fixture = ScopeCleanupTimeoutFixture::new(Duration::from_millis(10)).await;
    let scoped = GothamScopedBody::new(
        FailingHttpBody::new(),
        fixture.scope(),
        fixture.scope().cancellation().clone(),
    );

    let error = scoped
        .collect()
        .await
        .expect_err("Gotham body must report the scope cleanup timeout");
    assert!(
        error.to_string().contains("cleanup exceeded timeout"),
        "unexpected Gotham cleanup error: {error}"
    );
    fixture.assert_cleanup_timed_out();
    fixture.assert_redacted_warning().await;
    fixture.release();
    fixture.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn strict_aop_uses_declared_pattern_and_owned_snapshot() {
    let context = ready_aop_context(Operation::new("/orders/:id", "GET"), None).await;
    let middleware = VernalGothamMiddleware::strict_aop(context, "/orders/:id");

    let result = middleware
        .call(request_state("/orders/42"), |state| {
            Box::pin(async move {
                let request_context = state
                    .vernal_request_context()
                    .expect("strict request context");
                let snapshot = request_context
                    .extensions()
                    .get::<HttpRequestSnapshot>()
                    .await
                    .expect("owned HTTP snapshot");

                assert_eq!(request_context.route().handler(), "/orders/:id");
                assert_eq!(request_context.route().operation_name(), "GET");
                assert_eq!(request_context.route().path_template(), "/orders/:id");
                assert_eq!(snapshot.uri().path(), "/orders/42");
                Ok((state, Response::new("woven".into_body())))
            })
        })
        .await;
    let (_state, response) = expect_response(result);

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .expect("response body")
            .to_bytes(),
        "woven"
    );
}

#[tokio::test]
async fn strict_aop_maps_policy_failure_without_calling_handler() {
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
    let middleware = VernalGothamMiddleware::strict_aop(context, "/protected");

    let result = middleware
        .call(request_state("/protected"), move |state| {
            handler_called.store(true, Ordering::SeqCst);
            Box::pin(async move { Ok((state, Response::new("unreachable".into_body()))) })
        })
        .await;
    let (_state, response) = expect_response(result);

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .expect("response body")
            .to_bytes(),
        "Authentication is required"
    );
    assert!(!called.load(Ordering::SeqCst));
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn strict_aop_fails_closed_when_plan_is_missing() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(Operation::new("/different", "GET"), None).await;
    let handler_called = Arc::clone(&called);
    let middleware = VernalGothamMiddleware::strict_aop(context, "/missing-plan");

    let result = middleware
        .call(request_state("/missing-plan"), move |state| {
            handler_called.store(true, Ordering::SeqCst);
            Box::pin(async move { Ok((state, Response::new("unreachable".into_body()))) })
        })
        .await;
    let (_state, response) = expect_response(result);

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .expect("response body")
            .to_bytes(),
        "Vernal AOP invocation failed"
    );
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn strict_aop_fails_closed_when_pattern_is_empty() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(Operation::new("/empty", "GET"), None).await;
    let handler_called = Arc::clone(&called);
    let middleware = VernalGothamMiddleware::strict_aop(context, "");

    let result = middleware
        .call(request_state("/empty"), move |state| {
            handler_called.store(true, Ordering::SeqCst);
            Box::pin(async move { Ok((state, Response::new("unreachable".into_body()))) })
        })
        .await;
    let (_state, response) = expect_response(result);

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .expect("response body")
            .to_bytes(),
        "Gotham route pattern is unavailable"
    );
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn strict_aop_preserves_native_gotham_handler_error() {
    let context = ready_aop_context(Operation::new("/native-error", "GET"), None).await;
    let middleware = VernalGothamMiddleware::strict_aop(context, "/native-error");

    let result = middleware
        .call(request_state("/native-error"), |state| {
            Box::pin(async move {
                Err((
                    state,
                    HandlerError::from(io::Error::other("native failure"))
                        .with_status(StatusCode::IM_A_TEAPOT),
                ))
            })
        })
        .await;
    let Err((_state, error)) = result else {
        panic!("native Gotham error must remain an error");
    };

    assert_eq!(error.status(), StatusCode::IM_A_TEAPOT);
    assert_eq!(
        error
            .cause()
            .downcast_ref::<io::Error>()
            .expect("native source")
            .to_string(),
        "native failure"
    );
}

#[tokio::test]
async fn strict_aop_interceptor_can_return_native_response_without_handler() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(
        Operation::new("/native-short", "GET"),
        Some(Advisor::new(
            |_: &Operation| true,
            NativeResponseInterceptor,
            -1000,
        )),
    )
    .await;
    let handler_called = Arc::clone(&called);
    let middleware = VernalGothamMiddleware::strict_aop(context, "/native-short");

    let result = middleware
        .call(request_state("/native-short"), move |state| {
            handler_called.store(true, Ordering::SeqCst);
            Box::pin(async move { Ok((state, Response::new("unreachable".into_body()))) })
        })
        .await;
    let (_state, response) = expect_response(result);

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .expect("response body")
            .to_bytes(),
        "native-short-circuit"
    );
    assert!(!called.load(Ordering::SeqCst));
}
