//! Salvo Handler、Depot、IoC 与请求作用域集成测试。

#[path = "aop_support/policy_deny_interceptor.rs"]
mod policy_deny_interceptor;
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
use policy_deny_interceptor::PolicyDenyInterceptor;
use salvo::{
    Depot, FlowCtrl, Handler, Request, Response, Service,
    conn::SocketAddr,
    http::{HeaderMap, HeaderValue, Method, ResBody, StatusCode, uri::Scheme},
    routing::Router,
};
use strict_probe_handler::StrictProbeHandler;
use tokio::{sync::Notify, time::timeout};
use vernal_aop::{Advisor, Operation};
use vernal_context::{ApplicationContextBuilder, VernalApplicationBuilder};
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_salvo::{VernalSalvoDepotExt, VernalSalvoHoop};

struct Greeting(&'static str);

struct ProbeHandler {
    expected: Option<Arc<vernal_context::ApplicationContext>>,
    closed: Option<Arc<Notify>>,
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
            (Arc<Greeting>, Arc<vernal_web::WebRequestScope>),
            vernal_salvo::SalvoRejection,
        > = (|| {
            let context = depot.vernal_context()?;
            let greeting = depot.vernal_component::<Greeting>()?;
            let scope = depot.vernal_request_scope()?;
            if let Some(expected) = &self.expected {
                assert!(Arc::ptr_eq(&context, expected));
            }
            Ok((greeting, scope))
        })();
        match result {
            Ok((greeting, scope)) => {
                if let Some(closed) = &self.closed {
                    let closed = Arc::clone(closed);
                    scope
                        .on_close(move || async move {
                            closed.notify_one();
                            Ok::<_, std::io::Error>(())
                        })
                        .await
                        .expect("scope close hook");
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
        .register(ComponentDefinition::singleton::<Greeting, _>(|_| {
            Greeting("vernal-salvo")
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
    let closed = Arc::new(Notify::new());
    let handlers: Vec<Arc<dyn Handler>> = vec![
        Arc::new(VernalSalvoHoop::new(Arc::clone(&context))),
        Arc::new(ProbeHandler {
            expected: Some(Arc::clone(&context)),
            closed: Some(Arc::clone(&closed)),
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
    let body = response
        .take_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "vernal-salvo");
    timeout(Duration::from_secs(1), closed.notified())
        .await
        .expect("scope close");
}

#[tokio::test]
async fn missing_vernal_hoop_renders_safe_rejection() {
    let mut control = FlowCtrl::new(vec![Arc::new(ProbeHandler {
        expected: None,
        closed: None,
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
    let closed = Arc::new(Notify::new());
    let handlers: Vec<Arc<dyn Handler>> = vec![
        Arc::new(VernalSalvoHoop::new(Arc::clone(&context))),
        Arc::new(ProbeHandler {
            expected: Some(context),
            closed: Some(Arc::clone(&closed)),
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
    timeout(Duration::from_secs(1), closed.notified())
        .await
        .expect("scope close");
}

#[tokio::test]
async fn dropping_salvo_response_body_closes_request_scope() {
    let context = ready_context().await;
    let closed = Arc::new(Notify::new());
    let handlers: Vec<Arc<dyn Handler>> = vec![
        Arc::new(VernalSalvoHoop::new(Arc::clone(&context))),
        Arc::new(ProbeHandler {
            expected: Some(context),
            closed: Some(Arc::clone(&closed)),
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
    drop(response);

    timeout(Duration::from_secs(1), closed.notified())
        .await
        .expect("scope must close after response body cancellation");
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
    let context = ready_aop_context(
        Operation::new("/protected", "GET"),
        Some(Advisor::new(
            |_: &Operation| true,
            PolicyDenyInterceptor,
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
