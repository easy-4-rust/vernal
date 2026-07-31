//! 对标 Spring `spring-websocket` SockJS 测试矩阵：session 状态机、transport handler、HTTP 请求处理。

use std::time::Duration;

use vernal_websocket::sockjs::frame::default_sockjs_frame_format::DefaultSockJsFrameFormat;
use vernal_websocket::sockjs::frame::sockjs_frame::SockJsFrame;
use vernal_websocket::sockjs::frame::sockjs_frame_type::SockJsFrameType;
use vernal_websocket::sockjs::transport::SockJsServiceConfig;
use vernal_websocket::sockjs::transport::TransportType;
use vernal_websocket::sockjs::transport::handler::{
    HttpReceivingTransportHandler, HttpSendingTransportHandler, WebSocketTransportHandler,
};
use vernal_websocket::sockjs::transport::session::{
    AbstractSockJsSession, HttpSockJsSession, WebSocketServerSockJsSession,
};
use vernal_websocket::sockjs::transport::transport_handler::TransportHandler;

#[tokio::test]
async fn sockjs_session_lifecycle_new_to_open_to_closed() {
    let session =
        AbstractSockJsSession::new("test-1", Duration::from_secs(25), Duration::from_secs(5));
    assert!(session.is_new().await);
    assert!(!session.is_open().await);
    session.open().await;
    assert!(session.is_open().await);
    assert!(!session.is_new().await);
    session.close().await;
    assert!(session.is_closed().await);
}

#[tokio::test]
async fn sockjs_session_message_cache_enqueue_and_drain() {
    let session =
        AbstractSockJsSession::new("test-2", Duration::from_secs(25), Duration::from_secs(5));
    session.enqueue_message("hello").await;
    session.enqueue_message("world").await;
    assert_eq!(session.message_count().await, 2);
    let drained = session.drain_messages().await;
    assert_eq!(drained, vec!["hello".to_string(), "world".to_string()]);
    assert_eq!(session.message_count().await, 0);
}

#[tokio::test]
async fn sockjs_session_heartbeat_can_be_disabled() {
    let session =
        AbstractSockJsSession::new("test-3", Duration::from_secs(25), Duration::from_secs(5));
    assert!(!session.heartbeat_disabled());
    session.disable_heartbeat();
    assert!(session.heartbeat_disabled());
}

#[tokio::test]
async fn http_sockjs_session_initial_request_returns_open_frame() {
    let session = HttpSockJsSession::new(
        "http-1",
        Duration::from_secs(25),
        Duration::from_secs(5),
        128 * 1024,
    );
    let format = DefaultSockJsFrameFormat;
    let response = session.handle_initial_request(&format).await;
    let text = String::from_utf8_lossy(&response);
    assert!(text.starts_with("o\n"));
    assert!(session.is_active().await);
}

#[tokio::test]
async fn http_sockjs_session_successive_request_returns_heartbeat_when_no_messages() {
    let session = HttpSockJsSession::new(
        "http-2",
        Duration::from_secs(25),
        Duration::from_secs(5),
        128 * 1024,
    );
    let format = DefaultSockJsFrameFormat;
    session.handle_initial_request(&format).await;
    let response = session.handle_successive_request(&format).await;
    let text = String::from_utf8_lossy(&response);
    assert!(text.starts_with("h\n"));
}

#[tokio::test]
async fn http_sockjs_session_delegate_messages_then_drain() {
    let session = HttpSockJsSession::new(
        "http-3",
        Duration::from_secs(25),
        Duration::from_secs(5),
        128 * 1024,
    );
    session
        .delegate_messages(&["msg1".into(), "msg2".into()])
        .await;
    assert_eq!(session.base().message_count().await, 2);
    let drained = session.base().drain_messages().await;
    assert_eq!(drained, vec!["msg1".to_string(), "msg2".to_string()]);
}

#[tokio::test]
async fn http_receiving_handler_decodes_post_body_and_delegates() {
    let handler = HttpReceivingTransportHandler::xhr_send();
    let config = SockJsServiceConfig::new();
    handler.initialize(config);
    let session = HttpSockJsSession::new(
        "recv-1",
        Duration::from_secs(25),
        Duration::from_secs(5),
        128 * 1024,
    );
    let body = br#"["hello"]"#;
    let response = handler.handle_post(body, session.base()).await.unwrap();
    assert_eq!(response, b"ok");
    assert_eq!(session.base().message_count().await, 1);
}

#[tokio::test]
async fn http_receiving_handler_rejects_empty_payload() {
    let handler = HttpReceivingTransportHandler::xhr_send();
    let config = SockJsServiceConfig::new();
    handler.initialize(config);
    let session = HttpSockJsSession::new(
        "recv-2",
        Duration::from_secs(25),
        Duration::from_secs(5),
        128 * 1024,
    );
    let result = handler.handle_post(b"", session.base()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn http_sending_handler_xhr_polling_returns_content_type_and_open() {
    let handler = HttpSendingTransportHandler::xhr_polling();
    handler.initialize(SockJsServiceConfig::new());
    let session = HttpSockJsSession::new(
        "send-1",
        Duration::from_secs(25),
        Duration::from_secs(5),
        128 * 1024,
    );
    let (content_type, body) = handler.handle_send(&session).await.unwrap();
    assert!(content_type.contains("javascript"));
    let text = String::from_utf8_lossy(&body);
    assert!(text.starts_with("o"));
}

#[tokio::test]
async fn http_sending_handler_event_source_uses_data_prefix() {
    let handler = HttpSendingTransportHandler::event_source();
    handler.initialize(SockJsServiceConfig::new());
    let session = HttpSockJsSession::new(
        "send-2",
        Duration::from_secs(25),
        Duration::from_secs(5),
        128 * 1024,
    );
    let (content_type, body) = handler.handle_send(&session).await.unwrap();
    assert_eq!(content_type, "text/event-stream; charset=UTF-8");
    let text = String::from_utf8_lossy(&body);
    assert!(text.starts_with("data: o"));
}

#[tokio::test]
async fn websocket_transport_handler_reports_correct_type() {
    let handler = WebSocketTransportHandler::new();
    handler.initialize(SockJsServiceConfig::new());
    assert_eq!(handler.transport_type(), TransportType::WebSocket);
}

#[tokio::test]
async fn websocket_server_sockjs_session_open_and_close() {
    let session =
        WebSocketServerSockJsSession::new("ws-1", Duration::from_secs(25), Duration::from_secs(5));
    session.base().open().await;
    assert!(session.base().is_open().await);
    session.base().close().await;
    assert!(session.base().is_closed().await);
}

#[test]
fn sockjs_frame_close_go_away_matches_protocol() {
    let frame = SockJsFrame::close_frame_go_away();
    assert_eq!(frame.frame_type(), SockJsFrameType::Close);
    assert_eq!(frame.content(), "c[3000,\"Go away!\"]");
}

#[tokio::test]
async fn http_sockjs_session_stream_bytes_limit_recycling() {
    let session = HttpSockJsSession::new(
        "stream-1",
        Duration::from_secs(25),
        Duration::from_secs(5),
        10,
    );
    assert!(!session.add_stream_bytes(5).await);
    assert!(session.add_stream_bytes(6).await);
}
