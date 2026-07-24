//! Rocket Fairing、Managed State、Request Guard 与请求作用域集成测试。

use std::{sync::Arc, time::Duration};

use rocket::{get, http::Status, local::asynchronous::Client, routes};
use tokio::{sync::Notify, time::timeout};
use vernal_context::ApplicationContextBuilder;
use vernal_ioc::{ComponentDefinition, RegistryBuilder};
use vernal_rocket::{
    VernalRocketComponent, VernalRocketContext, VernalRocketFairing, VernalRocketRequestScope,
};

struct Greeting(&'static str);

struct ScopeClosed(Arc<Notify>);

#[get("/hello")]
async fn hello(
    context: VernalRocketContext,
    greeting: VernalRocketComponent<Greeting>,
    scope: VernalRocketRequestScope,
) -> &'static str {
    assert!(context.0.container().resolve::<Greeting>().is_ok());
    let closed = context
        .0
        .container()
        .resolve::<ScopeClosed>()
        .expect("scope notifier");
    scope
        .0
        .on_close(move || async move {
            closed.0.notify_one();
            Ok::<_, std::io::Error>(())
        })
        .await
        .expect("scope close hook");
    greeting.0.0
}

async fn ready_context(closed: Arc<Notify>) -> Arc<vernal_context::ApplicationContext> {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::singleton::<Greeting, _>(|_| {
            Greeting("vernal-rocket")
        }))
        .expect("greeting registration");
    registry
        .register(ComponentDefinition::singleton::<ScopeClosed, _>(
            move |_| ScopeClosed(Arc::clone(&closed)),
        ))
        .expect("notifier registration");
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

#[rocket::async_test]
async fn fairing_exposes_context_component_and_scope_until_body_finishes() {
    let closed = Arc::new(Notify::new());
    let context = ready_context(Arc::clone(&closed)).await;
    let rocket = rocket::build()
        .attach(VernalRocketFairing::new(context))
        .mount("/", routes![hello]);
    let client = Client::tracked(rocket).await.expect("Rocket client");

    let response = client.get("/hello").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_string().await.expect("response body"),
        "vernal-rocket"
    );
    timeout(Duration::from_secs(1), closed.notified())
        .await
        .expect("scope close");
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
async fn dropping_rocket_response_body_closes_request_scope() {
    let closed = Arc::new(Notify::new());
    let context = ready_context(Arc::clone(&closed)).await;
    let rocket = rocket::build()
        .attach(VernalRocketFairing::new(context))
        .mount("/", routes![hello]);
    let client = Client::tracked(rocket).await.expect("Rocket client");

    let response = client.get("/hello").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    drop(response);

    timeout(Duration::from_secs(1), closed.notified())
        .await
        .expect("scope must close after Rocket response cancellation");
}
