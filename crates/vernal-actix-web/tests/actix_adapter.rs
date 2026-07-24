//! Actix Web App Data、Middleware、Extractor 与 Scope 生命周期测试。

#[path = "aop_support/policy_deny_local_interceptor.rs"]
mod policy_deny_local_interceptor;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use actix_web::{App, HttpResponse, error::ErrorNotFound, http::StatusCode, test, web};
use policy_deny_local_interceptor::PolicyDenyLocalInterceptor;
use tokio::sync::Notify;
use vernal_actix_web::{
    VernalActixComponent, VernalActixContext, VernalActixMiddleware, VernalActixRequestContext,
    VernalActixRequestScope,
};
use vernal_aop::{LocalAdvisor, Operation};
use vernal_context::{ApplicationContextBuilder, VernalApplicationBuilder};
use vernal_http::HttpRequestSnapshot;
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

async fn ready_aop_context(
    operation: Operation,
    advisor: Option<LocalAdvisor>,
) -> Arc<vernal_context::ApplicationContext> {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder.operation(operation);
    if let Some(advisor) = advisor {
        builder.local_advisor(advisor);
    }
    let context = Arc::new(builder.build().expect("Local-AOP context build"));
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

#[actix_web::test]
async fn strict_local_aop_uses_matched_pattern_and_owned_snapshot() {
    let context = ready_aop_context(Operation::new("/orders/{id}", "GET"), None).await;
    let application = test::init_service(
        App::new().service(
            web::resource("/orders/{id}")
                .wrap(VernalActixMiddleware::strict_aop(Arc::clone(&context)))
                .route(web::get().to(
                    |VernalActixRequestContext(context): VernalActixRequestContext| async move {
                        let snapshot = context
                            .extensions()
                            .get::<HttpRequestSnapshot>()
                            .await
                            .expect("owned HTTP snapshot");
                        assert_eq!(context.route().handler(), "/orders/{id}");
                        assert_eq!(context.route().operation_name(), "GET");
                        assert_eq!(context.route().path_template(), "/orders/{id}");
                        assert_eq!(snapshot.method().as_str(), "GET");
                        assert_eq!(snapshot.uri().path(), "/orders/42");
                        HttpResponse::Ok().body("locally-woven")
                    },
                )),
        ),
    )
    .await;

    let request = test::TestRequest::get()
        .uri("/orders/42")
        .insert_header(("x-request-id", "request-42"))
        .to_request();
    let response = test::call_service(&application, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(test::read_body(response).await, "locally-woven");
}

#[actix_web::test]
async fn strict_local_aop_maps_policy_failure_without_calling_handler() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(
        Operation::new("/protected", "GET"),
        Some(LocalAdvisor::new(
            |_: &Operation| true,
            PolicyDenyLocalInterceptor,
            -1000,
        )),
    )
    .await;
    let handler_called = Arc::clone(&called);
    let application = test::init_service(
        App::new().service(
            web::resource("/protected")
                .wrap(VernalActixMiddleware::strict_aop(Arc::clone(&context)))
                .route(web::get().to(move || {
                    handler_called.store(true, Ordering::SeqCst);
                    async { HttpResponse::Ok().body("unreachable") }
                })),
        ),
    )
    .await;

    let request = test::TestRequest::get().uri("/protected").to_request();
    let response = test::call_service(&application, request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        test::read_body(response).await,
        "Authentication is required"
    );
    assert!(!called.load(Ordering::SeqCst));
}

#[actix_web::test]
async fn strict_local_aop_fails_closed_when_plan_is_missing() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(Operation::new("/declared", "GET"), None).await;
    let handler_called = Arc::clone(&called);
    let application = test::init_service(
        App::new().service(
            web::resource("/unplanned")
                .wrap(VernalActixMiddleware::strict_aop(Arc::clone(&context)))
                .route(web::get().to(move || {
                    handler_called.store(true, Ordering::SeqCst);
                    async { HttpResponse::Ok().body("unreachable") }
                })),
        ),
    )
    .await;

    let request = test::TestRequest::get().uri("/unplanned").to_request();
    let response = test::call_service(&application, request).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        test::read_body(response).await,
        "Vernal Local-AOP invocation failed"
    );
    assert!(!called.load(Ordering::SeqCst));
}

#[actix_web::test]
async fn strict_local_aop_preserves_native_actix_error_response() {
    let context = ready_aop_context(Operation::new("/missing", "GET"), None).await;
    let application = test::init_service(
        App::new().service(
            web::resource("/missing")
                .wrap(VernalActixMiddleware::strict_aop(Arc::clone(&context)))
                .route(
                    web::get()
                        .to(|| async { Err::<HttpResponse, _>(ErrorNotFound("native missing")) }),
                ),
        ),
    )
    .await;

    let request = test::TestRequest::get().uri("/missing").to_request();
    let response = test::call_service(&application, request).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(test::read_body(response).await, "native missing");
}
