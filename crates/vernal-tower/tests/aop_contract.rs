//! Vernal Tower AOP 顺序、短路、类型保持、错误边界与取消合同测试。

#[path = "aop_support/cancelling_interceptor.rs"]
mod cancelling_interceptor;
#[path = "aop_support/recording_interceptor.rs"]
mod recording_interceptor;
#[path = "aop_support/rejecting_interceptor.rs"]
mod rejecting_interceptor;
#[path = "aop_support/synthetic_response_interceptor.rs"]
mod synthetic_response_interceptor;

use std::{
    cell::Cell,
    convert::Infallible,
    future::pending,
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use cancelling_interceptor::CancellingInterceptor;
use http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use recording_interceptor::RecordingInterceptor;
use rejecting_interceptor::RejectingInterceptor;
use synthetic_response_interceptor::SyntheticResponseInterceptor;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tower::{Layer, ServiceExt, service_fn};
use vernal_aop::{Advisor, InvocationError, Operation};
use vernal_context::{ApplicationContext, VernalApplicationBuilder};
use vernal_http::HttpBody;
use vernal_tower::{
    AopLayer, AopServiceError, ContextPropagationLayer, MissingPlanPolicy, RequestScopeLayer,
    VernalLayer,
};
use vernal_web::{RequestContext, RouteMetadata, WebRequestScope};

async fn ready_context(
    operation: Option<Operation>,
    advisors: Vec<Advisor>,
) -> Arc<ApplicationContext> {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    for advisor in advisors {
        builder.advisor(advisor);
    }
    if let Some(operation) = operation {
        builder.operation(operation);
    }
    let context = Arc::new(builder.build().expect("valid AOP application"));
    context.refresh().await.expect("context refresh");
    context.start().await.expect("context start");
    context
}

fn tower_request<B>(body: B, context: Arc<ApplicationContext>, route: RouteMetadata) -> Request<B> {
    let scope = Arc::new(WebRequestScope::from_application_context(Arc::clone(
        &context,
    )));
    let mut request = Request::new(body);
    request.extensions_mut().insert(context);
    request.extensions_mut().insert(scope);
    request.extensions_mut().insert(route);
    request
}

fn operation() -> Operation {
    Operation::new("OrderHandler", "create")
}

fn route() -> RouteMetadata {
    RouteMetadata::new("OrderHandler", "create", "/orders")
}

#[tokio::test]
async fn recommended_layer_order_builds_context_scope_and_aop_invocation() {
    let context = ready_context(Some(operation()), Vec::new()).await;
    let handler = service_fn(|request: Request<()>| async move {
        assert!(
            request
                .extensions()
                .get::<Arc<ApplicationContext>>()
                .is_some()
        );
        assert!(request.extensions().get::<Arc<WebRequestScope>>().is_some());
        assert!(request.extensions().get::<Arc<RequestContext>>().is_some());
        Ok::<_, Infallible>(Response::new(HttpBody::full("woven")))
    });
    let aop = AopLayer::from_extension().layer(handler);
    let propagated = ContextPropagationLayer::from_extension().layer(aop);
    let scoped = RequestScopeLayer::new(Arc::clone(&context)).layer(propagated);
    let service = VernalLayer::new(context).layer(scoped);
    let mut request = Request::new(());
    request.extensions_mut().insert(route());

    let response = service.oneshot(request).await.expect("layered response");
    let body = response
        .into_body()
        .collect()
        .await
        .expect("scoped response body")
        .to_bytes();
    assert_eq!(body, "woven");
}

#[tokio::test]
async fn ordered_chain_preserves_non_sync_native_response_and_request_context() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let first_saw_context = Arc::new(AtomicBool::new(false));
    let second_saw_context = Arc::new(AtomicBool::new(false));
    let context = ready_context(
        Some(operation()),
        vec![
            Advisor::new(
                |_: &Operation| true,
                RecordingInterceptor::new(
                    "first",
                    Arc::clone(&events),
                    Arc::clone(&first_saw_context),
                ),
                10,
            ),
            Advisor::new(
                |_: &Operation| true,
                RecordingInterceptor::new(
                    "second",
                    Arc::clone(&events),
                    Arc::clone(&second_saw_context),
                ),
                20,
            ),
        ],
    )
    .await;
    let handler_events = Arc::clone(&events);
    let service = service_fn(move |request: Request<()>| {
        let handler_events = Arc::clone(&handler_events);
        async move {
            assert!(request.extensions().get::<Arc<RequestContext>>().is_some());
            handler_events.lock().await.push(String::from("handler"));
            Ok::<_, Infallible>(
                Response::builder()
                    .status(StatusCode::CREATED)
                    .header("x-vernal", "preserved")
                    .body(Cell::new(7_u8))
                    .expect("native response"),
            )
        }
    });

    let response = AopLayer::from_extension()
        .layer(service)
        .oneshot(tower_request((), context, route()))
        .await
        .expect("AOP response");

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()["x-vernal"], "preserved");
    assert_eq!(response.body().get(), 7);
    assert!(first_saw_context.load(Ordering::SeqCst));
    assert!(second_saw_context.load(Ordering::SeqCst));
    assert_eq!(
        *events.lock().await,
        [
            "first:before",
            "second:before",
            "handler",
            "second:after",
            "first:after"
        ]
    );
}

#[tokio::test]
async fn interceptor_can_short_circuit_with_native_response_without_calling_handler() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_context(
        Some(operation()),
        vec![Advisor::new(
            |_: &Operation| true,
            SyntheticResponseInterceptor::new(41),
            0,
        )],
    )
    .await;
    let handler_called = Arc::clone(&called);
    let service = service_fn(move |_request: Request<()>| {
        handler_called.store(true, Ordering::SeqCst);
        async { Ok::<_, Infallible>(Response::new(Cell::new(1_u8))) }
    });

    let response = AopLayer::from_extension()
        .layer(service)
        .oneshot(tower_request((), context, route()))
        .await
        .expect("short-circuit response");

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    assert_eq!(response.headers()["x-vernal-short-circuit"], "true");
    assert_eq!(response.body().get(), 41);
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn upstream_error_is_restored_but_same_typed_interceptor_error_stays_invocation_error() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let saw_context = Arc::new(AtomicBool::new(false));
    let upstream_context = ready_context(
        Some(operation()),
        vec![Advisor::new(
            |_: &Operation| true,
            RecordingInterceptor::new("observer", Arc::clone(&events), saw_context),
            0,
        )],
    )
    .await;
    let upstream = service_fn(|_request: Request<()>| async {
        Err::<Response<Cell<u8>>, _>(io::Error::other("handler failed"))
    });
    let upstream_error = AopLayer::from_extension()
        .layer(upstream)
        .oneshot(tower_request((), upstream_context, route()))
        .await
        .expect_err("upstream must fail");
    assert!(matches!(
        upstream_error,
        AopServiceError::Upstream(ref source) if source.to_string() == "handler failed"
    ));
    assert_eq!(*events.lock().await, ["observer:before", "observer:after"]);

    let rejected_context = ready_context(
        Some(operation()),
        vec![Advisor::new(|_: &Operation| true, RejectingInterceptor, 0)],
    )
    .await;
    let rejected = service_fn(|_request: Request<()>| async {
        Ok::<_, io::Error>(Response::new(Cell::new(1_u8)))
    });
    let rejected_error = AopLayer::from_extension()
        .layer(rejected)
        .oneshot(tower_request((), rejected_context, route()))
        .await
        .expect_err("interceptor must reject");
    assert!(matches!(
        rejected_error,
        AopServiceError::Invocation(InvocationError::Target { .. })
    ));
}

#[tokio::test]
async fn request_scope_cancellation_terminates_the_whole_aop_chain() {
    let context = ready_context(
        Some(operation()),
        vec![Advisor::new(|_: &Operation| true, CancellingInterceptor, 0)],
    )
    .await;
    let service = service_fn(|_request: Request<()>| async {
        pending::<Result<Response<Cell<u8>>, io::Error>>().await
    });

    let error = AopLayer::from_extension()
        .layer(service)
        .oneshot(tower_request((), context, route()))
        .await
        .expect_err("cancelled invocation must fail");

    assert!(matches!(
        error,
        AopServiceError::Invocation(InvocationError::Cancelled)
    ));
}

#[tokio::test]
async fn missing_plan_is_fail_closed_unless_proceed_is_explicit() {
    let strict_called = Arc::new(AtomicBool::new(false));
    let context = ready_context(None, Vec::new()).await;
    let handler_called = Arc::clone(&strict_called);
    let strict_service = service_fn(move |_request: Request<()>| {
        handler_called.store(true, Ordering::SeqCst);
        async { Ok::<_, Infallible>(Response::new(Cell::new(1_u8))) }
    });
    let strict_error = AopLayer::from_extension()
        .layer(strict_service)
        .oneshot(tower_request((), Arc::clone(&context), route()))
        .await
        .expect_err("strict layer must reject missing plan");
    assert!(matches!(
        strict_error,
        AopServiceError::Invocation(InvocationError::PlanNotFound { .. })
    ));
    assert!(!strict_called.load(Ordering::SeqCst));

    let optional_service = service_fn(|_request: Request<()>| async {
        Ok::<_, Infallible>(Response::new(Cell::new(9_u8)))
    });
    let response = AopLayer::from_extension()
        .with_missing_plan_policy(MissingPlanPolicy::Proceed)
        .layer(optional_service)
        .oneshot(tower_request((), context, route()))
        .await
        .expect("explicit proceed policy");
    assert_eq!(response.body().get(), 9);
}

#[tokio::test]
async fn missing_layer_prerequisites_return_distinct_structured_errors() {
    let context = ready_context(Some(operation()), Vec::new()).await;
    let handler = || {
        service_fn(|_request: Request<()>| async {
            Ok::<_, Infallible>(Response::new(Cell::new(1_u8)))
        })
    };

    let missing_context = AopLayer::from_extension()
        .layer(handler())
        .oneshot(Request::new(()))
        .await
        .expect_err("application context is required");
    assert!(matches!(
        missing_context,
        AopServiceError::MissingApplicationContext
    ));

    let mut missing_scope_request = Request::new(());
    missing_scope_request
        .extensions_mut()
        .insert(Arc::clone(&context));
    let missing_scope = AopLayer::from_extension()
        .layer(handler())
        .oneshot(missing_scope_request)
        .await
        .expect_err("request scope is required");
    assert!(matches!(
        missing_scope,
        AopServiceError::MissingRequestScope
    ));

    let mut missing_route_request = Request::new(());
    missing_route_request.extensions_mut().insert(context);
    missing_route_request
        .extensions_mut()
        .insert(Arc::new(WebRequestScope::new(CancellationToken::new())));
    let missing_route = AopLayer::from_extension()
        .layer(handler())
        .oneshot(missing_route_request)
        .await
        .expect_err("route metadata is required");
    assert!(matches!(
        missing_route,
        AopServiceError::MissingRouteMetadata
    ));
}
