//! Ntex Middleware、Extractor、IoC 与请求作用域生命周期测试。

use std::{
    future::poll_fn,
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use futures_util::stream;
use ntex::{
    http::{
        StatusCode,
        body::{Body, BodyStream, MessageBody, ResponseBody},
    },
    web::{self, App, DefaultError, Error, HttpResponse, error::ErrorNotFound, test},
};
use tokio_util::sync::CancellationToken;
use vernal_aop::{LocalAdvisor, Operation};
use vernal_context::{ApplicationContextBuilder, VernalApplicationBuilder};
use vernal_http::HttpRequestSnapshot;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_ntex::{
    NtexScopedBody, VernalNtexComponent, VernalNtexContext, VernalNtexMiddleware,
    VernalNtexRequestContext, VernalNtexRequestScope,
};
use vernal_web::WebRequestScope;
use vernal_web_testkit::{
    FailingByteStream, ScopeCleanupTimeoutFixture, ScopeCloseProbe, ScopeRejectingInterceptor,
    SecurityContractInterceptor, WebAdapterContract,
};

struct Greeting(&'static str);

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<Greeting, WebRequestScope, _>(
            |_| Greeting("vernal-ntex"),
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

#[ntex::test]
async fn middleware_exposes_context_component_and_scope_until_body_finishes() {
    let context = ready_context().await;
    let expected = Arc::clone(&context);
    let probe = Arc::new(ScopeCloseProbe::new());
    let handler_probe = Arc::clone(&probe);
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
                        let handler_probe = Arc::clone(&handler_probe);
                        async move {
                            WebAdapterContract::assert_request_binding(
                                &expected,
                                &actual,
                                &scope,
                                &greeting,
                            );
                            handler_probe.observe(&scope);
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
    probe.assert_open();
    assert_eq!(test::read_body(response).await, "vernal-ntex");
    probe.assert_closed_within(Duration::from_secs(1)).await;
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
async fn partial_response_consumption_then_disconnect_closes_request_scope() {
    let context = ready_context().await;
    let probe = Arc::new(ScopeCloseProbe::new());
    let handler_probe = Arc::clone(&probe);
    let application =
        test::init_service(App::new().wrap(VernalNtexMiddleware::new(context)).route(
            "/drop",
            web::get().to(
                move |VernalNtexRequestScope(scope): VernalNtexRequestScope| {
                    let handler_probe = Arc::clone(&handler_probe);
                    async move {
                        handler_probe.observe(&scope);
                        HttpResponse::Ok().body("stream is not consumed")
                    }
                },
            ),
        ))
        .await;

    let request = test::TestRequest::get().uri("/drop").to_request();
    let mut response = test::call_service(&application, request).await;
    let mut body = response.take_body();
    let data = poll_fn(|context| body.poll_next_chunk(context))
        .await
        .expect("Ntex response must emit one data chunk")
        .expect("Ntex data chunk must succeed");
    assert_eq!(data, "stream is not consumed");
    probe.assert_open();
    drop(body);

    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[ntex::test]
async fn ntex_body_error_closes_scope_before_restoring_upstream_error() {
    let scope = Arc::new(WebRequestScope::new(CancellationToken::new()));
    let upstream = stream::iter([Err::<ntex::util::Bytes, _>(io::Error::other(
        FailingByteStream::error_message(),
    ))]);
    let body = ResponseBody::Other(Body::from_message(BodyStream::new(upstream)));
    let mut scoped = NtexScopedBody::new(body, Arc::clone(&scope), CancellationToken::new());
    WebAdapterContract::assert_scope_open(&scope);

    let error = poll_fn(|context| scoped.poll_next_chunk(context))
        .await
        .expect("Ntex body must report one upstream error")
        .expect_err("synthetic stream must fail");
    assert_eq!(error.to_string(), FailingByteStream::error_message());
    WebAdapterContract::assert_scope_closed(&scope);
}

#[ntex::test]
async fn ntex_body_reports_cleanup_timeout_while_background_close_continues() {
    let fixture = ScopeCleanupTimeoutFixture::new(Duration::from_millis(10)).await;
    let upstream = stream::iter([Err::<ntex::util::Bytes, _>(io::Error::other(
        FailingByteStream::error_message(),
    ))]);
    let body = ResponseBody::Other(Body::from_message(BodyStream::new(upstream)));
    let mut scoped = NtexScopedBody::new(
        body,
        fixture.scope(),
        fixture.scope().cancellation().clone(),
    );

    let error = poll_fn(|context| scoped.poll_next_chunk(context))
        .await
        .expect("Ntex body must report one cleanup error")
        .expect_err("Ntex body must report the scope cleanup timeout");
    assert!(
        error.to_string().contains("cleanup exceeded timeout"),
        "unexpected Ntex cleanup error: {error}"
    );
    fixture.assert_cleanup_timed_out();
    fixture.assert_redacted_warning().await;
    fixture.release();
    fixture.assert_closed_within(Duration::from_secs(1)).await;
}

#[ntex::test]
async fn strict_local_aop_uses_declared_pattern_and_owned_snapshot() {
    let operation = Operation::new("/orders/{id}", "GET");
    let context = ready_aop_context(operation, None).await;
    let application = test::init_service(
        App::new().service(
            web::resource("/orders/{id}")
                .wrap(VernalNtexMiddleware::strict_aop(
                    Arc::clone(&context),
                    "/orders/{id}",
                ))
                .route(web::get().to(
                    |VernalNtexRequestContext(context): VernalNtexRequestContext| async move {
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
        .header("x-request-id", "request-42")
        .to_request();
    let response = test::call_service(&application, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(test::read_body(response).await, "locally-woven");
}

#[ntex::test]
async fn strict_local_aop_maps_policy_failure_without_calling_handler() {
    let called = Arc::new(AtomicBool::new(false));
    let probe = Arc::new(ScopeCloseProbe::new());
    let operation = Operation::new("/protected", "GET");
    let context = ready_aop_context(
        operation,
        Some(LocalAdvisor::new(
            |_: &Operation| true,
            ScopeRejectingInterceptor::new(Arc::clone(&probe)),
            -1000,
        )),
    )
    .await;
    let handler_called = Arc::clone(&called);
    let application = test::init_service(
        App::new().service(
            web::resource("/protected")
                .wrap(VernalNtexMiddleware::strict_aop(
                    Arc::clone(&context),
                    "/protected",
                ))
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
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[ntex::test]
async fn strict_local_aop_maps_authenticated_forbidden_to_403() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(
        Operation::new("/protected", "GET"),
        Some(LocalAdvisor::new(
            |_: &Operation| true,
            SecurityContractInterceptor::forbidden("operator-7", ["operator"]),
            -1000,
        )),
    )
    .await;
    let handler_called = Arc::clone(&called);
    let application = test::init_service(
        App::new().service(
            web::resource("/protected")
                .wrap(VernalNtexMiddleware::strict_aop(
                    Arc::clone(&context),
                    "/protected",
                ))
                .route(web::get().to(move || {
                    handler_called.store(true, Ordering::SeqCst);
                    async { HttpResponse::Ok().body("unreachable") }
                })),
        ),
    )
    .await;

    let request = test::TestRequest::get().uri("/protected").to_request();
    let response = test::call_service(&application, request).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(test::read_body(response).await, "Access is forbidden");
    assert!(!called.load(Ordering::SeqCst));
}

#[ntex::test]
async fn strict_local_aop_fails_closed_when_plan_is_missing() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(Operation::new("/declared", "GET"), None).await;
    let handler_called = Arc::clone(&called);
    let application = test::init_service(
        App::new().service(
            web::resource("/unplanned")
                .wrap(VernalNtexMiddleware::strict_aop(
                    Arc::clone(&context),
                    "/unplanned",
                ))
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

#[ntex::test]
async fn strict_local_aop_preserves_native_ntex_error_response() {
    let context = ready_aop_context(Operation::new("/missing", "GET"), None).await;
    let application = test::init_service(
        App::new().service(
            web::resource("/missing")
                .wrap(VernalNtexMiddleware::strict_aop(
                    Arc::clone(&context),
                    "/missing",
                ))
                .route(web::get().to(|| async {
                    Err::<HttpResponse, Error>(
                        ErrorNotFound::<_, DefaultError>("native missing").into(),
                    )
                })),
        ),
    )
    .await;

    let request = test::TestRequest::get().uri("/missing").to_request();
    let response = test::call_service(&application, request).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(test::read_body(response).await, "native missing");
}
