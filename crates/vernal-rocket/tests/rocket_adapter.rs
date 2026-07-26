//! Rocket Fairing、Managed State、Request Guard 与请求作用域集成测试。

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use rocket::{
    Request, get,
    http::Status,
    local::asynchronous::Client,
    request::{FromRequest, Outcome as RequestOutcome},
    response::Response as RocketResponse,
    routes,
    tokio::io::AsyncReadExt,
};
use tokio_util::sync::CancellationToken;
use vernal_aop::{Advisor, Operation};
use vernal_beans::{ComponentDefinition, RegistryBuilder};
use vernal_context::{ApplicationContextBuilder, VernalApplicationBuilder};
use vernal_http::HttpRequestSnapshot;
use vernal_rocket::{
    RocketScopedReader, VernalRocketComponent, VernalRocketContext, VernalRocketFairing,
    VernalRocketRequestContext, VernalRocketRequestScope, VernalRocketRoutesExt,
};
use vernal_web::WebRequestScope;
use vernal_web_testkit::{
    FailingTokioReader, ScopeCleanupTimeoutFixture, ScopeCloseProbe, ScopeRejectingInterceptor,
    SecurityContractInterceptor, WebAdapterContract,
};

struct Greeting(&'static str);

static PROTECTED_CALLED: AtomicBool = AtomicBool::new(false);

#[get("/hello")]
fn hello(
    context: VernalRocketContext,
    greeting: VernalRocketComponent<Greeting>,
    probe: VernalRocketComponent<ScopeCloseProbe>,
    scope: VernalRocketRequestScope,
) -> &'static str {
    let VernalRocketContext(context) = context;
    let VernalRocketComponent(greeting) = greeting;
    let VernalRocketComponent(probe) = probe;
    let VernalRocketRequestScope(scope) = scope;
    WebAdapterContract::assert_request_binding(&context, &context, &scope, &greeting);
    probe.observe(&scope);
    greeting.0
}

async fn ready_context_with_greeting(
    probe: Arc<ScopeCloseProbe>,
    greeting: &'static str,
) -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<Greeting, WebRequestScope, _>(
            move |_| Greeting(greeting),
        ))
        .expect("greeting registration");
    registry
        .register(ComponentDefinition::shared_arc(probe))
        .expect("scope probe registration");
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

async fn ready_context(probe: Arc<ScopeCloseProbe>) -> Arc<vernal_context::ApplicationContext> {
    ready_context_with_greeting(probe, "vernal-rocket").await
}

async fn ready_aop_context(
    operations: impl IntoIterator<Item = Operation>,
    advisor: Option<Advisor>,
) -> Arc<vernal_context::ApplicationContext> {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    for operation in operations {
        builder.operation(operation);
    }
    if let Some(advisor) = advisor {
        builder.advisor(advisor);
    }
    let context = Arc::new(builder.build().expect("AOP context build"));
    context.refresh().await.expect("AOP context refresh");
    context.start().await.expect("AOP context start");
    context
}

#[get("/orders/<id>")]
async fn strict_probe(id: u64, context: VernalRocketRequestContext) -> &'static str {
    let snapshot = context
        .0
        .extensions()
        .get::<HttpRequestSnapshot>()
        .await
        .expect("owned HTTP snapshot");
    assert_eq!(id, 42);
    assert_eq!(context.0.route().handler(), "/api/orders/<id>");
    assert_eq!(context.0.route().operation_name(), "GET");
    assert_eq!(context.0.route().path_template(), "/api/orders/<id>");
    assert_eq!(snapshot.uri().path(), "/api/orders/42");
    "woven"
}

#[get("/protected")]
fn protected() -> &'static str {
    PROTECTED_CALLED.store(true, Ordering::SeqCst);
    "unreachable"
}

#[get("/missing-plan")]
fn missing_plan() -> &'static str {
    PROTECTED_CALLED.store(true, Ordering::SeqCst);
    "unreachable"
}

struct ForwardGuard;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ForwardGuard {
    type Error = ();

    async fn from_request(_request: &'r Request<'_>) -> RequestOutcome<Self, Self::Error> {
        RequestOutcome::Forward(Status::NotFound)
    }
}

#[get("/forward", rank = 1)]
fn forwarding(_guard: ForwardGuard) -> &'static str {
    "unreachable"
}

#[get("/forward", rank = 2)]
fn forward_fallback() -> &'static str {
    "forwarded"
}

struct ErrorGuard;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ErrorGuard {
    type Error = ();

    async fn from_request(_request: &'r Request<'_>) -> RequestOutcome<Self, Self::Error> {
        RequestOutcome::Error((Status::ImATeapot, ()))
    }
}

#[get("/guard-error")]
fn guard_error(_guard: ErrorGuard) -> &'static str {
    "unreachable"
}

#[rocket::async_test]
async fn fairing_exposes_context_component_and_scope_until_body_finishes() {
    let probe = Arc::new(ScopeCloseProbe::new());
    let context = ready_context(Arc::clone(&probe)).await;
    let rocket = rocket::build()
        .attach(VernalRocketFairing::new(context))
        .mount("/", routes![hello]);
    let client = Client::tracked(rocket).await.expect("Rocket client");

    let response = client.get("/hello").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    probe.assert_open();
    assert_eq!(
        response.into_string().await.expect("response body"),
        "vernal-rocket"
    );
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[rocket::async_test]
async fn existing_managed_context_is_the_authority_for_request_scope() {
    let managed_probe = Arc::new(ScopeCloseProbe::new());
    let managed_context =
        ready_context_with_greeting(Arc::clone(&managed_probe), "managed-context").await;
    let fairing_context =
        ready_context_with_greeting(Arc::new(ScopeCloseProbe::new()), "fairing-context").await;
    let rocket = rocket::build()
        .manage(Arc::clone(&managed_context))
        .attach(VernalRocketFairing::new(fairing_context))
        .mount("/", routes![hello]);
    let client = Client::tracked(rocket).await.expect("Rocket client");

    // Handler 内的共享契约同时检查 Managed Context、Scope Owner 与请求组件
    // 的 Arc 身份，因此这里不仅验证字符串值，也验证三者使用同一组件图。
    let response = client.get("/hello").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    managed_probe.assert_open();
    assert_eq!(
        response.into_string().await.as_deref(),
        Some("managed-context")
    );
    managed_probe
        .assert_closed_within(Duration::from_secs(1))
        .await;
}

#[rocket::async_test]
async fn missing_fairing_rejects_request_without_leaking_internal_error() {
    let rocket = rocket::build().mount("/", routes![hello]);
    let client = Client::tracked(rocket).await.expect("Rocket client");

    let response = client.get("/hello").dispatch().await;
    assert_eq!(response.status(), Status::InternalServerError);
    let body = response.into_string().await.expect("error body");
    assert!(!body.contains("component resolution"));
    assert!(!body.contains("ResolveError"));
}

#[rocket::async_test]
async fn partial_response_consumption_then_disconnect_closes_request_scope() {
    let probe = Arc::new(ScopeCloseProbe::new());
    let context = ready_context(Arc::clone(&probe)).await;
    let rocket = rocket::build()
        .attach(VernalRocketFairing::new(context))
        .mount("/", routes![hello]);
    let client = Client::tracked(rocket).await.expect("Rocket client");

    let mut response = client.get("/hello").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    let mut first_byte = [0_u8; 1];
    assert_eq!(
        response
            .read(&mut first_byte)
            .await
            .expect("Rocket response must emit one byte"),
        1
    );
    assert_eq!(&first_byte, b"v");
    probe.assert_open();
    drop(response);

    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn rocket_reader_error_closes_scope_before_restoring_upstream_error() {
    let scope = Arc::new(WebRequestScope::new(CancellationToken::new()));
    let mut response = RocketResponse::build()
        .streamed_body(FailingTokioReader::new())
        .finalize();
    let body = std::mem::take(response.body_mut());
    let mut reader = RocketScopedReader::new(body, Arc::clone(&scope), CancellationToken::new());
    WebAdapterContract::assert_scope_open(&scope);

    let error = reader
        .read_to_end(&mut Vec::new())
        .await
        .expect_err("synthetic Rocket reader must fail");
    assert_eq!(error.to_string(), FailingTokioReader::error_message());
    WebAdapterContract::assert_scope_closed(&scope);
}

#[rocket::async_test]
async fn rocket_reader_reports_cleanup_timeout_while_background_close_continues() {
    let fixture = ScopeCleanupTimeoutFixture::new(Duration::from_millis(10)).await;
    let mut response = RocketResponse::build()
        .streamed_body(FailingTokioReader::new())
        .finalize();
    let body = std::mem::take(response.body_mut());
    let mut reader = RocketScopedReader::new(
        body,
        fixture.scope(),
        fixture.scope().cancellation().clone(),
    );

    let error = reader
        .read_to_end(&mut Vec::new())
        .await
        .expect_err("Rocket reader must report the scope cleanup timeout");
    assert!(
        error.to_string().contains("cleanup exceeded timeout"),
        "unexpected Rocket cleanup error: {error}"
    );
    fixture.assert_cleanup_timed_out();
    fixture.assert_redacted_warning().await;
    fixture.release();
    fixture.assert_closed_within(Duration::from_secs(1)).await;
}

#[rocket::async_test]
async fn strict_aop_routes_use_route_template_and_owned_snapshot() {
    let context = ready_aop_context([Operation::new("/api/orders/<id>", "GET")], None).await;
    let rocket = rocket::build()
        .attach(VernalRocketFairing::new(context))
        .mount("/api", routes![strict_probe].with_vernal_aop());
    let client = Client::tracked(rocket).await.expect("Rocket client");

    let response = client.get("/api/orders/42").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().await.as_deref(), Some("woven"));
}

#[rocket::async_test]
async fn strict_aop_maps_policy_failure_without_calling_handler() {
    PROTECTED_CALLED.store(false, Ordering::SeqCst);
    let probe = Arc::new(ScopeCloseProbe::new());
    let context = ready_aop_context(
        [Operation::new("/protected", "GET")],
        Some(Advisor::new(
            |_: &Operation| true,
            ScopeRejectingInterceptor::new(Arc::clone(&probe)),
            -1000,
        )),
    )
    .await;
    let rocket = rocket::build()
        .attach(VernalRocketFairing::new(context))
        .mount("/", routes![protected].with_vernal_aop());
    let client = Client::tracked(rocket).await.expect("Rocket client");

    let response = client.get("/protected").dispatch().await;
    assert_eq!(response.status(), Status::Unauthorized);
    assert_eq!(
        response.into_string().await.as_deref(),
        Some("Authentication is required")
    );
    assert!(!PROTECTED_CALLED.load(Ordering::SeqCst));
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[rocket::async_test]
async fn strict_aop_maps_authenticated_forbidden_to_403() {
    PROTECTED_CALLED.store(false, Ordering::SeqCst);
    let context = ready_aop_context(
        [Operation::new("/protected", "GET")],
        Some(Advisor::new(
            |_: &Operation| true,
            SecurityContractInterceptor::forbidden("operator-7", ["operator"]),
            -1000,
        )),
    )
    .await;
    let rocket = rocket::build()
        .attach(VernalRocketFairing::new(context))
        .mount("/", routes![protected].with_vernal_aop());
    let client = Client::tracked(rocket).await.expect("Rocket client");

    let response = client.get("/protected").dispatch().await;
    assert_eq!(response.status(), Status::Forbidden);
    assert_eq!(
        response.into_string().await.as_deref(),
        Some("Access is forbidden")
    );
    assert!(!PROTECTED_CALLED.load(Ordering::SeqCst));
}

#[rocket::async_test]
async fn strict_aop_fails_closed_when_plan_is_missing() {
    PROTECTED_CALLED.store(false, Ordering::SeqCst);
    let context = ready_aop_context([Operation::new("/different", "GET")], None).await;
    let rocket = rocket::build()
        .attach(VernalRocketFairing::new(context))
        .mount("/", routes![missing_plan].with_vernal_aop());
    let client = Client::tracked(rocket).await.expect("Rocket client");

    let response = client.get("/missing-plan").dispatch().await;
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(
        response.into_string().await.as_deref(),
        Some("Vernal AOP invocation failed")
    );
    assert!(!PROTECTED_CALLED.load(Ordering::SeqCst));
}

#[rocket::async_test]
async fn strict_aop_preserves_native_forward_outcome() {
    let context = ready_aop_context([Operation::new("/forward", "GET")], None).await;
    let rocket = rocket::build()
        .attach(VernalRocketFairing::new(context))
        .mount("/", routes![forwarding, forward_fallback].with_vernal_aop());
    let client = Client::tracked(rocket).await.expect("Rocket client");

    let response = client.get("/forward").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().await.as_deref(), Some("forwarded"));
}

#[rocket::async_test]
async fn strict_aop_preserves_native_error_outcome() {
    let context = ready_aop_context([Operation::new("/guard-error", "GET")], None).await;
    let rocket = rocket::build()
        .attach(VernalRocketFairing::new(context))
        .mount("/", routes![guard_error].with_vernal_aop());
    let client = Client::tracked(rocket).await.expect("Rocket client");

    let response = client.get("/guard-error").dispatch().await;
    assert_eq!(response.status(), Status::ImATeapot);
}
