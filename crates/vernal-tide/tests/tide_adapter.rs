//! Tide Middleware、Request Extension、IoC 与 Body 生命周期测试。

use std::{io, sync::Arc, time::Duration};

use tide::{
    Request, Response, StatusCode,
    http::{Method, Request as HttpRequest, Url},
};
use tokio::sync::Notify;
use vernal_context::ApplicationContextBuilder;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_tide::{VernalTideMiddleware, VernalTideRequestExt};

struct Greeting(&'static str);

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::singleton::<Greeting, _>(|_| {
            Greeting("vernal-tide")
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

fn request(path: &str) -> HttpRequest {
    HttpRequest::new(
        Method::Get,
        Url::parse(&format!("http://localhost{path}")).expect("request URL"),
    )
}

#[tokio::test]
async fn middleware_exposes_context_component_and_scope_until_body_finishes() {
    let context = ready_context().await;
    let expected = Arc::clone(&context);
    let closed = Arc::new(Notify::new());
    let handler_closed = Arc::clone(&closed);
    let mut application = tide::new();
    application.with(VernalTideMiddleware::new(context));
    application.at("/hello").get(move |request: Request<()>| {
        let expected = Arc::clone(&expected);
        let handler_closed = Arc::clone(&handler_closed);
        async move {
            let actual = request.vernal_context().expect("Vernal context");
            let greeting = request
                .vernal_component::<Greeting>()
                .expect("Vernal component");
            let scope = request.vernal_request_scope().expect("request scope");
            assert!(Arc::ptr_eq(&actual, &expected));
            assert!(!scope.cancellation().is_cancelled());
            scope
                .on_close(move || async move {
                    handler_closed.notify_one();
                    Ok::<_, io::Error>(())
                })
                .await
                .expect("scope close hook");
            Ok(greeting.0)
        }
    });

    let mut response: Response = application
        .respond(request("/hello"))
        .await
        .expect("Tide response");
    assert_eq!(response.status(), StatusCode::Ok);
    assert_eq!(response.len(), Some("vernal-tide".len()));
    assert_eq!(
        response
            .take_body()
            .into_string()
            .await
            .expect("response body"),
        "vernal-tide"
    );
    closed.notified().await;
}

#[tokio::test]
async fn missing_middleware_returns_safe_internal_server_error() {
    let mut application = tide::new();
    application
        .at("/missing")
        .get(|request: Request<()>| async move {
            match request.vernal_component::<Greeting>() {
                Ok(_) => Ok(Response::new(StatusCode::Ok)),
                Err(rejection) => Ok(rejection.response()),
            }
        });

    let mut response: Response = application
        .respond(request("/missing"))
        .await
        .expect("Tide response");
    assert_eq!(response.status(), StatusCode::InternalServerError);
    assert_eq!(
        response
            .take_body()
            .into_string()
            .await
            .expect("rejection body"),
        "Vernal application context is unavailable"
    );
}

#[tokio::test]
async fn dropping_response_body_closes_request_scope() {
    let context = ready_context().await;
    let closed = Arc::new(Notify::new());
    let handler_closed = Arc::clone(&closed);
    let mut application = tide::new();
    application.with(VernalTideMiddleware::new(context));
    application.at("/drop").get(move |request: Request<()>| {
        let handler_closed = Arc::clone(&handler_closed);
        async move {
            let scope = request.vernal_request_scope().expect("request scope");
            scope
                .on_close(move || async move {
                    handler_closed.notify_one();
                    Ok::<_, io::Error>(())
                })
                .await
                .expect("scope close hook");
            Ok("stream is not consumed")
        }
    });

    let response: Response = application
        .respond(request("/drop"))
        .await
        .expect("Tide response");
    drop(response);

    tokio::time::timeout(Duration::from_secs(1), closed.notified())
        .await
        .expect("scope must close after Tide response body cancellation");
}
