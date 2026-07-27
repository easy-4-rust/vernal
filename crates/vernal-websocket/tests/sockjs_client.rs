//! 对标 Spring `spring-websocket` SockJS client 测试矩阵 + IMPLEMENTED_UNVERIFIED 行为验证。

use std::sync::Arc;

use http::HeaderMap;
use vernal_websocket::sockjs::client::{
    ClientSessionState, ClientSockJsSession, DefaultTransportRequest, HttpRequestExecutor,
    InfoReceiver, ServerInfo, SockJsClient, SockJsUrlInfo, Transport, WebSocketClientTransport,
    XhrTransportImpl, parse_info_json,
};
use vernal_websocket::sockjs::frame::json_sockjs_message_codec::JsonSockJsMessageCodec;
use vernal_websocket::sockjs::transport::transport_type::TransportType;

// ---- SockJsUrlInfo ----

#[test]
fn url_info_generates_server_and_session_ids() {
    let info = SockJsUrlInfo::new("http://localhost:8080/sockjs");
    let sid = info.server_id();
    let sess = info.session_id();
    assert!(!sid.is_empty());
    assert!(!sess.is_empty());
    // 多次调用返回相同值
    assert_eq!(info.server_id(), sid);
    assert_eq!(info.session_id(), sess);
}

#[test]
fn url_info_info_url_appends_info_suffix() {
    let info = SockJsUrlInfo::new("http://localhost:8080/sockjs");
    assert_eq!(info.info_url(), "http://localhost:8080/sockjs/info");
}

#[test]
fn url_info_transport_url_includes_server_session_transport() {
    let info = SockJsUrlInfo::with_ids("http://localhost/sockjs", "123", "abc456");
    let url = info.transport_url(TransportType::Xhr);
    assert_eq!(url, "http://localhost/sockjs/123/abc456/xhr");
}

// ---- InfoReceiver / parse_info_json ----

#[test]
fn parse_info_json_extracts_websocket_and_cookie() {
    let json = r#"{"websocket":true,"cookie_needed":false,"entropy":12345}"#;
    let info = parse_info_json(json).unwrap();
    assert!(info.websocket);
    assert!(!info.cookie_needed);
    assert_eq!(info.entropy, 12345);
}

struct MockInfoReceiver {
    response: String,
}

impl InfoReceiver for MockInfoReceiver {
    fn fetch_info<'a>(&'a self, _url: &'a str) -> vernal_websocket::sockjs::client::InfoFuture<'a> {
        Box::pin(async move { parse_info_json(&self.response) })
    }
}

// ---- ClientSockJsSession ----

struct NullHandler;
impl vernal_websocket::WebSocketHandler for NullHandler {}

#[tokio::test]
async fn client_session_starts_new_and_transitions_on_open_frame() {
    let handler: Arc<dyn vernal_websocket::WebSocketHandler> = Arc::new(NullHandler);
    let codec: Arc<dyn vernal_websocket::sockjs::frame::sockjs_message_codec::SockJsMessageCodec> =
        Arc::new(JsonSockJsMessageCodec::new());
    let session = ClientSockJsSession::new("client-1", codec, handler);
    assert_eq!(session.state().await, ClientSessionState::New);
    session.handle_frame("o").await;
    assert_eq!(session.state().await, ClientSessionState::Open);
}

#[tokio::test]
async fn client_session_caches_messages_from_message_frame() {
    let handler: Arc<dyn vernal_websocket::WebSocketHandler> = Arc::new(NullHandler);
    let codec: Arc<dyn vernal_websocket::sockjs::frame::sockjs_message_codec::SockJsMessageCodec> =
        Arc::new(JsonSockJsMessageCodec::new());
    let session = ClientSockJsSession::new("client-2", codec, handler);
    session.handle_frame("o").await;
    session.handle_frame(r#"a["hello","world"]"#).await;
    assert_eq!(session.message_count().await, 2);
    let drained = session.drain_messages().await;
    assert_eq!(drained, vec!["hello".to_string(), "world".to_string()]);
}

#[tokio::test]
async fn client_session_closes_on_close_frame() {
    let handler: Arc<dyn vernal_websocket::WebSocketHandler> = Arc::new(NullHandler);
    let codec: Arc<dyn vernal_websocket::sockjs::frame::sockjs_message_codec::SockJsMessageCodec> =
        Arc::new(JsonSockJsMessageCodec::new());
    let session = ClientSockJsSession::new("client-3", codec, handler);
    session.handle_frame("o").await;
    session.handle_frame("c[3000,\"Go away!\"]").await;
    assert_eq!(session.state().await, ClientSessionState::Closed);
}

#[tokio::test]
async fn client_session_heartbeat_is_ignored() {
    let handler: Arc<dyn vernal_websocket::WebSocketHandler> = Arc::new(NullHandler);
    let codec: Arc<dyn vernal_websocket::sockjs::frame::sockjs_message_codec::SockJsMessageCodec> =
        Arc::new(JsonSockJsMessageCodec::new());
    let session = ClientSockJsSession::new("client-4", codec, handler);
    session.handle_frame("o").await;
    session.handle_frame("h").await;
    assert_eq!(session.message_count().await, 0);
}

#[tokio::test]
async fn client_session_disable_heartbeat_flag() {
    let handler: Arc<dyn vernal_websocket::WebSocketHandler> = Arc::new(NullHandler);
    let codec: Arc<dyn vernal_websocket::sockjs::frame::sockjs_message_codec::SockJsMessageCodec> =
        Arc::new(JsonSockJsMessageCodec::new());
    let session = ClientSockJsSession::new("client-5", codec, handler);
    assert!(!session.heartbeat_disabled());
    session.disable_heartbeat();
    assert!(session.heartbeat_disabled());
}

// ---- Transport SPI ----

#[test]
fn websocket_client_transport_reports_websocket_type() {
    let transport = WebSocketClientTransport::new();
    assert_eq!(transport.transport_types(), vec![TransportType::WebSocket]);
}

struct MockHttpExecutor;

impl HttpRequestExecutor for MockHttpExecutor {
    fn execute_get<'a>(
        &'a self,
        _url: &'a str,
        _headers: &'a HeaderMap,
    ) -> futures_util::future::BoxFuture<'a, Result<(u16, Vec<u8>), String>> {
        Box::pin(async { Ok((200, b"{}".to_vec())) })
    }
    fn execute_post<'a>(
        &'a self,
        _url: &'a str,
        _headers: &'a HeaderMap,
        _body: &'a [u8],
    ) -> futures_util::future::BoxFuture<'a, Result<(u16, Vec<u8>), String>> {
        Box::pin(async { Ok((204, Vec::new())) })
    }
}

#[test]
fn xhr_streaming_transport_reports_both_streaming_and_polling() {
    let executor: Arc<dyn HttpRequestExecutor> = Arc::new(MockHttpExecutor);
    let transport = XhrTransportImpl::new_streaming(executor);
    assert!(transport.is_streaming());
    let types = transport.transport_types();
    assert!(types.contains(&TransportType::XhrStreaming));
    assert!(types.contains(&TransportType::Xhr));
}

#[test]
fn xhr_polling_transport_reports_only_xhr() {
    let executor: Arc<dyn HttpRequestExecutor> = Arc::new(MockHttpExecutor);
    let transport = XhrTransportImpl::new_polling(executor);
    assert!(!transport.is_streaming());
    assert_eq!(transport.transport_types(), vec![TransportType::Xhr]);
}

#[tokio::test]
async fn xhr_transport_send_messages_succeeds_on_204() {
    let executor: Arc<dyn HttpRequestExecutor> = Arc::new(MockHttpExecutor);
    let transport = XhrTransportImpl::new_polling(executor);
    let result = transport
        .send_messages(
            "http://localhost/xhr_send",
            b"[\"hello\"]",
            &HeaderMap::new(),
        )
        .await;
    assert!(result.is_ok());
}

// ---- DefaultTransportRequest ----

#[test]
fn default_transport_request_builds_correct_transport_url() {
    use vernal_websocket::sockjs::client::TransportRequest;
    let info = SockJsUrlInfo::with_ids("http://localhost/sjs", "0", "sess");
    let codec: Arc<dyn vernal_websocket::sockjs::frame::sockjs_message_codec::SockJsMessageCodec> =
        Arc::new(JsonSockJsMessageCodec::new());
    let req = DefaultTransportRequest::new(info, TransportType::XhrSend, codec);
    assert_eq!(req.transport_url(), "http://localhost/sjs/0/sess/xhr_send");
}

// ---- SockJsClient ----

#[tokio::test]
async fn sockjs_client_connects_and_creates_session() {
    let info_receiver: Arc<dyn InfoReceiver> = Arc::new(MockInfoReceiver {
        response: r#"{"websocket":true,"cookie_needed":false,"entropy":42}"#.to_string(),
    });
    let ws_transport: Arc<dyn Transport> =
        Arc::new(WebSocketClientTransport::with_validator(Arc::new(|_| {
            Ok(())
        })));
    let client = SockJsClient::new(vec![ws_transport], info_receiver);
    assert_eq!(client.transport_count(), 1);
    let handler: Arc<dyn vernal_websocket::WebSocketHandler> = Arc::new(NullHandler);
    let session = client
        .connect("http://localhost:8080/sockjs", handler)
        .await
        .unwrap();
    assert!(!session.id().is_empty());
}

// ---- IMPLEMENTED_UNVERIFIED 行为验证 ----

#[tokio::test]
async fn abstract_websocket_session_initializes_native_and_send_fails_before_init() {
    use vernal_websocket::adapter::AbstractWebSocketSession;
    use vernal_websocket::{WebSocketMessage, WebSocketSession};

    struct DummyNative;
    let session = AbstractWebSocketSession::<DummyNative>::new(
        "adapter-test",
        None,
        HeaderMap::new(),
        None,
        Default::default(),
    );
    // 未初始化 native 时 send 应报错
    let result = session.send(WebSocketMessage::text("hi")).await;
    assert!(result.is_err());
    // 初始化 native 后 send 成功
    session
        .initialize_native_session(Arc::new(DummyNative))
        .await;
    let result = session.send(WebSocketMessage::text("hi")).await;
    assert!(result.is_ok());
    // close 成功
    let result = session.close(vernal_websocket::CloseStatus::normal()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn abstract_websocket_client_default_headers_preserved() {
    use vernal_websocket::client::AbstractWebSocketClient;
    let mut headers = HeaderMap::new();
    headers.insert("x-custom", "value".parse().unwrap());
    let client = AbstractWebSocketClient::new().with_default_headers(headers);
    assert_eq!(client.default_headers.get("x-custom").unwrap(), "value");
}

#[tokio::test]
async fn transport_handling_sockjs_service_returns_handler_by_type() {
    use std::sync::Arc;
    use vernal_websocket::sockjs::transport::transport_handler::TransportHandler;
    use vernal_websocket::sockjs::transport::{
        SockJsServiceConfig, TransportHandlingSockJsService, TransportType,
    };

    struct StubHandler;
    impl TransportHandler for StubHandler {
        fn initialize(&self, _config: SockJsServiceConfig) {}
        fn transport_type(&self) -> TransportType {
            TransportType::Xhr
        }
        fn check_session_type(
            &self,
            _: &dyn vernal_websocket::sockjs::transport::sockjs_session::SockJsSession,
        ) -> bool {
            true
        }
        fn handle_request(
            &self,
            _: Arc<dyn vernal_websocket::WebSocketHandler>,
            _: Arc<dyn vernal_websocket::sockjs::transport::sockjs_session::SockJsSession>,
        ) -> vernal_websocket::sockjs::transport::transport_handler::TransportHandleFuture {
            Box::pin(async { Ok(()) })
        }
    }

    let handler: Arc<dyn TransportHandler> = Arc::new(StubHandler);
    let config = SockJsServiceConfig::new();
    let service = TransportHandlingSockJsService::new(config, vec![handler]);
    assert!(service.handler(TransportType::Xhr).is_some());
    assert!(service.handler(TransportType::WebSocket).is_none());
}
