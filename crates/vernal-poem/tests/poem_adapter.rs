//! Poem Middleware、Endpoint、Extractor、IoC 与请求作用域集成测试。

use std::{sync::Arc, time::Duration};

use poem::{Endpoint, EndpointExt, FromRequest, Request, endpoint::make, http::StatusCode};
use tokio::{sync::Notify, time::timeout};
use vernal_context::ApplicationContextBuilder;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_poem::{
    VernalPoemComponent, VernalPoemContext, VernalPoemMiddleware, VernalPoemRequestScope,
};

struct Greeting(&'static str);

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::singleton::<Greeting, _>(|_| {
            Greeting("vernal-poem")
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
async fn endpoint_extracts_context_component_and_request_scope() {
    let context = ready_context().await;
    let expected = Arc::clone(&context);
    let closed = Arc::new(Notify::new());
    let handler_closed = Arc::clone(&closed);
    let endpoint = make(move |request| {
        let expected = Arc::clone(&expected);
        let handler_closed = Arc::clone(&handler_closed);
        async move {
            let VernalPoemContext(actual) =
                VernalPoemContext::from_request_without_body(&request).await?;
            let VernalPoemComponent(greeting) =
                VernalPoemComponent::<Greeting>::from_request_without_body(&request).await?;
            let VernalPoemRequestScope(scope) =
                VernalPoemRequestScope::from_request_without_body(&request).await?;

            assert!(Arc::ptr_eq(&actual, &expected));
            assert!(!scope.cancellation().is_cancelled());
            scope
                .on_close(move || async move {
                    handler_closed.notify_one();
                    Ok::<_, std::io::Error>(())
                })
                .await
                .expect("scope close hook");
            Ok::<_, poem::Error>(greeting.0)
        }
    })
    .with(VernalPoemMiddleware::new(Arc::clone(&context)));

    let response = endpoint
        .call(Request::default())
        .await
        .expect("endpoint response");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .into_body()
            .into_string()
            .await
            .expect("response body"),
        "vernal-poem"
    );
    closed.notified().await;
}

#[tokio::test]
async fn missing_vernal_middleware_returns_safe_rejection() {
    let endpoint = make(|request| async move {
        VernalPoemComponent::<Greeting>::from_request_without_body(&request)
            .await
            .map(|_| "unreachable")
    });

    let response = endpoint.get_response(Request::default()).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response
            .into_body()
            .into_string()
            .await
            .expect("response body"),
        "Vernal application context is unavailable"
    );
}

#[tokio::test]
async fn dropping_response_body_closes_request_scope() {
    let context = ready_context().await;
    let closed = Arc::new(Notify::new());
    let handler_closed = Arc::clone(&closed);
    let endpoint = make(move |request| {
        let handler_closed = Arc::clone(&handler_closed);
        async move {
            let VernalPoemRequestScope(scope) =
                VernalPoemRequestScope::from_request_without_body(&request).await?;
            scope
                .on_close(move || async move {
                    handler_closed.notify_one();
                    Ok::<_, std::io::Error>(())
                })
                .await
                .expect("scope close hook");
            Ok::<_, poem::Error>("stream is not consumed")
        }
    })
    .with(VernalPoemMiddleware::new(context));

    let response = endpoint
        .call(Request::default())
        .await
        .expect("endpoint response");
    drop(response);

    timeout(Duration::from_secs(1), closed.notified())
        .await
        .expect("scope must close after response body cancellation");
}
