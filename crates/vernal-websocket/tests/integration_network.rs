//! 真实网络集成测试：XHR/WebSocket client transport + SockJsClient fallback chain + 并发压力。

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http::HeaderMap;
use vernal_websocket::sockjs::client::{
    ClientSessionState, DefaultTransportRequest, HttpRequestExecutor, InfoReceiver, SockJsClient,
    SockJsUrlInfo, Transport, TransportRequest, WebSocketClientTransport, XhrTransportImpl,
    parse_info_json,
};
use vernal_websocket::sockjs::frame::json_sockjs_message_codec::JsonSockJsMessageCodec;
use vernal_websocket::sockjs::frame::sockjs_message_codec::SockJsMessageCodec;
use vernal_websocket::sockjs::transport::SockJsServiceConfig;
use vernal_websocket::sockjs::transport::handler::{
    HttpReceivingTransportHandler, HttpSendingTransportHandler,
};
use vernal_websocket::sockjs::transport::session::{PollingSockJsSession, StreamingSockJsSession};
use vernal_websocket::sockjs::transport::transport_handler::TransportHandler;
use vernal_websocket::sockjs::transport::transport_type::TransportType;
use vernal_websocket::{
    CloseStatus, MemoryWebSocketSession, WebSocketError, WebSocketHandler, WebSocketMessage,
    WebSocketSession,
    handler::{
        AbstractWebSocketHandler, ConcurrentWebSocketSessionDecorator, OverflowStrategy,
        TextWebSocketHandler,
    },
};

struct MockHttpExecutor {
    responses: std::sync::Mutex<Vec<(u16, Vec<u8>)>>,
}
impl MockHttpExecutor {
    fn new(responses: Vec<(u16, Vec<u8>)>) -> Self {
        Self {
            responses: std::sync::Mutex::new(responses),
        }
    }
}
impl HttpRequestExecutor for MockHttpExecutor {
    fn execute_get<'a>(
        &'a self,
        _: &'a str,
        _: &'a HeaderMap,
    ) -> futures_util::future::BoxFuture<'a, Result<(u16, Vec<u8>), String>> {
        Box::pin(async {
            Ok(self
                .responses
                .lock()
                .expect("poisoned")
                .pop()
                .unwrap_or((200, Vec::new())))
        })
    }
    fn execute_post<'a>(
        &'a self,
        _: &'a str,
        _: &'a HeaderMap,
        _: &'a [u8],
    ) -> futures_util::future::BoxFuture<'a, Result<(u16, Vec<u8>), String>> {
        Box::pin(async {
            Ok(self
                .responses
                .lock()
                .expect("poisoned")
                .pop()
                .unwrap_or((204, Vec::new())))
        })
    }
}

struct MockInfoReceiver {
    response: String,
}
impl InfoReceiver for MockInfoReceiver {
    fn fetch_info<'a>(&'a self, _: &'a str) -> vernal_websocket::sockjs::client::InfoFuture<'a> {
        Box::pin(async { parse_info_json(&self.response) })
    }
}

struct NullHandler;
impl WebSocketHandler for NullHandler {}

struct FailingTransport;
impl Transport for FailingTransport {
    fn transport_types(&self) -> Vec<TransportType> {
        vec![TransportType::Xhr]
    }
    fn connect_async<'a>(
        &'a self,
        _: &'a dyn TransportRequest,
        _: Arc<dyn WebSocketHandler>,
    ) -> vernal_websocket::sockjs::client::TransportConnectFuture<'a> {
        Box::pin(async { Err("simulated failure".to_string()) })
    }
}

struct SucceedingTransport;
impl Transport for SucceedingTransport {
    fn transport_types(&self) -> Vec<TransportType> {
        vec![TransportType::XhrStreaming]
    }
    fn connect_async<'a>(
        &'a self,
        _: &'a dyn TransportRequest,
        _: Arc<dyn WebSocketHandler>,
    ) -> vernal_websocket::sockjs::client::TransportConnectFuture<'a> {
        Box::pin(async { Ok(()) })
    }
}

#[tokio::test]
async fn xhr_connect_succeeds_with_open_frame() {
    let executor: Arc<dyn HttpRequestExecutor> =
        Arc::new(MockHttpExecutor::new(vec![(200, b"o\n".to_vec())]));
    let transport = XhrTransportImpl::new_polling(executor);
    let request = DefaultTransportRequest::new(
        SockJsUrlInfo::new("http://localhost/sjs"),
        TransportType::Xhr,
        Arc::new(JsonSockJsMessageCodec::new()),
    );
    let result = transport
        .connect_async(&request, Arc::new(NullHandler))
        .await;
    assert!(result.is_ok(), "XHR connect: {result:?}");
}

#[tokio::test]
async fn xhr_connect_fails_on_500() {
    let executor: Arc<dyn HttpRequestExecutor> =
        Arc::new(MockHttpExecutor::new(vec![(500, b"err".to_vec())]));
    let transport = XhrTransportImpl::new_polling(executor);
    let request = DefaultTransportRequest::new(
        SockJsUrlInfo::new("http://localhost/sjs"),
        TransportType::Xhr,
        Arc::new(JsonSockJsMessageCodec::new()),
    );
    assert!(
        transport
            .connect_async(&request, Arc::new(NullHandler))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn xhr_connect_fails_without_open_frame() {
    let executor: Arc<dyn HttpRequestExecutor> =
        Arc::new(MockHttpExecutor::new(vec![(200, b"h\n".to_vec())]));
    let transport = XhrTransportImpl::new_polling(executor);
    let request = DefaultTransportRequest::new(
        SockJsUrlInfo::new("http://localhost/sjs"),
        TransportType::Xhr,
        Arc::new(JsonSockJsMessageCodec::new()),
    );
    assert!(
        transport
            .connect_async(&request, Arc::new(NullHandler))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn ws_transport_rejects_http_scheme() {
    let transport = WebSocketClientTransport::new();
    let request = DefaultTransportRequest::new(
        SockJsUrlInfo::with_ids("http://localhost/sjs", "0", "sess"),
        TransportType::WebSocket,
        Arc::new(JsonSockJsMessageCodec::new()),
    );
    assert!(
        transport
            .connect_async(&request, Arc::new(NullHandler))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn ws_transport_accepts_with_custom_validator() {
    let transport = WebSocketClientTransport::with_validator(Arc::new(|_| Ok(())));
    let request = DefaultTransportRequest::new(
        SockJsUrlInfo::new("http://localhost/sjs"),
        TransportType::WebSocket,
        Arc::new(JsonSockJsMessageCodec::new()),
    );
    assert!(
        transport
            .connect_async(&request, Arc::new(NullHandler))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn sockjs_client_fallback_on_first_failure() {
    let info_receiver: Arc<dyn InfoReceiver> = Arc::new(MockInfoReceiver {
        response: r#"{"websocket":true,"entropy":42}"#.into(),
    });
    let client = SockJsClient::new(
        vec![Arc::new(FailingTransport), Arc::new(SucceedingTransport)],
        info_receiver,
    );
    let session = client
        .connect("http://localhost/sockjs", Arc::new(NullHandler))
        .await
        .unwrap();
    assert_eq!(session.state().await, ClientSessionState::Open);
}

#[tokio::test]
async fn sockjs_client_fails_when_all_fail() {
    let info_receiver: Arc<dyn InfoReceiver> = Arc::new(MockInfoReceiver {
        response: r#"{"websocket":true,"entropy":42}"#.into(),
    });
    let client = SockJsClient::new(
        vec![Arc::new(FailingTransport), Arc::new(FailingTransport)],
        info_receiver,
    );
    assert!(
        client
            .connect("http://localhost/sockjs", Arc::new(NullHandler))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn sockjs_client_skips_ws_when_no_websocket() {
    let info_receiver: Arc<dyn InfoReceiver> = Arc::new(MockInfoReceiver {
        response: r#"{"websocket":false,"entropy":42}"#.into(),
    });
    let client = SockJsClient::new(
        vec![
            Arc::new(WebSocketClientTransport::new()),
            Arc::new(SucceedingTransport),
        ],
        info_receiver,
    );
    assert!(
        client
            .connect("http://localhost/sockjs", Arc::new(NullHandler))
            .await
            .is_ok()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_decorator_concurrent_sends() {
    let inner: Arc<dyn WebSocketSession> =
        MemoryWebSocketSession::new("c1", None, HeaderMap::new());
    let decorator = Arc::new(ConcurrentWebSocketSessionDecorator::new(
        Arc::clone(&inner),
        Duration::from_secs(10),
        1024 * 1024,
    ));
    let mut handles = Vec::new();
    for i in 0..20 {
        let d = Arc::clone(&decorator);
        handles.push(tokio::spawn(async move {
            d.send_message(WebSocketMessage::text(format!("msg-{i}")))
                .await
        }));
    }
    for h in handles {
        assert!(h.await.unwrap().is_ok());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_decorator_drop_never_errors() {
    let inner: Arc<dyn WebSocketSession> =
        MemoryWebSocketSession::new("c2", None, HeaderMap::new());
    let decorator = Arc::new(ConcurrentWebSocketSessionDecorator::with_strategy(
        Arc::clone(&inner),
        Duration::from_secs(60),
        0,
        OverflowStrategy::Drop,
    ));
    for i in 0..10 {
        assert!(
            Arc::clone(&decorator)
                .send_message(WebSocketMessage::text(format!("m{i}")))
                .await
                .is_ok()
        );
    }
}

#[tokio::test]
async fn polling_session_open_and_message_flush() {
    let codec: Arc<dyn SockJsMessageCodec> = Arc::new(JsonSockJsMessageCodec::new());
    let session = PollingSockJsSession::new(
        "p1",
        Duration::from_secs(25),
        Duration::from_secs(5),
        Arc::clone(&codec),
    );
    let fmt =
        vernal_websocket::sockjs::frame::default_sockjs_frame_format::DefaultSockJsFrameFormat;
    let r = session.handle_request(true, &fmt).await;
    assert!(String::from_utf8_lossy(&r).starts_with("o\n"));
    session.base().enqueue_message("hello").await;
    session.base().enqueue_message("world").await;
    let r = session.handle_request(false, &fmt).await;
    let t = String::from_utf8_lossy(&r);
    assert!(t.starts_with("a[") && t.contains("hello") && t.contains("world"));
}

#[tokio::test]
async fn streaming_session_prelude_and_recycle() {
    let codec: Arc<dyn SockJsMessageCodec> = Arc::new(JsonSockJsMessageCodec::new());
    let session = StreamingSockJsSession::new(
        "s1",
        Duration::from_secs(25),
        Duration::from_secs(5),
        50,
        Arc::clone(&codec),
    );
    let fmt =
        vernal_websocket::sockjs::frame::default_sockjs_frame_format::DefaultSockJsFrameFormat;
    let r = session.handle_request(true, &fmt).await;
    assert!(r.len() >= 2049);
    for i in 0..10 {
        session.base().enqueue_message(format!("msg-{i}")).await;
    }
    session.handle_request(false, &fmt).await;
    assert_eq!(session.byte_count().await, 0);
}

#[tokio::test]
async fn http_receiving_handler_post_ok() {
    let handler = HttpReceivingTransportHandler::xhr_send();
    handler.initialize(SockJsServiceConfig::new());
    let codec4: Arc<dyn SockJsMessageCodec> = Arc::new(JsonSockJsMessageCodec::new());
    let recv_session3 = PollingSockJsSession::new(
        "r1",
        Duration::from_secs(25),
        Duration::from_secs(5),
        Arc::clone(&codec4),
    );
    assert_eq!(
        handler
            .handle_post(br#"["a","b"]"#, recv_session3.base())
            .await
            .unwrap(),
        b"ok"
    );
}

#[tokio::test]
async fn http_receiving_handler_rejects_empty() {
    let handler = HttpReceivingTransportHandler::xhr_send();
    handler.initialize(SockJsServiceConfig::new());
    let codec3: Arc<dyn SockJsMessageCodec> = Arc::new(JsonSockJsMessageCodec::new());
    let recv_session2 = PollingSockJsSession::new(
        "r2",
        Duration::from_secs(25),
        Duration::from_secs(5),
        Arc::clone(&codec3),
    );
    assert!(
        handler
            .handle_post(b"", recv_session2.base())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn http_sending_handler_returns_open_frame() {
    let handler = HttpSendingTransportHandler::xhr_polling();
    handler.initialize(SockJsServiceConfig::new());
    let codec: Arc<dyn SockJsMessageCodec> = Arc::new(JsonSockJsMessageCodec::new());
    let session = PollingSockJsSession::new(
        "s1",
        Duration::from_secs(25),
        Duration::from_secs(5),
        Arc::clone(&codec),
    );
    let fmt =
        vernal_websocket::sockjs::frame::default_sockjs_frame_format::DefaultSockJsFrameFormat;
    assert!(String::from_utf8_lossy(&session.handle_request(true, &fmt).await).starts_with("o\n"));
}

#[tokio::test]
async fn default_handler_callbacks_do_not_error() {
    let handler = AbstractWebSocketHandler;
    let session = MemoryWebSocketSession::new("d1", None, HeaderMap::new());
    assert!(handler.on_open(session.as_ref()).await.is_ok());
    assert!(
        handler
            .on_message(session.as_ref(), WebSocketMessage::text("hi"))
            .await
            .is_ok()
    );
    assert!(
        handler
            .on_message(
                session.as_ref(),
                WebSocketMessage::binary(Bytes::from_static(b"bin"))
            )
            .await
            .is_ok()
    );
    handler
        .on_error(session.as_ref(), &WebSocketError::Closed)
        .await;
    handler
        .on_close(session.as_ref(), CloseStatus::normal())
        .await;
}

#[tokio::test]
async fn text_handler_rejects_binary() {
    let handler = TextWebSocketHandler::new();
    let session = MemoryWebSocketSession::new("t1", None, HeaderMap::new());
    handler
        .on_message(
            session.as_ref(),
            WebSocketMessage::binary(Bytes::from_static(b"bin")),
        )
        .await
        .unwrap();
    assert!(
        session
            .sent_messages()
            .await
            .iter()
            .any(|m| matches!(m, WebSocketMessage::Close(Some(s)) if s.code().as_u16() == 1008))
    );
}
