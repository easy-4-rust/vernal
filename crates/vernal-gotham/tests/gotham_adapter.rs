//! Gotham Middleware、State、IoC 与响应 Body 生命周期测试。

use std::{io, net::SocketAddr, sync::Arc, time::Duration};

use bytes::Bytes;
use futures_util::stream;
use gotham::{
    handler::{HandlerResult, IntoBody},
    helpers::http::Body,
    middleware::Middleware,
    state::State,
};
use http::{HeaderMap, Request, Response, StatusCode};
use http_body::Frame;
use http_body_util::{BodyExt, Empty, StreamBody};
use tokio::sync::Notify;
use vernal_context::ApplicationContextBuilder;
use vernal_gotham::{VernalGothamMiddleware, VernalGothamStateExt};
use vernal_ioc::{ComponentDefinition, RegistryBuilder};

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
        .register(ComponentDefinition::singleton::<Greeting, _>(|_| {
            Greeting("vernal-gotham")
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
    let closed = Arc::new(Notify::new());
    let handler_closed = Arc::clone(&closed);
    let middleware = VernalGothamMiddleware::new(context);

    let result = middleware
        .call(request_state("/hello"), move |state| {
            Box::pin(async move {
                let actual = state.vernal_context().expect("Vernal context");
                let greeting = state
                    .vernal_component::<Greeting>()
                    .expect("Vernal component");
                let scope = state.vernal_request_scope().expect("request scope");
                assert!(Arc::ptr_eq(&actual, &expected));
                assert!(!scope.cancellation().is_cancelled());
                scope
                    .on_close(move || async move {
                        handler_closed.notify_one();
                        Ok::<_, io::Error>(())
                    })
                    .await
                    .expect("scope close hook");
                Ok((state, Response::new(greeting.0.into_body())))
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
        "vernal-gotham"
    );
    closed.notified().await;
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
async fn dropping_response_body_closes_request_scope() {
    let context = ready_context().await;
    let closed = Arc::new(Notify::new());
    let handler_closed = Arc::clone(&closed);
    let middleware = VernalGothamMiddleware::new(context);

    let result = middleware
        .call(request_state("/drop"), move |state| {
            Box::pin(async move {
                let scope = state.vernal_request_scope().expect("request scope");
                scope
                    .on_close(move || async move {
                        handler_closed.notify_one();
                        Ok::<_, io::Error>(())
                    })
                    .await
                    .expect("scope close hook");
                Ok((state, Response::new("stream is not consumed".into_body())))
            })
        })
        .await;
    let (_state, response) = expect_response(result);
    drop(response);

    tokio::time::timeout(Duration::from_secs(1), closed.notified())
        .await
        .expect("scope must close after Gotham response body cancellation");
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
