//! Vernal Tower 错误恢复、转换与 readiness 语义合同测试。

#[path = "error_mapping_support/readiness_failure_service.rs"]
mod readiness_failure_service;
#[path = "error_mapping_support/test_error_mapper.rs"]
mod test_error_mapper;

use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use http::{Request, Response, StatusCode};
use readiness_failure_service::ReadinessFailureService;
use test_error_mapper::TestErrorMapper;
use tower::{Layer, ServiceExt, service_fn};
use vernal_tower::ErrorMappingLayer;

#[tokio::test]
async fn call_error_can_recover_into_native_response() {
    let service = service_fn(|_request: Request<()>| async {
        Err::<Response<u8>, _>(io::Error::other("dependency unavailable"))
    });

    let response = ErrorMappingLayer::new(TestErrorMapper::recovering())
        .layer(service)
        .oneshot(Request::new(()))
        .await
        .expect("error should recover into response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()["x-vernal-mapped"], "true");
    assert_eq!(*response.body(), 42);
}

#[tokio::test]
async fn call_error_can_transform_into_new_service_error() {
    let service = service_fn(|_request: Request<()>| async {
        Err::<Response<u8>, _>(io::Error::other("original"))
    });

    let error = ErrorMappingLayer::new(TestErrorMapper::transforming())
        .layer(service)
        .oneshot(Request::new(()))
        .await
        .expect_err("mapped error should remain Service::Error");

    assert_eq!(error.to_string(), "mapped: original");
}

#[tokio::test]
async fn successful_response_bypasses_mapper_unchanged() {
    let service = service_fn(|_request: Request<()>| async {
        Ok::<_, io::Error>(
            Response::builder()
                .status(StatusCode::CREATED)
                .body(7)
                .expect("valid response"),
        )
    });

    let response = ErrorMappingLayer::new(TestErrorMapper::transforming())
        .layer(service)
        .oneshot(Request::new(()))
        .await
        .expect("successful response");

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(*response.body(), 7);
}

#[tokio::test]
async fn readiness_error_recovery_returns_response_without_calling_inner() {
    let called = Arc::new(AtomicBool::new(false));
    let service = ReadinessFailureService::new(Arc::clone(&called));

    let response = ErrorMappingLayer::new(TestErrorMapper::recovering())
        .layer(service)
        .oneshot(Request::new(()))
        .await
        .expect("readiness error should become next call response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(*response.body(), 42);
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn readiness_error_can_become_call_error_without_calling_inner() {
    let called = Arc::new(AtomicBool::new(false));
    let service = ReadinessFailureService::new(Arc::clone(&called));

    let error = ErrorMappingLayer::new(TestErrorMapper::transforming())
        .layer(service)
        .oneshot(Request::new(()))
        .await
        .expect_err("readiness failure should become mapped call error");

    assert_eq!(error.to_string(), "mapped: not ready");
    assert!(!called.load(Ordering::SeqCst));
}
