//! Actix Web App Data、Middleware、Extractor 与 Scope 生命周期测试。

use std::sync::Arc;

use actix_web::{App, HttpResponse, http::StatusCode, test, web};
use tokio::sync::Notify;
use vernal_actix_web::{
    VernalActixComponent, VernalActixContext, VernalActixMiddleware, VernalActixRequestScope,
};
use vernal_context::ApplicationContextBuilder;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};

struct Greeting(&'static str);

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::singleton::<Greeting, _>(|_| {
            Greeting("vernal-actix")
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

#[actix_web::test]
async fn middleware_exposes_context_component_and_scope_until_body_finishes() {
    let context = ready_context().await;
    let expected = Arc::clone(&context);
    let closed = Arc::new(Notify::new());
    let handler_closed = Arc::clone(&closed);
    let application = test::init_service(
        App::new()
            .app_data(web::Data::from(Arc::clone(&context)))
            .wrap(VernalActixMiddleware::new(Arc::clone(&context)))
            .route(
                "/hello",
                web::get().to(
                    move |VernalActixContext(actual): VernalActixContext,
                          VernalActixComponent(greeting): VernalActixComponent<Greeting>,
                          VernalActixRequestScope(scope): VernalActixRequestScope| {
                        let expected = Arc::clone(&expected);
                        let handler_closed = Arc::clone(&handler_closed);
                        async move {
                            assert!(Arc::ptr_eq(&actual, &expected));
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
    assert_eq!(test::read_body(response).await, "vernal-actix");
    closed.notified().await;
}

#[actix_web::test]
async fn missing_context_returns_safe_internal_server_error() {
    let application = test::init_service(App::new().route(
        "/missing",
        web::get().to(|_: VernalActixComponent<Greeting>| async { HttpResponse::Ok().finish() }),
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
