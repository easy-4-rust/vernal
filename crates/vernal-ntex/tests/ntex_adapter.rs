//! Ntex Middleware、Extractor、IoC 与请求作用域生命周期测试。

use std::{sync::Arc, time::Duration};

use ntex::{
    http::StatusCode,
    web::{self, App, HttpResponse, test},
};
use tokio::sync::Notify;
use vernal_context::ApplicationContextBuilder;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_ntex::{
    VernalNtexComponent, VernalNtexContext, VernalNtexMiddleware, VernalNtexRequestScope,
};

struct Greeting(&'static str);

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::singleton::<Greeting, _>(|_| {
            Greeting("vernal-ntex")
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

#[ntex::test]
async fn middleware_exposes_context_component_and_scope_until_body_finishes() {
    let context = ready_context().await;
    let expected = Arc::clone(&context);
    let closed = Arc::new(Notify::new());
    let handler_closed = Arc::clone(&closed);
    let application = test::init_service(
        App::new()
            .state(Arc::clone(&context))
            .wrap(VernalNtexMiddleware::new(Arc::clone(&context)))
            .route(
                "/hello",
                web::get().to(
                    move |VernalNtexContext(actual): VernalNtexContext,
                          VernalNtexComponent(greeting): VernalNtexComponent<Greeting>,
                          VernalNtexRequestScope(scope): VernalNtexRequestScope| {
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
                            HttpResponse::Ok().body(greeting.0)
                        }
                    },
                ),
            ),
    )
    .await;

    let request = test::TestRequest::get().uri("/hello").to_request();
    let response = test::call_service(&application, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(test::read_body(response).await, "vernal-ntex");
    closed.notified().await;
}

#[ntex::test]
async fn missing_middleware_returns_safe_internal_server_error() {
    let application = test::init_service(App::new().route(
        "/missing",
        web::get().to(|_: VernalNtexComponent<Greeting>| async { HttpResponse::Ok().finish() }),
    ))
    .await;
    let request = test::TestRequest::get().uri("/missing").to_request();
    let response = test::call_service(&application, request).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        test::read_body(response).await,
        "Vernal application context is unavailable"
    );
}

#[ntex::test]
async fn dropping_response_body_closes_request_scope() {
    let context = ready_context().await;
    let closed = Arc::new(Notify::new());
    let handler_closed = Arc::clone(&closed);
    let application =
        test::init_service(App::new().wrap(VernalNtexMiddleware::new(context)).route(
            "/drop",
            web::get().to(
                move |VernalNtexRequestScope(scope): VernalNtexRequestScope| {
                    let handler_closed = Arc::clone(&handler_closed);
                    async move {
                        scope
                            .on_close(move || async move {
                                handler_closed.notify_one();
                                Ok::<_, std::io::Error>(())
                            })
                            .await
                            .expect("scope close hook");
                        HttpResponse::Ok().body("stream is not consumed")
                    }
                },
            ),
        ))
        .await;

    let request = test::TestRequest::get().uri("/drop").to_request();
    let response = test::call_service(&application, request).await;
    drop(response);

    tokio::time::timeout(Duration::from_secs(1), closed.notified())
        .await
        .expect("scope must close after Ntex response body cancellation");
}
