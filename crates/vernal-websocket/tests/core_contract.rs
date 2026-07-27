//! WebSocket 核心行为测试。

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http::{HeaderMap, Method, Uri};
use vernal_websocket::{
    BackpressurePolicy, CloseCode, CloseStatus, HandshakeRequest, LifecycleController,
    LifecycleState, MemoryWebSocketSession, MessageKind, OriginPolicy, OutboundQueue, SessionState,
    WebSocketError, WebSocketHandler, WebSocketMessage, WebSocketRegistry, WebSocketSession,
    is_valid_subprotocol, negotiate_subprotocol,
};

#[test]
fn message_kinds_and_payloads_match_spring_semantics() {
    let text = WebSocketMessage::text("hello");
    assert_eq!(text.kind(), MessageKind::Text);
    assert_eq!(text.payload_len(), 5);
    assert!(text.is_last());
    assert!(!text.is_control());

    let binary = WebSocketMessage::binary(Bytes::from_static(b"data"));
    assert_eq!(binary.kind(), MessageKind::Binary);
    assert_eq!(binary.payload_len(), 4);

    let continuation = WebSocketMessage::Continuation {
        payload: Bytes::from_static(b"part"),
        last: false,
    };
    assert_eq!(continuation.kind(), MessageKind::Continuation);
    assert!(!continuation.is_last());
}

#[test]
fn close_status_validates_wire_codes_and_reason_size() {
    assert_eq!(CloseStatus::normal().code(), CloseCode::Normal);
    assert!(CloseStatus::new(CloseCode::Normal, "done").is_ok());
    assert!(CloseStatus::new(CloseCode::NoStatus, "").is_err());
    assert!(CloseStatus::new(CloseCode::Custom(4999), "").is_ok());
    assert!(CloseStatus::new(CloseCode::Normal, "x".repeat(124)).is_err());
}

#[tokio::test]
async fn memory_session_sends_and_closes_exactly_once() {
    let session = MemoryWebSocketSession::new(
        "session-1",
        Some(Uri::from_static("ws://localhost/ws")),
        HeaderMap::new(),
    );
    assert_eq!(session.state(), SessionState::Open);
    session
        .send(WebSocketMessage::text("hello"))
        .await
        .expect("send succeeds");
    session
        .close(CloseStatus::normal())
        .await
        .expect("close succeeds");
    session
        .close(CloseStatus::normal())
        .await
        .expect("second close is idempotent");
    assert_eq!(session.state(), SessionState::Closed);
    assert_eq!(session.sent_messages().await.len(), 2);
}

#[tokio::test]
async fn handler_default_lifecycle_is_async_and_successful() {
    let handler = TestHandler;
    let session = MemoryWebSocketSession::new("handler", None, HeaderMap::new());
    handler.on_open(session.as_ref()).await.expect("open");
    handler
        .on_message(session.as_ref(), WebSocketMessage::text("x"))
        .await
        .expect("message");
    handler
        .on_error(session.as_ref(), &WebSocketError::Closed)
        .await;
    handler
        .on_close(session.as_ref(), CloseStatus::normal())
        .await;
    assert!(!handler.supports_partial_messages());
}

struct TestHandler;
impl WebSocketHandler for TestHandler {}

#[tokio::test]
async fn backpressure_policy_enforces_bounded_queue() {
    let queue = OutboundQueue::new(1, BackpressurePolicy::Terminate, Duration::from_millis(10));
    queue
        .enqueue(WebSocketMessage::text("first"))
        .await
        .expect("first fits");
    assert!(matches!(
        queue.enqueue(WebSocketMessage::text("second")).await,
        Err(WebSocketError::Backpressure)
    ));
    assert_eq!(queue.recv().await, Some(WebSocketMessage::text("first")));
}

#[tokio::test]
async fn wait_policy_times_out_when_consumer_is_slow() {
    let queue = OutboundQueue::new(1, BackpressurePolicy::Wait, Duration::from_millis(1));
    queue
        .enqueue(WebSocketMessage::text("first"))
        .await
        .expect("first fits");
    assert!(matches!(
        queue.enqueue(WebSocketMessage::text("second")).await,
        Err(WebSocketError::Backpressure)
    ));
}

#[test]
fn handshake_validates_upgrade_headers_and_subprotocols() {
    let mut headers = HeaderMap::new();
    headers.insert("upgrade", "websocket".parse().expect("header"));
    headers.insert("connection", "keep-alive, Upgrade".parse().expect("header"));
    headers.insert("sec-websocket-version", "13".parse().expect("header"));
    headers.insert("sec-websocket-key", "key".parse().expect("header"));
    headers.insert(
        "sec-websocket-protocol",
        "chat, v12.stomp".parse().expect("header"),
    );
    let request = HandshakeRequest::new(Method::GET, Uri::from_static("/ws"), headers);
    request.validate().expect("valid handshake");
    assert_eq!(
        request.negotiate_subprotocol(&["v12.stomp".into()]),
        Some("v12.stomp".into())
    );
}

#[test]
fn origin_policy_and_protocol_names_are_deterministic() {
    let policy = OriginPolicy::deny_all().allow("https://example.test");
    let mut headers = HeaderMap::new();
    headers.insert("origin", "https://example.test".parse().expect("header"));
    policy.validate(&headers).expect("origin allowed");
    assert!(is_valid_subprotocol("v12.stomp"));
    assert!(!is_valid_subprotocol("bad protocol"));
    assert_eq!(
        negotiate_subprotocol(&["chat", "v12.stomp"], &["v12.stomp".into()]),
        Some("v12.stomp".into())
    );
}

#[tokio::test]
async fn registry_supports_exact_and_prefix_routes() {
    let mut registry = WebSocketRegistry::new();
    registry.register("/chat/*", Arc::new(TestHandler));
    assert!(registry.get("/chat/room-1").is_some());
    assert!(registry.get("/other").is_none());
}

#[tokio::test]
async fn lifecycle_controller_drains_and_cancels() {
    let lifecycle = LifecycleController::new();
    assert_eq!(lifecycle.state().await, LifecycleState::Created);
    lifecycle.start().await;
    assert_eq!(lifecycle.state().await, LifecycleState::Running);
    lifecycle.drain().await;
    assert!(lifecycle.cancellation().is_cancelled());
    assert_eq!(lifecycle.state().await, LifecycleState::Draining);
    lifecycle.stop().await;
    assert_eq!(lifecycle.state().await, LifecycleState::Stopped);
}
