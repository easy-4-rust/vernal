//! Vernal HTTP 有限 Body、流式 Frame、取消和 owned snapshot 合同测试。

use futures_util::stream;
use tokio_util::sync::CancellationToken;
use vernal_http::{
    Bytes, Frame, HeaderMap, HeaderValue, HttpBody, HttpBodyError, HttpRequestSnapshot, Method, Uri,
};

#[tokio::test]
async fn finite_body_collects_without_losing_bytes() {
    let collected = HttpBody::full("vernal")
        .collect_limited(64)
        .await
        .expect("finite body should collect");
    assert_eq!(collected.bytes(), &Bytes::from_static(b"vernal"));
    assert!(collected.trailers().is_none());
}

#[tokio::test]
async fn stream_preserves_data_frames_and_trailers() {
    let mut trailers = HeaderMap::new();
    trailers.insert("x-vernal-checksum", HeaderValue::from_static("ok"));
    let frames = stream::iter(vec![
        Ok(Frame::data(Bytes::from_static(b"grow "))),
        Ok(Frame::data(Bytes::from_static(b"components"))),
        Ok(Frame::trailers(trailers)),
    ]);

    let collected = HttpBody::from_stream(frames)
        .collect_limited(64)
        .await
        .expect("stream should collect");
    assert_eq!(collected.bytes(), &Bytes::from_static(b"grow components"));
    assert_eq!(
        collected
            .trailers()
            .and_then(|headers| headers.get("x-vernal-checksum")),
        Some(&HeaderValue::from_static("ok"))
    );
}

#[tokio::test]
async fn collection_rejects_body_over_explicit_limit() {
    let error = HttpBody::full("12345")
        .collect_limited(4)
        .await
        .expect_err("limit should be enforced");
    assert!(matches!(
        error,
        HttpBodyError::LimitExceeded {
            limit: 4,
            observed: 5
        }
    ));
}

#[tokio::test]
async fn cancellation_wakes_a_pending_body_stream() {
    let cancellation = CancellationToken::new();
    let body = HttpBody::from_stream(stream::pending()).with_cancellation(cancellation.clone());
    let task = tokio::spawn(async move { body.collect_limited(64).await });
    tokio::task::yield_now().await;
    cancellation.cancel();

    assert!(matches!(
        task.await.expect("collector task should join"),
        Err(HttpBodyError::Cancelled)
    ));
}

#[test]
fn snapshot_owns_method_uri_version_and_headers_without_body() {
    let request = http::Request::builder()
        .method(Method::POST)
        .uri("/orders?dry_run=true")
        .header("x-request-id", "request-42")
        .body(())
        .expect("valid request");
    let snapshot = HttpRequestSnapshot::capture(&request);
    drop(request);

    assert_eq!(snapshot.method(), Method::POST);
    assert_eq!(
        snapshot.uri(),
        &"/orders?dry_run=true"
            .parse::<Uri>()
            .expect("valid expected URI")
    );
    assert_eq!(
        snapshot.headers().get("x-request-id"),
        Some(&HeaderValue::from_static("request-42"))
    );
}

#[test]
fn snapshot_can_be_constructed_from_framework_native_request_parts() {
    let request = http::Request::builder()
        .method(Method::PATCH)
        .uri("/orders/42")
        .header("x-request-id", "request-43")
        .body(())
        .expect("valid request");
    let (parts, ()) = request.into_parts();
    let snapshot =
        HttpRequestSnapshot::from_parts(parts.method, parts.uri, parts.version, parts.headers);

    assert_eq!(snapshot.method(), Method::PATCH);
    assert_eq!(snapshot.uri().path(), "/orders/42");
    assert_eq!(
        snapshot.headers().get("x-request-id"),
        Some(&HeaderValue::from_static("request-43"))
    );
}
