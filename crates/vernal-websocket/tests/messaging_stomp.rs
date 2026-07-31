//! 对标 Spring `spring-websocket` messaging 测试矩阵：SubProtocol / STOMP bridge / user registry / events。

use std::sync::Arc;

use http::HeaderMap;
use vernal_messaging::{
    DefaultSimpUserRegistry, GenericMessage, InMemoryChannel, MessageChannel, SimpSubscription,
};
use vernal_websocket::MemoryWebSocketSession;
use vernal_websocket::messaging::{
    SessionDisconnectEvent, SessionSubscribeEvent, StompErrorMessage, StompSessionState,
    StompSubProtocolErrorHandler, StompSubProtocolHandler, SubProtocolErrorHandler,
    SubProtocolEvent, SubProtocolHandler, WebSocketStompClient,
};
use vernal_websocket::WebSocketMessage;

#[tokio::test]
async fn stomp_sub_protocol_handler_decodes_connect_and_sends_to_inbound_channel() {
    let channel = Arc::new(InMemoryChannel::new(16));
    let handler = StompSubProtocolHandler::new();
    let session = MemoryWebSocketSession::new("stomp-1", None, HeaderMap::new());
    let connect_frame = "CONNECT\naccept-version:1.2\nhost:stomp.github.org\n\n\x00";
    handler
        .handle_message_from_client(
            session.as_ref(),
            WebSocketMessage::text(connect_frame),
            channel.clone(),
        )
        .await
        .unwrap();
    let received = channel
        .receive()
        .await
        .unwrap()
        .expect("message sent to channel");
    assert_eq!(received.payload(), b"");
    let headers = received.headers();
    assert_eq!(
        headers.get("simpMessageType").map(String::as_str),
        Some("CONNECT")
    );
    assert_eq!(
        headers.get("simpSessionId").map(String::as_str),
        Some("stomp-1")
    );
}

#[tokio::test]
async fn stomp_sub_protocol_handler_encodes_connected_for_connect_message_type() {
    let handler = StompSubProtocolHandler::new();
    let session = MemoryWebSocketSession::new("stomp-2", None, HeaderMap::new());
    let mut message = GenericMessage::new("msg-1", Vec::new());
    message = message
        .with_header("simpMessageType", "CONNECT")
        .with_header("simpSessionId", "stomp-2");
    handler
        .handle_message_to_client(session.as_ref(), Arc::new(message))
        .await
        .unwrap();
    let sent = session.sent_messages().await;
    assert_eq!(sent.len(), 1);
    match &sent[0] {
        WebSocketMessage::Text(text) => assert!(text.starts_with("CONNECTED")),
        other => panic!("expected text, got {other:?}"),
    }
}

#[tokio::test]
async fn stomp_sub_protocol_handler_encodes_message_frame_with_destination_and_subscription() {
    let handler = StompSubProtocolHandler::new();
    let session = MemoryWebSocketSession::new("stomp-3", None, HeaderMap::new());
    let mut message = GenericMessage::new("msg-2", b"payload".to_vec());
    message = message
        .with_header("simpMessageType", "MESSAGE")
        .with_header("simpSessionId", "stomp-3")
        .with_header("simpDestination", "/topic/test")
        .with_header("simpSubscriptionId", "sub-1");
    handler
        .handle_message_to_client(session.as_ref(), Arc::new(message))
        .await
        .unwrap();
    let sent = session.sent_messages().await;
    assert_eq!(sent.len(), 1);
    match &sent[0] {
        WebSocketMessage::Text(text) => {
            assert!(text.starts_with("MESSAGE"));
            assert!(text.contains("destination:/topic/test"));
            assert!(text.contains("subscription:sub-1"));
        }
        other => panic!("expected text, got {other:?}"),
    }
}

#[tokio::test]
async fn stomp_sub_protocol_handler_resolves_session_id_from_headers() {
    let handler = StompSubProtocolHandler::new();
    let mut message = GenericMessage::new("m", Vec::new());
    message = message.with_header("simpSessionId", "resolve-test");
    assert_eq!(
        handler.resolve_session_id(&message),
        Some("resolve-test".to_string())
    );
}

#[tokio::test]
async fn stomp_sub_protocol_error_handler_produces_error_frame() {
    let handler = StompSubProtocolErrorHandler::new();
    let error = StompErrorMessage {
        message: "Forbidden".into(),
    };
    let bytes = handler.handle_error(&error).await.unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.starts_with("ERROR"));
    assert!(text.contains("message:Forbidden"));
}

#[tokio::test]
async fn stomp_client_connect_advances_to_connected() {
    let client = WebSocketStompClient::new("ws://localhost/stomp");
    let connect = client.connect("localhost").unwrap();
    if let WebSocketMessage::Text(text) = &connect {
        assert!(text.starts_with("CONNECT"));
    } else {
        panic!("expected text");
    }
    let connected = "CONNECTED\nversion:1.2\nheart-beat:10000,10000\n\n\x00";
    client
        .session
        .handle_message(&WebSocketMessage::text(connected))
        .await
        .unwrap();
    assert_eq!(client.session.state().await, StompSessionState::Connected);
}

#[tokio::test]
async fn stomp_client_subscribe_unsubscribe_and_send() {
    let client = WebSocketStompClient::new("ws://localhost/stomp");
    let subscribe = client
        .session
        .subscribe("sub-1", "/topic/test")
        .await
        .unwrap();
    if let WebSocketMessage::Text(text) = &subscribe {
        assert!(text.starts_with("SUBSCRIBE"));
        assert!(text.contains("destination:/topic/test"));
    } else {
        panic!("expected text");
    }
    assert_eq!(client.session.subscription_count().await, 1);
    let send = client
        .session
        .send_frame("/queue/work", b"do work")
        .unwrap();
    if let WebSocketMessage::Text(text) = &send {
        assert!(text.starts_with("SEND"));
        assert!(text.contains("do work"));
    }
    let _ = client.session.unsubscribe("sub-1").await.unwrap();
    assert_eq!(client.session.subscription_count().await, 0);
}

#[tokio::test]
async fn stomp_client_error_frame_closes_session() {
    let client = WebSocketStompClient::new("ws://localhost/stomp");
    let error_frame = "ERROR\nmessage:access denied\n\n\x00";
    let result = client
        .session
        .handle_message(&WebSocketMessage::text(error_frame))
        .await;
    assert!(result.is_err());
    assert_eq!(client.session.state().await, StompSessionState::Closed);
}

#[tokio::test]
async fn default_simp_user_registry_tracks_sessions_and_subscriptions() {
    let registry = DefaultSimpUserRegistry::new();
    registry
        .register_session(Some("alice".into()), "sess-1")
        .await;
    assert_eq!(registry.user_count().await, 1);
    assert_eq!(registry.session_count().await, 1);
    registry
        .add_subscription(
            "sess-1",
            SimpSubscription {
                id: "sub-1".into(),
                destination: "/topic/news".into(),
                session_id: "sess-1".into(),
            },
        )
        .await;
    let session = registry.get_session("sess-1").await.unwrap();
    assert_eq!(session.subscriptions.len(), 1);
    registry.remove_subscription("sess-1", "sub-1").await;
    let session = registry.get_session("sess-1").await.unwrap();
    assert!(session.subscriptions.is_empty());
    registry.remove_session("sess-1").await;
    assert_eq!(registry.session_count().await, 0);
}

#[test]
fn sub_protocol_event_carries_session_and_user() {
    let event = SubProtocolEvent::new("stomp", "sess-1").with_user("alice");
    assert_eq!(event.session_id, "sess-1");
    assert_eq!(event.user.as_deref(), Some("alice"));
}

#[test]
fn session_disconnect_event_includes_close_code_and_reason() {
    let event = SessionDisconnectEvent::new("stomp", "sess-2", 1000, "normal");
    assert_eq!(event.close_code, 1000);
    assert_eq!(event.close_reason, "normal");
}

#[test]
fn session_subscribe_event_includes_subscription_and_destination() {
    let event = SessionSubscribeEvent::new("stomp", "sess-3", "sub-1", "/topic/test");
    assert_eq!(event.subscription_id, "sub-1");
    assert_eq!(event.destination, "/topic/test");
}

#[tokio::test]
async fn stomp_handler_after_session_ended_sends_disconnect_message() {
    let channel = Arc::new(InMemoryChannel::new(16));
    let handler = StompSubProtocolHandler::new();
    let session = MemoryWebSocketSession::new("stomp-end", None, HeaderMap::new());
    handler
        .after_session_ended(
            session.as_ref(),
            vernal_websocket::CloseStatus::normal(),
            channel.clone(),
        )
        .await
        .unwrap();
    let received = channel
        .receive()
        .await
        .unwrap()
        .expect("disconnect message");
    let headers = received.headers();
    assert_eq!(
        headers.get("simpMessageType").map(String::as_str),
        Some("DISCONNECT")
    );
    assert_eq!(
        headers.get("simpSessionId").map(String::as_str),
        Some("stomp-end")
    );
}
