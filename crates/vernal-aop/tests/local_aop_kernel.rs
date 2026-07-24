//! Vernal Local-AOP 的 `!Send` Around、短路、目录与取消合同测试。

#[path = "local_aop_support/recording_local_interceptor.rs"]
mod recording_local_interceptor;
#[path = "local_aop_support/short_circuit_local_interceptor.rs"]
mod short_circuit_local_interceptor;

use std::{
    cell::RefCell,
    future::pending,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use recording_local_interceptor::RecordingLocalInterceptor;
use short_circuit_local_interceptor::ShortCircuitLocalInterceptor;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use vernal_aop::{
    Invocation, LocalAdvisor, LocalInvocationError, LocalInvocationPlanBuilder,
    LocalInvocationResult, LocalInvocationTarget, LocalInvocationValue, Operation,
};

#[tokio::test]
async fn local_plan_wraps_non_send_target_value_in_stable_order() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut builder = LocalInvocationPlanBuilder::new();
    builder
        .register(LocalAdvisor::new(
            |_: &Operation| true,
            RecordingLocalInterceptor::new("late", Arc::clone(&events)),
            20,
        ))
        .register(LocalAdvisor::new(
            |_: &Operation| true,
            RecordingLocalInterceptor::new("early", Arc::clone(&events)),
            10,
        ));
    let operation = Operation::new("ActixOrderEndpoint", "GET");
    let plan = builder.build(operation.clone());
    let local_events = Rc::new(RefCell::new(Vec::new()));
    let target_events = Rc::clone(&local_events);
    let target: Rc<LocalInvocationTarget> = Rc::new(move |_| {
        let target_events = Rc::clone(&target_events);
        Box::pin(async move {
            target_events.borrow_mut().push("target");
            Ok(Box::new(Rc::new(String::from("local-response"))) as LocalInvocationValue)
        })
    });

    let value = plan
        .invoke(Invocation::new(operation).shared(), target)
        .await
        .expect("local plan");
    let value = value
        .downcast::<Rc<String>>()
        .expect("non-Send Rc return value");
    assert_eq!(value.as_str(), "local-response");
    assert_eq!(*local_events.borrow(), ["target"]);
    assert_eq!(
        *events.lock().await,
        ["early:before", "late:before", "late:after", "early:after"]
    );
}

#[tokio::test]
async fn local_interceptor_can_short_circuit_without_running_target() {
    let called = Arc::new(AtomicBool::new(false));
    let mut builder = LocalInvocationPlanBuilder::new();
    builder.register(LocalAdvisor::new(
        |_: &Operation| true,
        ShortCircuitLocalInterceptor,
        0,
    ));
    let operation = Operation::new("ActixAuthEndpoint", "GET");
    let plan = builder.build(operation.clone());
    let target_called = Arc::clone(&called);
    let target: Rc<LocalInvocationTarget> = Rc::new(move |_| {
        target_called.store(true, Ordering::SeqCst);
        Box::pin(async { Ok(Box::new(Rc::new(String::from("target"))) as LocalInvocationValue) })
    });

    let value = plan
        .invoke(Invocation::new(operation).shared(), target)
        .await
        .expect("local short circuit");
    let value = value.downcast::<Rc<String>>().expect("Rc response");
    assert_eq!(value.as_str(), "blocked");
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn local_plan_catalog_coalesces_operations_and_rejects_mismatch() {
    let declared = Operation::new("ActixCatalogEndpoint", "GET");
    let actual = Operation::new("ActixCatalogEndpoint", "POST");
    let builder = LocalInvocationPlanBuilder::new();
    let catalog = builder.build_catalog([declared.clone(), declared.clone()]);
    assert_eq!(catalog.len(), 1);

    let target: Rc<LocalInvocationTarget> =
        Rc::new(|_| Box::pin(async { Ok(Box::new(()) as LocalInvocationValue) }));
    let error = catalog
        .get(&declared)
        .expect("declared local plan")
        .invoke(Invocation::new(actual.clone()).shared(), target)
        .await
        .expect_err("mismatched operation");
    assert!(matches!(
        error,
        LocalInvocationError::PlanMismatch {
            expected,
            actual: returned
        } if expected == declared && returned == actual
    ));
}

#[tokio::test]
async fn cancellation_stops_pending_non_send_local_target() {
    let operation = Operation::new("ActixJobEndpoint", "POST");
    let plan = LocalInvocationPlanBuilder::new().build(operation.clone());
    let local_marker = Rc::new(RefCell::new(String::from("worker-local")));
    let target_marker = Rc::clone(&local_marker);
    let target: Rc<LocalInvocationTarget> = Rc::new(move |_| {
        let target_marker = Rc::clone(&target_marker);
        Box::pin(async move {
            let result = pending::<LocalInvocationResult>().await;
            assert_eq!(target_marker.borrow().as_str(), "worker-local");
            result
        })
    });
    let cancellation = CancellationToken::new();
    let invocation = Invocation::new(operation)
        .with_cancellation(cancellation.clone())
        .shared();
    cancellation.cancel();

    assert!(matches!(
        plan.invoke(invocation, target).await,
        Err(LocalInvocationError::Cancelled)
    ));
    assert_eq!(local_marker.borrow().as_str(), "worker-local");
}
