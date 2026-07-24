//! Salvo Handler、Depot、IoC 与请求作用域集成测试。

use std::{sync::Arc, time::Duration};

use http_body_util::BodyExt;
use salvo::{
    Depot, FlowCtrl, Handler, Request, Response,
    http::{HeaderMap, HeaderValue, ResBody, StatusCode},
};
use tokio::{sync::Notify, time::timeout};
use vernal_context::ApplicationContextBuilder;
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
