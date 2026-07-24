//! Tide Middleware、Request Extension、IoC 与 Body 生命周期测试。

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use tide::{
    Request, Response, StatusCode,
    http::{Method, Request as HttpRequest, Url},
};
use vernal_aop::{Advisor, Operation};
use vernal_context::{ApplicationContextBuilder, VernalApplicationBuilder};
use vernal_http::{HttpRequestSnapshot, Method as SnapshotMethod};
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_tide::{VernalTideMiddleware, VernalTideRequestExt};
use vernal_web::WebRequestScope;
use vernal_web_testkit::{ScopeCloseProbe, ScopeRejectingInterceptor, WebAdapterContract};

struct Greeting(&'static str);

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<Greeting, WebRequestScope, _>(
            |_| Greeting("vernal-tide"),
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

async fn response_body(response: &mut Response) -> String {
    response
        .take_body()
        .into_string()
        .await
        .expect("response body")
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
    let probe = Arc::new(ScopeCloseProbe::new());
    let handler_probe = Arc::clone(&probe);
    let mut application = tide::new();
    application.with(VernalTideMiddleware::new(context));
    application.at("/hello").get(move |request: Request<()>| {
        let expected = Arc::clone(&expected);
        let handler_probe = Arc::clone(&handler_probe);
        async move {
            let actual = request.vernal_context().expect("Vernal context");
            let greeting = request
                .vernal_component::<Greeting>()
                .expect("Vernal component");
            let scope = request.vernal_request_scope().expect("request scope");
            WebAdapterContract::assert_request_binding(&expected, &actual, &scope, &greeting);
            handler_probe.observe(&scope);
            Ok(greeting.0)
        }
    });

    let mut response: Response = application
        .respond(request("/hello"))
        .await
        .expect("Tide response");
    assert_eq!(response.status(), StatusCode::Ok);
    assert_eq!(response.len(), Some("vernal-tide".len()));
    probe.assert_open();
    assert_eq!(
        response
            .take_body()
            .into_string()
            .await
            .expect("response body"),
        "vernal-tide"
    );
    probe.assert_closed_within(Duration::from_secs(1)).await;
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
    let probe = Arc::new(ScopeCloseProbe::new());
    let handler_probe = Arc::clone(&probe);
    let mut application = tide::new();
    application.with(VernalTideMiddleware::new(context));
    application.at("/drop").get(move |request: Request<()>| {
        let handler_probe = Arc::clone(&handler_probe);
        async move {
            let scope = request.vernal_request_scope().expect("request scope");
            handler_probe.observe(&scope);
            Ok("stream is not consumed")
        }
    });

    let response: Response = application
        .respond(request("/drop"))
        .await
        .expect("Tide response");
    probe.assert_open();
    drop(response);

    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn strict_aop_uses_declared_pattern_and_owned_snapshot() {
    let context = ready_aop_context(Operation::new("/orders/:id", "GET"), None).await;
    let mut application = tide::new();
    application
        .at("/orders/:id")
        .with(VernalTideMiddleware::strict_aop(
            Arc::clone(&context),
            "/orders/:id",
        ))
        .get(|request: Request<()>| async move {
            let request_context = request
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
            assert_eq!(snapshot.method(), SnapshotMethod::GET);
            assert_eq!(snapshot.uri().path(), "/orders/42");
            Ok("woven")
        });

    let mut response: Response = application
        .respond(request("/orders/42"))
        .await
        .expect("strict Tide response");

    assert_eq!(response.status(), StatusCode::Ok);
    assert_eq!(response_body(&mut response).await, "woven");
}

#[tokio::test]
async fn strict_aop_maps_policy_failure_without_calling_endpoint() {
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
    let endpoint_called = Arc::clone(&called);
    let mut application = tide::new();
    application
        .at("/protected")
        .with(VernalTideMiddleware::strict_aop(
            Arc::clone(&context),
            "/protected",
        ))
        .get(move |_| {
            endpoint_called.store(true, Ordering::SeqCst);
            async { Ok("unreachable") }
        });

    let mut response: Response = application
        .respond(request("/protected"))
        .await
        .expect("policy Tide response");

    assert_eq!(response.status(), StatusCode::Unauthorized);
    assert_eq!(
        response_body(&mut response).await,
        "Authentication is required"
    );
    assert!(!called.load(Ordering::SeqCst));
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn strict_aop_fails_closed_when_plan_is_missing() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(Operation::new("/different", "GET"), None).await;
    let endpoint_called = Arc::clone(&called);
    let mut application = tide::new();
    application
        .at("/missing-plan")
        .with(VernalTideMiddleware::strict_aop(
            Arc::clone(&context),
            "/missing-plan",
        ))
        .get(move |_| {
            endpoint_called.store(true, Ordering::SeqCst);
            async { Ok("unreachable") }
        });

    let mut response: Response = application
        .respond(request("/missing-plan"))
        .await
        .expect("missing plan Tide response");

    assert_eq!(response.status(), StatusCode::InternalServerError);
    assert_eq!(
        response_body(&mut response).await,
        "Vernal AOP invocation failed"
    );
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn strict_aop_fails_closed_when_pattern_is_empty() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(Operation::new("/empty", "GET"), None).await;
    let endpoint_called = Arc::clone(&called);
    let mut application = tide::new();
    application
        .at("/empty")
        .with(VernalTideMiddleware::strict_aop(Arc::clone(&context), ""))
        .get(move |_| {
            endpoint_called.store(true, Ordering::SeqCst);
            async { Ok("unreachable") }
        });

    let mut response: Response = application
        .respond(request("/empty"))
        .await
        .expect("missing metadata Tide response");

    assert_eq!(response.status(), StatusCode::InternalServerError);
    assert_eq!(
        response_body(&mut response).await,
        "Tide route pattern is unavailable"
    );
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn strict_aop_preserves_native_tide_response() {
    let context = ready_aop_context(Operation::new("/native-error", "GET"), None).await;
    let mut application = tide::new();
    application
        .at("/native-error")
        .with(VernalTideMiddleware::strict_aop(
            Arc::clone(&context),
            "/native-error",
        ))
        .get(|_| async {
            let mut response = Response::new(StatusCode::NotFound);
            response.set_body("native-not-found");
            Ok(response)
        });

    let mut response: Response = application
        .respond(request("/native-error"))
        .await
        .expect("native Tide response");

    assert_eq!(response.status(), StatusCode::NotFound);
    assert_eq!(response_body(&mut response).await, "native-not-found");
}
