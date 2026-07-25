//! `IoC` 托管强类型事件监听器的启动、失败与作用域合同测试。

mod event_listener_support;

use std::{error::Error, sync::Arc, time::Duration};

use event_listener_support::{
    FailingEventListener, InventoryEventModule, InventoryReserved, InventoryReservedListener,
    InventoryReservingLifecycle,
};
use tokio::sync::Mutex;
use vernal_context::{
    ApplicationBuildError, ConditionalComponentModule, ContextError, ProfileCondition,
    VernalApplicationBuilder,
};
use vernal_beans::ComponentDefinition;

/// 创建使用当前 Tokio Runtime 的测试应用建造器。
fn application() -> VernalApplicationBuilder {
    VernalApplicationBuilder::new(tokio::runtime::Handle::current())
}

/// 等待异步监听任务把指定序号写入探针。
async fn wait_for_sequence(sequences: &Arc<Mutex<Vec<u64>>>, expected: u64) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if sequences.lock().await.contains(&expected) {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("managed listener should receive event");
}

#[tokio::test]
async fn application_module_atomically_contributes_listener_component_and_subscription() {
    let sequences = Arc::new(Mutex::new(Vec::new()));
    let mut application = application();
    application
        .register_module(InventoryEventModule::new(Arc::clone(&sequences)))
        .expect("event module registration");
    let context = application.build().expect("event module context");
    context
        .refresh()
        .await
        .expect("module listener subscription");

    assert_eq!(
        context
            .events()
            .publish(InventoryReserved { sequence: 83 })
            .await,
        1
    );
    wait_for_sequence(&sequences, 83).await;
    context.close().await.expect("module listener shutdown");
}

#[tokio::test]
async fn refresh_subscribes_ioc_listener_before_lifecycle_initialize_publishes() {
    let sequences = Arc::new(Mutex::new(Vec::new()));
    let mut application = application();
    application
        .register(ComponentDefinition::shared_arc(Arc::new(
            InventoryReservedListener::new(Arc::clone(&sequences)),
        )))
        .expect("listener definition");
    application
        .register(
            ComponentDefinition::singleton::<InventoryReservingLifecycle, _>(|resolver| {
                InventoryReservingLifecycle::new(
                    resolver
                        .resolve()
                        .expect("EventBus is a validated dependency"),
                )
            })
            .depends_on::<vernal_context::EventBus>(),
        )
        .expect("publishing lifecycle definition");
    application
        .event_listener::<InventoryReserved, InventoryReservedListener>()
        .lifecycle::<InventoryReservingLifecycle>();

    let context = application.build().expect("managed listener context");
    context.refresh().await.expect("refresh and publish");
    wait_for_sequence(&sequences, 41).await;

    assert_eq!(*sequences.lock().await, [41]);
    assert_eq!(
        context
            .managed_tasks()
            .expect("managed task supervisor")
            .active_count(),
        1
    );
    context.close().await.expect("listener should stop cleanly");
    assert_eq!(
        context
            .managed_tasks()
            .expect("managed task supervisor")
            .active_count(),
        0
    );
}

#[tokio::test]
async fn listener_failure_cancels_application_and_redacts_default_error_text() {
    let mut application = application();
    application
        .register(ComponentDefinition::shared_value(FailingEventListener))
        .expect("failing listener definition");
    application.event_listener::<InventoryReserved, FailingEventListener>();
    let context = application.build().expect("failing listener context");
    context.refresh().await.expect("listener subscription");

    assert_eq!(
        context
            .events()
            .publish(InventoryReserved { sequence: 7 })
            .await,
        1
    );
    tokio::time::timeout(
        Duration::from_secs(2),
        context.cancellation_token().cancelled(),
    )
    .await
    .expect("listener failure should cancel application");

    let error = context
        .close()
        .await
        .expect_err("task failure must surface");
    assert!(matches!(error, ContextError::ManagedTask { .. }));
    assert!(
        !error
            .to_string()
            .contains("secret-listener-storage-response")
    );
    assert!(!format!("{error:?}").contains("secret-listener-storage-response"));
    assert!(
        error
            .source()
            .and_then(Error::source)
            .and_then(Error::source)
            .expect("explicit listener source chain")
            .to_string()
            .contains("secret-listener-storage-response")
    );
}

#[tokio::test]
async fn application_rejects_non_singleton_listener_without_promoting_its_scope() {
    let mut application = application();
    application
        .register(
            ComponentDefinition::transient::<InventoryReservedListener, _>(|_| {
                InventoryReservedListener::new(Arc::new(Mutex::new(Vec::new())))
            }),
        )
        .expect("transient listener definition");
    application.event_listener::<InventoryReserved, InventoryReservedListener>();

    let Err(error) = application.build() else {
        panic!("transient event listener must be rejected");
    };
    assert!(matches!(
        error,
        ApplicationBuildError::Context {
            source: ContextError::EventListenerScope {
                scope: "transient",
                ..
            }
        }
    ));
}

#[tokio::test]
async fn application_rejects_duplicate_component_event_subscription() {
    let mut application = application();
    application
        .register(ComponentDefinition::shared_value(FailingEventListener))
        .expect("listener definition");
    application
        .event_listener::<InventoryReserved, FailingEventListener>()
        .event_listener::<InventoryReserved, FailingEventListener>();

    let Err(error) = application.build() else {
        panic!("duplicate listener declaration must be rejected");
    };
    assert!(matches!(
        error,
        ApplicationBuildError::Context {
            source: ContextError::DuplicateEventListener { .. }
        }
    ));
}

#[tokio::test]
async fn unmatched_condition_omits_listener_component_and_subscription_atomically() {
    let sequences = Arc::new(Mutex::new(Vec::new()));
    let mut conditional = ConditionalComponentModule::new(
        "test.conditional-inventory-events",
        ProfileCondition::any(["inventory-events"]).expect("valid profile condition"),
    );
    conditional
        .register(ComponentDefinition::shared_arc(Arc::new(
            InventoryReservedListener::new(Arc::clone(&sequences)),
        )))
        .event_listener::<InventoryReserved, InventoryReservedListener>();

    let mut application = application();
    application
        .register_conditional(conditional)
        .expect("conditional listener module");
    let context = application.build().expect("unmatched listener context");
    context.refresh().await.expect("context refresh");

    assert_eq!(
        context
            .events()
            .publish(InventoryReserved { sequence: 101 })
            .await,
        0
    );
    assert!(sequences.lock().await.is_empty());
    let report = context.startup_report().await;
    assert!(!report.condition_evaluations()[0].matched());
    assert_eq!(report.condition_evaluations()[0].event_listener_count(), 1);
    context.close().await.expect("empty listener shutdown");
}
