//! Tonic Context、组件、Scope、方法元数据与 Status 映射合同测试。

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use http::{Request as HttpRequest, Response};
use http_body_util::BodyExt;
use tonic::{
    Code, GrpcMethod, Request,
    body::empty_body,
    service::{Interceptor, interceptor},
};
use tower::{Layer, ServiceExt, service_fn};
use vernal_aop::{Advisor, Operation};
use vernal_context::{ApplicationContextBuilder, VernalApplicationBuilder};
use vernal_http::{HttpBody, HttpRequestSnapshot};
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_tonic::{
    RequestScopeLayer, TonicAopLayer, TonicContextInterceptor, TonicRequestExt, TonicStatusMapper,
    VernalLayer,
};
use vernal_web::{ProblemDetails, ProblemKind, RequestContext, WebRequestScope};
use vernal_web_testkit::{
    FailingHttpBody, ScopeCleanupTimeoutFixture, ScopeCloseProbe, ScopeRejectingInterceptor,
    WebAdapterContract,
};

struct Greeting(&'static str);

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<Greeting, WebRequestScope, _>(
            |_| Greeting("vernal-grpc"),
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
async fn interceptor_exposes_context_but_requires_scope_for_component_resolution() {
    let context = ready_context().await;
    let mut interceptor = TonicContextInterceptor::new(Arc::clone(&context));
    let request = interceptor
        .call(Request::new(()))
        .expect("interceptor should accept request");

    assert!(Arc::ptr_eq(
        &request.vernal_context().expect("context extension"),
        &context
    ));
    assert!(matches!(
        request.vernal_component::<Greeting>(),
        Err(vernal_tonic::TonicRequestError::MissingRequestScope)
    ));
}

#[tokio::test]
async fn request_reads_scoped_component_and_grpc_method_from_native_extensions() {
    let context = ready_context().await;
    let scope = Arc::new(WebRequestScope::from_application_context(Arc::clone(
        &context,
    )));
    let mut request = Request::new("stream-message");
    request.extensions_mut().insert(Arc::clone(&context));
    request.extensions_mut().insert(Arc::clone(&scope));
    request
        .extensions_mut()
        .insert(GrpcMethod::new("greeter.Greeter", "SayHello"));

    assert!(Arc::ptr_eq(
        &request.vernal_request_scope().expect("request scope"),
        &scope
    ));
    let component = request.vernal_component::<Greeting>().expect("component");
    WebAdapterContract::assert_request_binding(
        &context,
        &request.vernal_context().expect("context"),
        &scope,
        &component,
    );
    assert_eq!(component.0, "vernal-grpc");
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
    let probe = Arc::new(ScopeCloseProbe::new());
    let service_probe = Arc::clone(&probe);

    let inner = service_fn(move |request: HttpRequest<HttpBody>| {
        let expected_context = Arc::clone(&expected_context);
        let service_probe = Arc::clone(&service_probe);
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
            service_probe.observe(&scope);
            Ok::<_, std::convert::Infallible>(Response::new(HttpBody::full("grpc")))
        }
    });
    let grpc_interceptor = TonicContextInterceptor::new(Arc::clone(&context));
    let service = interceptor(grpc_interceptor).layer(inner);
    let service = RequestScopeLayer::new(Arc::clone(&context)).layer(service);
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
    probe.assert_open();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "grpc");
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn partial_response_consumption_then_disconnect_closes_request_scope() {
    let context = ready_context().await;
    let probe = Arc::new(ScopeCloseProbe::new());
    let service_probe = Arc::clone(&probe);
    let inner = service_fn(move |request: HttpRequest<HttpBody>| {
        let service_probe = Arc::clone(&service_probe);
        async move {
            let scope = request
                .extensions()
                .get::<Arc<WebRequestScope>>()
                .expect("request scope")
                .clone();
            service_probe.observe(&scope);
            Ok::<_, std::convert::Infallible>(Response::new(HttpBody::full(
                "stream is not consumed",
            )))
        }
    });
    let service = RequestScopeLayer::new(context).layer(inner);

    let response = service
        .oneshot(
            HttpRequest::builder()
                .uri("/greeter.Greeter/StreamHello")
                .body(HttpBody::empty())
                .expect("request"),
        )
        .await
        .expect("service response");
    let mut body = response.into_body();
    let data = body
        .frame()
        .await
        .expect("Tonic response must emit one data frame")
        .expect("Tonic data frame must succeed")
        .into_data()
        .expect("first Tonic frame must contain data");
    assert_eq!(data, "stream is not consumed");
    probe.assert_open();
    drop(body);

    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn tonic_layer_body_error_closes_scope_before_restoring_upstream_error() {
    let context = ready_context().await;
    let probe = Arc::new(ScopeCloseProbe::new());
    let service_probe = Arc::clone(&probe);
    let inner = service_fn(move |request: HttpRequest<HttpBody>| {
        let service_probe = Arc::clone(&service_probe);
        async move {
            let scope = request
                .extensions()
                .get::<Arc<WebRequestScope>>()
                .expect("request scope")
                .clone();
            service_probe.observe(&scope);
            Ok::<_, std::convert::Infallible>(Response::new(FailingHttpBody::new()))
        }
    });
    let service = RequestScopeLayer::new(context).layer(inner);

    let response = service
        .oneshot(
            HttpRequest::builder()
                .uri("/greeter.Greeter/StreamFailure")
                .body(HttpBody::empty())
                .expect("request"),
        )
        .await
        .expect("service response");
    probe.assert_open();
    let error = response
        .into_body()
        .collect()
        .await
        .expect_err("synthetic Tonic body must fail");
    assert!(matches!(
        error,
        vernal_tower::TowerBodyError::Upstream(ref source)
            if source.to_string() == FailingHttpBody::error_message()
    ));
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn tonic_body_reports_cleanup_timeout_while_background_close_continues() {
    let fixture = ScopeCleanupTimeoutFixture::new(Duration::from_millis(10)).await;
    let scoped = vernal_tower::ScopedBody::new(
        FailingHttpBody::new(),
        fixture.scope(),
        fixture.scope().cancellation().clone(),
    );

    let error = scoped
        .collect()
        .await
        .expect_err("Tonic body must report the scope cleanup timeout");
    assert!(
        error.to_string().contains("cleanup exceeded timeout"),
        "unexpected Tonic cleanup error: {error}"
    );
    fixture.assert_cleanup_timed_out();
    fixture.assert_redacted_warning().await;
    fixture.release();
    fixture.assert_closed_within(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn tonic_aop_layer_resolves_grpc_uri_and_propagates_owned_snapshot() {
    let context = ready_aop_context(Operation::new("greeter.Greeter", "SayHello"), None).await;
    let inner = service_fn(|request: HttpRequest<HttpBody>| async move {
        let context = request
            .extensions()
            .get::<Arc<RequestContext>>()
            .expect("request context")
            .clone();
        let snapshot = context
            .extensions()
            .get::<HttpRequestSnapshot>()
            .await
            .expect("HTTP snapshot");
        assert_eq!(context.route().handler(), "greeter.Greeter");
        assert_eq!(context.route().operation_name(), "SayHello");
        assert_eq!(context.route().path_template(), "/greeter.Greeter/SayHello");
        assert_eq!(snapshot.uri().path(), "/greeter.Greeter/SayHello");
        Ok::<_, std::convert::Infallible>(Response::new(empty_body()))
    });
    let service = TonicAopLayer::new().layer(inner);
    let service = RequestScopeLayer::new(Arc::clone(&context)).layer(service);
    let service = VernalLayer::new(context).layer(service);

    let response = service
        .oneshot(
            HttpRequest::builder()
                .uri("/greeter.Greeter/SayHello")
                .body(HttpBody::empty())
                .expect("request"),
        )
        .await
        .expect("gRPC AOP response");
    assert_eq!(response.status(), http::StatusCode::OK);
    response
        .into_body()
        .collect()
        .await
        .expect("empty gRPC response body");
}

#[tokio::test]
async fn tonic_aop_layer_maps_policy_failure_to_grpc_status_without_calling_handler() {
    let called = Arc::new(AtomicBool::new(false));
    let probe = Arc::new(ScopeCloseProbe::new());
    let context = ready_aop_context(
        Operation::new("greeter.Greeter", "Protected"),
        Some(Advisor::new(
            |_: &Operation| true,
            ScopeRejectingInterceptor::new(Arc::clone(&probe)),
            -1000,
        )),
    )
    .await;
    let handler_called = Arc::clone(&called);
    let inner = service_fn(move |_request: HttpRequest<HttpBody>| {
        handler_called.store(true, Ordering::SeqCst);
        async { Ok::<_, std::convert::Infallible>(Response::new(empty_body())) }
    });
    let service = TonicAopLayer::new().layer(inner);
    let service = RequestScopeLayer::new(Arc::clone(&context)).layer(service);
    let service = VernalLayer::new(context).layer(service);

    let response = service
        .oneshot(
            HttpRequest::builder()
                .uri("/greeter.Greeter/Protected")
                .body(HttpBody::empty())
                .expect("request"),
        )
        .await
        .expect("gRPC status response");
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.headers()["grpc-status"], "16");
    assert!(!called.load(Ordering::SeqCst));
    response
        .into_body()
        .collect()
        .await
        .expect("empty gRPC error body");
    probe.assert_closed_within(Duration::from_secs(1)).await;
}
