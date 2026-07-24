//! Poem Middleware、Endpoint、Extractor、IoC 与请求作用域集成测试。

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use poem::{
    Endpoint, EndpointExt, FromRequest, Request, Route,
    endpoint::make,
    error::NotFoundError,
    http::{Method, StatusCode, Uri},
};
use vernal_aop::{Advisor, Operation};
use vernal_context::{ApplicationContextBuilder, VernalApplicationBuilder};
use vernal_http::HttpRequestSnapshot;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_poem::{
    VernalPoemComponent, VernalPoemContext, VernalPoemMiddleware, VernalPoemRequestContext,
    VernalPoemRequestScope,
};
use vernal_web::WebRequestScope;
use vernal_web_testkit::{ScopeCloseProbe, ScopeRejectingInterceptor, WebAdapterContract};

struct Greeting(&'static str);

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<Greeting, WebRequestScope, _>(
            |_| Greeting("vernal-poem"),
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

#[tokio::test]
async fn endpoint_extracts_context_component_and_request_scope() {
    let context = ready_context().await;
    let expected = Arc::clone(&context);
    let probe = Arc::new(ScopeCloseProbe::new());
    let handler_probe = Arc::clone(&probe);
    let endpoint = make(move |request| {
        let expected = Arc::clone(&expected);
        let handler_probe = Arc::clone(&handler_probe);
        async move {
            let VernalPoemContext(actual) =
                VernalPoemContext::from_request_without_body(&request).await?;
            let VernalPoemComponent(greeting) =
                VernalPoemComponent::<Greeting>::from_request_without_body(&request).await?;
            let VernalPoemRequestScope(scope) =
                VernalPoemRequestScope::from_request_without_body(&request).await?;

            WebAdapterContract::assert_request_binding(&expected, &actual, &scope, &greeting);
            handler_probe.observe(&scope);
            Ok::<_, poem::Error>(greeting.0)
        }
    })
    .with(VernalPoemMiddleware::new(Arc::clone(&context)));

    let response = endpoint
        .call(Request::default())
        .await
        .expect("endpoint response");
    assert_eq!(response.status(), StatusCode::OK);
    probe.assert_open();
    assert_eq!(
        response
            .into_body()
            .into_string()
            .await
            .expect("response body"),
        "vernal-poem"
    );
    probe.assert_closed_within(Duration::from_secs(1)).await;
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
    let probe = Arc::new(ScopeCloseProbe::new());
    let handler_probe = Arc::clone(&probe);
    let endpoint = make(move |request| {
        let handler_probe = Arc::clone(&handler_probe);
        async move {
            let VernalPoemRequestScope(scope) =
                VernalPoemRequestScope::from_request_without_body(&request).await?;
            handler_probe.observe(&scope);
            Ok::<_, poem::Error>("stream is not consumed")
        }
    })
    .with(VernalPoemMiddleware::new(context));

    let response = endpoint
        .call(Request::default())
        .await
        .expect("endpoint response");
    probe.assert_open();
    drop(response);

    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn strict_aop_endpoint_uses_matched_path_and_propagates_owned_snapshot() {
    let context = ready_aop_context(Operation::new("/orders/:id", "GET"), None).await;
    let endpoint = make(|request| async move {
        let VernalPoemRequestContext(context) =
            VernalPoemRequestContext::from_request_without_body(&request).await?;
        let snapshot = context
            .extensions()
            .get::<HttpRequestSnapshot>()
            .await
            .expect("owned HTTP snapshot");

        assert_eq!(context.route().handler(), "/orders/:id");
        assert_eq!(context.route().operation_name(), "GET");
        assert_eq!(context.route().path_template(), "/orders/:id");
        assert_eq!(snapshot.method(), Method::GET);
        assert_eq!(snapshot.uri().path(), "/orders/42");
        Ok::<_, poem::Error>("woven")
    })
    .with(VernalPoemMiddleware::strict_aop(Arc::clone(&context)));
    let app = Route::new().at("/orders/:id", endpoint);

    let response = app
        .call(
            Request::builder()
                .method(Method::GET)
                .uri("/orders/42".parse::<Uri>().expect("orders URI"))
                .finish(),
        )
        .await
        .expect("strict AOP response");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .into_body()
            .into_string()
            .await
            .expect("response body"),
        "woven"
    );
}

#[tokio::test]
async fn strict_aop_endpoint_maps_policy_failure_without_calling_handler() {
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
    let endpoint = make(move |_| {
        handler_called.store(true, Ordering::SeqCst);
        async { Ok::<_, poem::Error>("unreachable") }
    })
    .with(VernalPoemMiddleware::strict_aop(Arc::clone(&context)));
    let app = Route::new().at("/protected", endpoint);

    let response = app
        .get_response(
            Request::builder()
                .method(Method::GET)
                .uri("/protected".parse::<Uri>().expect("protected URI"))
                .finish(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response
            .into_body()
            .into_string()
            .await
            .expect("response body"),
        "Authentication is required"
    );
    assert!(!called.load(Ordering::SeqCst));
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn strict_aop_endpoint_fails_closed_without_matched_path_metadata() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(Operation::new("/bare", "GET"), None).await;
    let handler_called = Arc::clone(&called);
    let endpoint = make(move |_| {
        handler_called.store(true, Ordering::SeqCst);
        async { Ok::<_, poem::Error>("unreachable") }
    })
    .with(VernalPoemMiddleware::strict_aop(context));

    let response = endpoint
        .get_response(
            Request::builder()
                .method(Method::GET)
                .uri("/bare".parse::<Uri>().expect("bare URI"))
                .finish(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response
            .into_body()
            .into_string()
            .await
            .expect("response body"),
        "Poem matched route metadata is unavailable"
    );
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn strict_aop_endpoint_fails_closed_when_operation_plan_is_missing() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(Operation::new("/declared", "GET"), None).await;
    let handler_called = Arc::clone(&called);
    let endpoint = make(move |_| {
        handler_called.store(true, Ordering::SeqCst);
        async { Ok::<_, poem::Error>("unreachable") }
    })
    .with(VernalPoemMiddleware::strict_aop(Arc::clone(&context)));
    let app = Route::new().at("/unplanned", endpoint);

    let response = app
        .get_response(
            Request::builder()
                .method(Method::GET)
                .uri("/unplanned".parse::<Uri>().expect("unplanned URI"))
                .finish(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response
            .into_body()
            .into_string()
            .await
            .expect("response body"),
        "Vernal AOP invocation failed"
    );
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn strict_aop_endpoint_preserves_native_poem_error_status() {
    let context = ready_aop_context(Operation::new("/missing", "GET"), None).await;
    let endpoint = make(|_| async { Err::<&'static str, poem::Error>(NotFoundError.into()) })
        .with(VernalPoemMiddleware::strict_aop(Arc::clone(&context)));
    let app = Route::new().at("/missing", endpoint);

    let error = app
        .call(
            Request::builder()
                .method(Method::GET)
                .uri("/missing".parse::<Uri>().expect("missing URI"))
                .finish(),
        )
        .await
        .expect_err("native endpoint error");
    assert_eq!(error.status(), StatusCode::NOT_FOUND);
    assert!(error.is::<NotFoundError>());
}
