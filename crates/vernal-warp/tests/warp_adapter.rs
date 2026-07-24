//! Warp Filter、Tower Service、IoC 与请求作用域集成测试。

use std::sync::Arc;

use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use tokio::sync::Notify;
use tower::{Layer, ServiceExt};
use vernal_context::ApplicationContextBuilder;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_warp::{
    VernalWarpComponent, VernalWarpContext, VernalWarpLayer, VernalWarpRequestScope, WarpRejection,
};
use warp::{Filter, Rejection, Reply, http::StatusCode};

struct Greeting(&'static str);

async fn ready_context() -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::singleton::<Greeting, _>(|_| {
            Greeting("vernal-warp")
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
async fn service_exposes_context_component_and_scope_until_body_finishes() {
    let context = ready_context().await;
    let expected = Arc::clone(&context);
    let closed = Arc::new(Notify::new());
    let handler_closed = Arc::clone(&closed);
    let route = VernalWarpContext::filter()
        .and(VernalWarpComponent::<Greeting>::filter())
        .and(VernalWarpRequestScope::filter())
        .and_then(
            move |actual: VernalWarpContext,
                  greeting: VernalWarpComponent<Greeting>,
                  scope: VernalWarpRequestScope| {
                let expected = Arc::clone(&expected);
                let handler_closed = Arc::clone(&handler_closed);
                async move {
                    assert!(Arc::ptr_eq(&actual.0, &expected));
                    scope
                        .0
                        .on_close(move || async move {
                            handler_closed.notify_one();
                            Ok::<_, std::io::Error>(())
                        })
                        .await
                        .expect("scope close hook");
                    Ok::<_, Rejection>(greeting.0)
                }
            },
        )
        .recover(WarpRejection::recover);
    let service = VernalWarpLayer::new(context).layer(warp::service(route));
    let request = warp::http::Request::builder()
        .uri("/hello")
        .body(Empty::<Bytes>::new())
        .expect("request");

    let response = service.oneshot(request).await.expect("service response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "vernal-warp");
    closed.notified().await;
}

#[tokio::test]
async fn missing_vernal_layer_returns_safe_rejection() {
    let route = VernalWarpComponent::<Greeting>::filter()
        .map(|_| "unreachable")
        .recover(WarpRejection::recover);
    let request = warp::http::Request::builder()
        .uri("/missing")
        .body(Empty::<Bytes>::new())
        .expect("request");

    let response = warp::service(route)
        .oneshot(request)
        .await
        .expect("service response");
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response
        .into_response()
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    assert_eq!(body, "Vernal application context is unavailable");
}
