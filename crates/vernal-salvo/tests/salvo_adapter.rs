//! Salvo Handler、Depot、IoC 与请求作用域集成测试。

#[path = "aop_support/strict_probe_handler.rs"]
mod strict_probe_handler;

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use http_body_util::BodyExt;
use salvo::{
    Depot, FlowCtrl, Handler, Request, Response, Service,
    conn::SocketAddr,
    http::{HeaderMap, HeaderValue, Method, ResBody, StatusCode, uri::Scheme},
    routing::Router,
};
use strict_probe_handler::StrictProbeHandler;
use tokio_util::sync::CancellationToken;
use vernal_aop::{Advisor, Operation};
use vernal_context::{ApplicationContextBuilder, VernalApplicationBuilder};
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_salvo::SalvoScopedBody;
use vernal_salvo::{VernalSalvoDepotExt, VernalSalvoHoop};
use vernal_web::WebRequestScope;
use vernal_web_testkit::{
    FailingByteStream, ScopeCloseProbe, ScopeRejectingInterceptor, WebAdapterContract,
};

struct Greeting(&'static str);

struct ProbeHandler {
    expected: Option<Arc<vernal_context::ApplicationContext>>,
    probe: Option<Arc<ScopeCloseProbe>>,
    stream_trailers: bool,
}

#[salvo::async_trait]
impl Handler for ProbeHandler {
    async fn handle(
        &self,
        _request: &mut Request,
        depot: &mut Depot,
        response: &mut Response,
        _control: &mut FlowCtrl,
    ) {
        let result: Result<
            (
                Arc<vernal_context::ApplicationContext>,
                Arc<Greeting>,
                Arc<vernal_web::WebRequestScope>,
            ),
            vernal_salvo::SalvoRejection,
        > = (|| {
            let context = depot.vernal_context()?;
            let greeting = depot.vernal_component::<Greeting>()?;
            let scope = depot.vernal_request_scope()?;
            Ok((context, greeting, scope))
        })();
        match result {
            Ok((context, greeting, scope)) => {
                if let Some(expected) = &self.expected {
                    WebAdapterContract::assert_request_binding(
                        expected, &context, &scope, &greeting,
                    );
                }
                if let Some(probe) = &self.probe {
                    probe.observe(&scope);
                }
                if self.stream_trailers {
                    let (mut sender, body) = ResBody::channel();
                    response.body(body);
                    tokio::spawn(async move {
                        sender
                            .send_data(greeting.0)
                            .await
                            .expect("stream response data");
                        let mut trailers = HeaderMap::new();
                        trailers.insert("x-vernal-scope", HeaderValue::from_static("closed"));
                        sender
                            .send_trailers(trailers)
                            .await
                            .expect("stream response trailers");
                    });
                } else {
                    response.render(greeting.0);
                }
            }
            Err(rejection) => response.render(rejection),
        }
    }
}

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<Greeting, WebRequestScope, _>(
            |_| Greeting("vernal-salvo"),
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

async fn response_body(response: &mut Response) -> bytes::Bytes {
    response
        .take_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes()
}

async fn send_get(service: &Service, path: &str) -> Response {
    let mut request = Request::new();
    *request.method_mut() = Method::GET;
    *request.uri_mut() = format!("http://localhost{path}")
        .parse()
        .expect("test request URI");
    service
        .hyper_handler(
            SocketAddr::Unknown,
            SocketAddr::Unknown,
            Scheme::HTTP,
            None,
            None,
        )
        .handle(request)
        .await
}

#[tokio::test]
async fn hoop_exposes_context_component_and_scope_until_body_finishes() {
    let context = ready_context().await;
    let probe = Arc::new(ScopeCloseProbe::new());
    let handlers: Vec<Arc<dyn Handler>> = vec![
        Arc::new(VernalSalvoHoop::new(Arc::clone(&context))),
        Arc::new(ProbeHandler {
            expected: Some(Arc::clone(&context)),
            probe: Some(Arc::clone(&probe)),
            stream_trailers: false,
        }),
    ];
    let mut control = FlowCtrl::new(handlers);
    let mut request = Request::new();
    let mut depot = Depot::new();
    let mut response = Response::new();

    control
        .call_next(&mut request, &mut depot, &mut response)
        .await;
    probe.assert_open();
    let body = response
        .take_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "vernal-salvo");
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn missing_vernal_hoop_renders_safe_rejection() {
    let mut control = FlowCtrl::new(vec![Arc::new(ProbeHandler {
        expected: None,
        probe: None,
        stream_trailers: false,
    })]);
    let mut request = Request::new();
    let mut depot = Depot::new();
    let mut response = Response::new();

    control
        .call_next(&mut request, &mut depot, &mut response)
        .await;
    assert_eq!(
        response.status_code,
        Some(StatusCode::INTERNAL_SERVER_ERROR)
    );
    let body = response
        .take_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "Vernal application context is unavailable");
}

#[tokio::test]
async fn salvo_body_preserves_data_trailers_and_backpressure() {
    let context = ready_context().await;
    let probe = Arc::new(ScopeCloseProbe::new());
    let handlers: Vec<Arc<dyn Handler>> = vec![
        Arc::new(VernalSalvoHoop::new(Arc::clone(&context))),
        Arc::new(ProbeHandler {
            expected: Some(context),
            probe: Some(Arc::clone(&probe)),
            stream_trailers: true,
        }),
    ];
    let mut control = FlowCtrl::new(handlers);
    let mut request = Request::new();
    let mut depot = Depot::new();
    let mut response = Response::new();

    control
        .call_next(&mut request, &mut depot, &mut response)
        .await;
    probe.assert_open();
    let mut body = response.take_body();
    let data = body
        .frame()
        .await
        .expect("data frame")
        .expect("data frame result")
        .into_data()
        .expect("data");
    assert_eq!(data, "vernal-salvo");
    let trailers = body
        .frame()
        .await
        .expect("trailer frame")
        .expect("trailer frame result")
        .into_trailers()
        .expect("trailers");
    assert_eq!(trailers["x-vernal-scope"], "closed");
    assert!(body.frame().await.is_none());
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn salvo_stream_error_closes_scope_before_restoring_upstream_error() {
    let scope = Arc::new(WebRequestScope::new(CancellationToken::new()));
    let body = ResBody::stream(FailingByteStream::new());
    let scoped = SalvoScopedBody::new(body, Arc::clone(&scope), CancellationToken::new());
    WebAdapterContract::assert_scope_open(&scope);

    let error = scoped
        .collect()
        .await
        .expect_err("synthetic Salvo stream must fail");
    assert_eq!(error.to_string(), FailingByteStream::error_message());
    WebAdapterContract::assert_scope_closed(&scope);
}

#[tokio::test]
async fn dropping_salvo_response_body_closes_request_scope() {
    let context = ready_context().await;
    let probe = Arc::new(ScopeCloseProbe::new());
    let handlers: Vec<Arc<dyn Handler>> = vec![
        Arc::new(VernalSalvoHoop::new(Arc::clone(&context))),
        Arc::new(ProbeHandler {
            expected: Some(context),
            probe: Some(Arc::clone(&probe)),
            stream_trailers: false,
        }),
    ];
    let mut control = FlowCtrl::new(handlers);
    let mut request = Request::new();
    let mut depot = Depot::new();
    let mut response = Response::new();

    control
        .call_next(&mut request, &mut depot, &mut response)
        .await;
    probe.assert_open();
    drop(response);

    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn strict_aop_uses_matched_path_and_owned_snapshot() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(Operation::new("/orders/{id}", "GET"), None).await;
    let router = Router::with_path("orders/{id}")
        .hoop(VernalSalvoHoop::strict_aop(Arc::clone(&context)))
        .get(StrictProbeHandler::new(
            Arc::clone(&called),
            true,
            StatusCode::OK,
            "woven",
        ));
    let service = Service::new(router);

    let mut response = send_get(&service, "/orders/42").await;

    assert_eq!(response.status_code, Some(StatusCode::OK));
    assert_eq!(response_body(&mut response).await, "woven");
    assert!(called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn strict_aop_maps_policy_failure_without_calling_handler() {
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
    let router = Router::with_path("protected")
        .hoop(VernalSalvoHoop::strict_aop(Arc::clone(&context)))
        .get(StrictProbeHandler::new(
            Arc::clone(&called),
            false,
            StatusCode::OK,
            "unreachable",
        ));
    let service = Service::new(router);

    let mut response = send_get(&service, "/protected").await;

    assert_eq!(response.status_code, Some(StatusCode::UNAUTHORIZED));
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
    let router = Router::with_path("missing-plan")
        .hoop(VernalSalvoHoop::strict_aop(Arc::clone(&context)))
        .get(StrictProbeHandler::new(
            Arc::clone(&called),
            false,
            StatusCode::OK,
            "unreachable",
        ));
    let service = Service::new(router);

    let mut response = send_get(&service, "/missing-plan").await;

    assert_eq!(
        response.status_code,
        Some(StatusCode::INTERNAL_SERVER_ERROR)
    );
    assert_eq!(
        response_body(&mut response).await,
        "Vernal AOP invocation failed"
    );
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn strict_aop_fails_closed_without_matched_route_metadata() {
    let context = ready_aop_context(Operation::new("/not-found", "GET"), None).await;
    let service =
        Service::new(Router::new()).hoop(VernalSalvoHoop::strict_aop(Arc::clone(&context)));

    let mut response = send_get(&service, "/not-found").await;

    assert_eq!(
        response.status_code,
        Some(StatusCode::INTERNAL_SERVER_ERROR)
    );
    assert_eq!(
        response_body(&mut response).await,
        "Salvo matched route metadata is unavailable"
    );
}

#[tokio::test]
async fn strict_aop_preserves_native_salvo_response() {
    let called = Arc::new(AtomicBool::new(false));
    let context = ready_aop_context(Operation::new("/native-error", "GET"), None).await;
    let router = Router::with_path("native-error")
        .hoop(VernalSalvoHoop::strict_aop(Arc::clone(&context)))
        .get(StrictProbeHandler::new(
            Arc::clone(&called),
            false,
            StatusCode::NOT_FOUND,
            "native-not-found",
        ));
    let service = Service::new(router);

    let mut response = send_get(&service, "/native-error").await;

    assert_eq!(response.status_code, Some(StatusCode::NOT_FOUND));
    assert_eq!(response_body(&mut response).await, "native-not-found");
    assert!(called.load(Ordering::SeqCst));
}
