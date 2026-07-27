//! 对标 Spring `spring-websocket` 测试矩阵：握手拦截器链、handler mapping、握手 handler。

use std::sync::Arc;

use http::{HeaderMap, Method, Uri};
use vernal_websocket::HandshakeRequest;
use vernal_websocket::WebSocketError;
use vernal_websocket::server::{
    AbstractHandshakeHandler, HandshakeContext, HandshakeInterceptor, HandshakeInterceptorChain,
    HttpSession, HttpSessionHandshakeInterceptor, OriginHandshakeInterceptor,
    RequestUpgradeStrategy, WebSocketHandlerMapping, WebSocketHttpRequestHandler,
    handshake_failure_error::HandshakeFailureError,
};
use vernal_websocket::{WebSocketExtension, WebSocketHandler};
use vernal_websocket::{WebSocketHttpHeaders, WebSocketSession};

#[tokio::test]
async fn handshake_interceptor_chain_short_circuits_on_false() {
    struct Reject;
    impl HandshakeInterceptor for Reject {
        fn before_handshake<'a>(
            &'a self,
            _context: &'a mut HandshakeContext,
            _handler: &'a Arc<dyn WebSocketHandler>,
        ) -> vernal_websocket::server::BeforeHandshakeFuture<'a> {
            Box::pin(async { Ok(false) })
        }
    }
    let handler: Arc<dyn WebSocketHandler> = Arc::new(NullHandler);
    let chain = HandshakeInterceptorChain::new(vec![Arc::new(Reject)], Arc::clone(&handler));
    let mut ctx = HandshakeContext::new(Method::GET, Uri::from_static("/ws"), HeaderMap::new());
    let proceed = chain.apply_before_handshake(&mut ctx).await.unwrap();
    assert!(!proceed);
}

#[tokio::test]
async fn handshake_interceptor_chain_propagates_attributes_through_chain() {
    let mut http = HttpSession::new();
    http.id = Some("sid-123".to_owned());
    http.attributes.insert("user".into(), "alice".into());

    let interceptor = HttpSessionHandshakeInterceptor::new().with_http_session(http);
    let handler: Arc<dyn WebSocketHandler> = Arc::new(NullHandler);
    let chain = HandshakeInterceptorChain::new(vec![Arc::new(interceptor)], Arc::clone(&handler));
    let mut ctx = HandshakeContext::new(Method::GET, Uri::from_static("/ws"), HeaderMap::new());
    let proceed = chain.apply_before_handshake(&mut ctx).await.unwrap();
    assert!(proceed);
    assert_eq!(
        ctx.attributes.get("HTTP.SESSION.ID").map(String::as_str),
        Some("sid-123")
    );
    assert_eq!(
        ctx.attributes.get("user").map(String::as_str),
        Some("alice")
    );
}

#[tokio::test]
async fn origin_interceptor_rejects_disallowed_origin() {
    let interceptor = OriginHandshakeInterceptor::new()
        .with_allowed_origins(["https://allowed.test".to_string()]);
    let handler: Arc<dyn WebSocketHandler> = Arc::new(NullHandler);
    let mut headers = HeaderMap::new();
    headers.insert("origin", "https://evil.test".parse().unwrap());
    let mut ctx = HandshakeContext::new(Method::GET, Uri::from_static("/ws"), headers);
    let result = interceptor.before_handshake(&mut ctx, &handler).await;
    assert!(matches!(
        result,
        Err(WebSocketError::Handshake { status: 403, .. })
    ));
}

#[tokio::test]
async fn origin_interceptor_allows_missing_origin_by_default() {
    let interceptor = OriginHandshakeInterceptor::new();
    let handler: Arc<dyn WebSocketHandler> = Arc::new(NullHandler);
    let mut ctx = HandshakeContext::new(Method::GET, Uri::from_static("/ws"), HeaderMap::new());
    let result = interceptor.before_handshake(&mut ctx, &handler).await;
    assert!(result.unwrap());
}

#[test]
fn handler_mapping_supports_exact_prefix_and_wildcard() {
    let mut mapping = WebSocketHandlerMapping::new();
    mapping.register("/exact", Arc::new(NullHandler) as Arc<dyn WebSocketHandler>);
    mapping.register(
        "/chat/*",
        Arc::new(NullHandler) as Arc<dyn WebSocketHandler>,
    );
    assert!(mapping.lookup("/exact").is_some());
    assert!(mapping.lookup("/chat/room-1").is_some());
    assert!(mapping.lookup("/missing").is_none());
    assert_eq!(mapping.len(), 2);
}

struct StubUpgradeStrategy;
impl RequestUpgradeStrategy for StubUpgradeStrategy {
    fn supported_versions(&self) -> &'static [&'static str] {
        &["13"]
    }
    fn supported_extensions(&self) -> Vec<WebSocketExtension> {
        vec![WebSocketExtension::new("permessage-deflate", None).unwrap()]
    }
    fn upgrade(
        &self,
        _selected_protocol: Option<&str>,
        _selected_extensions: Vec<WebSocketExtension>,
        _handler: Arc<dyn WebSocketHandler>,
    ) -> Result<Arc<dyn WebSocketSession>, WebSocketError> {
        Err(HandshakeFailureError::new("no transport configured").into())
    }
}

#[tokio::test]
async fn abstract_handshake_handler_negotiates_subprotocol_and_extensions() {
    let handler = AbstractHandshakeHandler::new(Arc::new(StubUpgradeStrategy))
        .with_supported_protocols(["chat".to_string(), "v12.stomp".to_string()]);
    let mut headers = HeaderMap::new();
    headers.insert("upgrade", "websocket".parse().unwrap());
    headers.insert("connection", "keep-alive, Upgrade".parse().unwrap());
    headers.insert("sec-websocket-version", "13".parse().unwrap());
    headers.insert("sec-websocket-key", "key".parse().unwrap());
    headers.insert("sec-websocket-protocol", "v12.stomp, chat".parse().unwrap());
    headers.insert(
        "sec-websocket-extensions",
        "permessage-deflate; client_no_context_takeover"
            .parse()
            .unwrap(),
    );
    let request = HandshakeRequest::new(Method::GET, Uri::from_static("/ws"), headers);
    request.validate().unwrap();
    assert_eq!(
        handler.select_protocol(&request).as_deref(),
        Some("v12.stomp")
    );
    let extensions = handler.select_extensions(&request);
    assert_eq!(extensions.len(), 1);
    assert_eq!(extensions[0].name(), "permessage-deflate");
}

#[tokio::test]
async fn http_request_handler_invokes_interceptors_and_returns_session_failure() {
    let handler: Arc<dyn WebSocketHandler> = Arc::new(NullHandler);
    let http = WebSocketHttpRequestHandler::new(
        Arc::new(AbstractHandshakeHandler::new(Arc::new(StubUpgradeStrategy))),
        Vec::new(),
    );
    let mut ctx = HandshakeContext::new(Method::GET, Uri::from_static("/ws"), HeaderMap::new());
    let response = http
        .handle(&mut ctx, handler, Default::default())
        .await
        .unwrap();
    assert_eq!(response.status, 500);
    assert!(response.session.is_none());
    assert!(!response.body.is_empty());
}

#[test]
fn websocket_http_headers_round_trip_protocols_and_extensions() {
    use std::collections::BTreeMap;
    let mut headers = WebSocketHttpHeaders::new();
    headers.set_sec_websocket_protocol(&["chat", "v12.stomp"]);
    assert_eq!(
        headers.sec_websocket_protocol(),
        vec!["chat".to_string(), "v12.stomp".to_string()]
    );
    let ext = WebSocketExtension::new(
        "permessage-deflate",
        Some(BTreeMap::from([(
            "server_max_window_bits".to_string(),
            "10".to_string(),
        )])),
    )
    .unwrap();
    headers.set_sec_websocket_extensions(&[ext]).unwrap();
    let parsed = headers.sec_websocket_extensions().unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].name(), "permessage-deflate");
    assert_eq!(
        parsed[0]
            .parameters()
            .get("server_max_window_bits")
            .map(String::as_str),
        Some("10")
    );
}

struct NullHandler;
impl WebSocketHandler for NullHandler {}
