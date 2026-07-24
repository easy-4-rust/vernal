//! Tonic Context、组件、Scope、方法元数据与 Status 映射合同测试。

use std::sync::Arc;

use http::{Request as HttpRequest, Response};
use http_body_util::BodyExt;
use tokio::sync::Notify;
use tonic::{
    Code, GrpcMethod, Request,
    service::{Interceptor, interceptor},
};
use tower::{Layer, ServiceExt, service_fn};
use vernal_context::ApplicationContextBuilder;
use vernal_http::HttpBody;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_tonic::{
    RequestScopeLayer, TonicContextInterceptor, TonicRequestExt, TonicStatusMapper, VernalLayer,
};
use vernal_web::{ProblemDetails, ProblemKind, WebRequestScope};

struct Greeting(&'static str);

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::singleton::<Greeting, _>(|_| {
            Greeting("vernal-grpc")
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
async fn interceptor_exposes_context_and_typed_component() {
    let context = ready_context().await;
    let mut interceptor = TonicContextInterceptor::new(Arc::clone(&context));
    let request = interceptor
        .call(Request::new(()))
        .expect("interceptor should accept request");

    assert!(Arc::ptr_eq(
        &request.vernal_context().expect("context extension"),
        &context
    ));
    assert_eq!(
        request.vernal_component::<Greeting>().expect("component").0,
        "vernal-grpc"
    );
}

#[test]
fn request_reads_scope_and_grpc_method_from_native_extensions() {
    let cancellation = tokio_util::sync::CancellationToken::new();
    let scope = Arc::new(WebRequestScope::new(cancellation));
    let mut request = Request::new("stream-message");
    request.extensions_mut().insert(Arc::clone(&scope));
    request
        .extensions_mut()
        .insert(GrpcMethod::new("greeter.Greeter", "SayHello"));

    assert!(Arc::ptr_eq(
        &request.vernal_request_scope().expect("request scope"),
        &scope
    ));
    let route = request.vernal_route_metadata().expect("route metadata");
    assert_eq!(route.handler(), "greeter.Greeter");
    assert_eq!(route.operation_name(), "SayHello");
    assert_eq!(route.path_template(), "/greeter.Greeter/SayHello");
}

#[test]
fn missing_context_and_problem_mapping_use_stable_statuses() {
    let Err(missing) = Request::new(()).vernal_context() else {
        panic!("context should be missing");
    };
    let missing = tonic::Status::from(missing);
    assert_eq!(missing.code(), Code::Internal);
    assert_eq!(
        missing.message(),
        "Vernal application context is unavailable"
    );

    let problem = ProblemDetails::new(ProblemKind::PolicyDenied, 403, "denied")
        .with_detail("permission denied");
    let status = TonicStatusMapper::from_problem(&problem);
    assert_eq!(status.code(), Code::PermissionDenied);
    assert_eq!(status.message(), "permission denied");
}

#[tokio::test]
async fn tower_layers_preserve_context_and_scope_through_tonic_interceptor() {
    let context = ready_context().await;
    let expected_context = Arc::clone(&context);
    let closed = Arc::new(Notify::new());
    let service_closed = Arc::clone(&closed);

    let inner = service_fn(move |request: HttpRequest<HttpBody>| {
        let expected_context = Arc::clone(&expected_context);
        let service_closed = Arc::clone(&service_closed);
        async move {
            assert!(Arc::ptr_eq(
                request
                    .extensions()
                    .get::<Arc<vernal_context::ApplicationContext>>()
                    .expect("context in downstream service"),
                &expected_context
            ));
            let scope = request
                .extensions()
                .get::<Arc<WebRequestScope>>()
                .expect("scope in downstream service")
                .clone();
            scope
                .on_close(move || async move {
                    service_closed.notify_one();
                    Ok::<_, std::io::Error>(())
                })
                .await
                .expect("close hook");
            Ok::<_, std::convert::Infallible>(Response::new(HttpBody::full("grpc")))
        }
    });
    let grpc_interceptor = TonicContextInterceptor::new(Arc::clone(&context));
    let service = interceptor(grpc_interceptor).layer(inner);
    let service = RequestScopeLayer::new().layer(service);
    let service = VernalLayer::new(Arc::clone(&context)).layer(service);

    let response = service
        .oneshot(
            HttpRequest::builder()
                .uri("/greeter.Greeter/SayHello")
                .body(HttpBody::empty())
                .expect("request"),
        )
        .await
        .expect("service response");
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "grpc");
    closed.notified().await;
}
