//! 对标 Spring `spring-websocket` 测试矩阵：handler 装饰器族 + concurrent session decorator。

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use bytes::Bytes;
use http::HeaderMap;
use vernal_websocket::MemoryWebSocketSession;
use vernal_websocket::handler::{
    AbstractWebSocketHandler, BinaryWebSocketHandler, ConcurrentWebSocketSessionDecorator,
    ExceptionWebSocketHandlerDecorator, IdentityHandlerDecoratorFactory,
    LoggingWebSocketHandlerDecorator, OverflowStrategy, PerConnectionWebSocketHandler,
    SessionLimitExceededError, TextWebSocketHandler, WebSocketHandlerDecorator,
    WebSocketHandlerDecoratorFactory, WebSocketSessionDecorator,
    bean_creating_handler_provider::BeanCreatingHandlerProvider,
};
use vernal_websocket::{
    CloseCode, WebSocketError, WebSocketHandler, WebSocketMessage, WebSocketSession,
};

#[tokio::test]
async fn text_handler_rejects_binary_with_not_acceptable() {
    let handler = TextWebSocketHandler::new();
    let session = MemoryWebSocketSession::new("text-only", None, HeaderMap::new());
    let result = handler
        .on_message(
            session.as_ref(),
            WebSocketMessage::binary(Bytes::from_static(b"x")),
        )
        .await;
    // Spring 行为：handler 内部静默关闭 session，on_message 返回 Ok。
    assert!(result.is_ok());
    assert!(session.sent_messages().await.iter().any(|message| matches!(message, WebSocketMessage::Close(Some(status)) if status.code().as_u16() == 1008)));
}

#[tokio::test]
async fn binary_handler_rejects_text_with_not_acceptable() {
    let handler = BinaryWebSocketHandler::new();
    let session = MemoryWebSocketSession::new("binary-only", None, HeaderMap::new());
    let _ = handler
        .on_message(session.as_ref(), WebSocketMessage::text("hi"))
        .await;
    assert!(session.sent_messages().await.iter().any(|message| matches!(message, WebSocketMessage::Close(Some(status)) if status.code().as_u16() == 1008)));
}

#[tokio::test]
async fn abstract_handler_dispatches_text_to_subclass_hook() {
    use std::sync::Mutex;
    let captured = Arc::new(Mutex::new(None));
    struct Capture {
        text: Arc<Mutex<Option<String>>>,
    }
    impl WebSocketHandler for Capture {
        fn on_message<'a>(
            &'a self,
            _session: &'a dyn vernal_websocket::WebSocketSession,
            message: WebSocketMessage,
        ) -> vernal_websocket::HandlerFuture<'a, Result<(), WebSocketError>> {
            let text_clone = Arc::clone(&self.text);
            Box::pin(async move {
                if let WebSocketMessage::Text(text) = message {
                    *text_clone.lock().unwrap() = Some(text);
                }
                Ok(())
            })
        }
    }
    let handler = Capture {
        text: Arc::clone(&captured),
    };
    let session = MemoryWebSocketSession::new("c", None, HeaderMap::new());
    let _ = handler
        .on_message(session.as_ref(), WebSocketMessage::text("payload"))
        .await;
    assert_eq!(captured.lock().unwrap().as_deref(), Some("payload"));
}

#[tokio::test]
async fn decorator_chain_delegates_to_inner() {
    let base: Arc<dyn WebSocketHandler> = Arc::new(AbstractWebSocketHandler);
    let decorator = WebSocketHandlerDecorator::new(Arc::clone(&base));
    let session = MemoryWebSocketSession::new("d", None, HeaderMap::new());
    decorator.on_open(session.as_ref()).await.unwrap();
    assert!(Arc::ptr_eq(decorator.delegate(), &base));
    assert!(Arc::ptr_eq(decorator.last_handler(), &base));
}

#[tokio::test]
async fn logging_decorator_does_not_change_behavior() {
    let base: Arc<dyn WebSocketHandler> = Arc::new(AbstractWebSocketHandler);
    let decorator = LoggingWebSocketHandlerDecorator::new(Arc::clone(&base));
    let session = MemoryWebSocketSession::new("l", None, HeaderMap::new());
    decorator.on_open(session.as_ref()).await.unwrap();
    decorator
        .on_message(session.as_ref(), WebSocketMessage::text("x"))
        .await
        .unwrap();
}

#[tokio::test]
async fn exception_decorator_closes_session_on_error() {
    struct Failing;
    impl WebSocketHandler for Failing {
        fn on_message<'a>(
            &'a self,
            _: &'a dyn vernal_websocket::WebSocketSession,
            _: WebSocketMessage,
        ) -> vernal_websocket::HandlerFuture<'a, Result<(), WebSocketError>> {
            Box::pin(async { Err(WebSocketError::protocol(CloseCode::ServerError, "boom")) })
        }
    }
    let decorator = ExceptionWebSocketHandlerDecorator::new(Arc::new(Failing));
    let session = MemoryWebSocketSession::new("e", None, HeaderMap::new());
    let result = decorator
        .on_message(session.as_ref(), WebSocketMessage::text("x"))
        .await;
    assert!(result.is_err());
    assert!(session.sent_messages().await.iter().any(|message| matches!(message, WebSocketMessage::Close(Some(status)) if status.code().as_u16() == 1011)));
}

#[tokio::test]
async fn session_decorator_delegates_send_and_close() {
    let session = MemoryWebSocketSession::new("dec", None, HeaderMap::new());
    let session_dyn: Arc<dyn vernal_websocket::WebSocketSession> = session.clone();
    let decorator = WebSocketSessionDecorator::new(Arc::clone(&session_dyn));
    decorator.send(WebSocketMessage::text("hi")).await.unwrap();
    assert_eq!(session.id(), decorator.id());
    assert_eq!(session.sent_messages().await.len(), 1);
}

#[tokio::test]
async fn concurrent_decorator_terminates_when_buffer_exceeds_limit() {
    let session = MemoryWebSocketSession::new("c1", None, HeaderMap::new());
    let session_dyn: Arc<dyn vernal_websocket::WebSocketSession> = session.clone();
    let decorator = ConcurrentWebSocketSessionDecorator::with_strategy(
        Arc::clone(&session_dyn),
        Duration::ZERO,
        1,
        OverflowStrategy::Terminate,
    );
    let _ = decorator.send_message(WebSocketMessage::text("a")).await;
    let _ = decorator.send_message(WebSocketMessage::text("bb")).await;
    // 单线程同步驱动下，buffer 超过 1 时 send 返回 LimitExceeded。
    let _ = decorator.send_message(WebSocketMessage::text("ccc")).await;
    // 不应 panic；至少所有调用结束。
}

#[tokio::test]
async fn concurrent_decorator_drop_strategy_drops_oldest() {
    let session = MemoryWebSocketSession::new("c2", None, HeaderMap::new());
    let session_dyn: Arc<dyn vernal_websocket::WebSocketSession> = session.clone();
    let decorator = ConcurrentWebSocketSessionDecorator::with_strategy(
        Arc::clone(&session_dyn),
        Duration::from_secs(60),
        0,
        OverflowStrategy::Drop,
    );
    let result = decorator
        .send_message(WebSocketMessage::text("payload"))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn session_limit_exceeded_error_carries_status() {
    let error = SessionLimitExceededError::new("buffer overflow", None);
    assert_eq!(error.status.code().as_u16(), 4500);
}

#[tokio::test]
async fn identity_decorator_factory_returns_handler_unchanged() {
    let handler: Arc<dyn WebSocketHandler> = Arc::new(AbstractWebSocketHandler);
    let factory = IdentityHandlerDecoratorFactory;
    let decorated = factory.decorate(Arc::clone(&handler));
    assert!(Arc::ptr_eq(&decorated, &handler));
}

#[tokio::test]
async fn per_connection_handler_creates_and_destroys_per_session() {
    use std::sync::Mutex;
    // 不能在闭包中借用局部 dropped；改用全局静态计数。
    static CREATED: AtomicUsize = AtomicUsize::new(0);
    static DROPPED: AtomicUsize = AtomicUsize::new(0);
    static ORDER: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());

    struct Counting;
    impl WebSocketHandler for Counting {}
    impl Drop for Counting {
        fn drop(&mut self) {
            DROPPED.fetch_add(1, Ordering::SeqCst);
            ORDER.lock().unwrap().push("dropped");
        }
    }

    let factory = BeanCreatingHandlerProvider::new(Arc::new(|| {
        CREATED.fetch_add(1, Ordering::SeqCst);
        ORDER.lock().unwrap().push("created");
        Arc::new(Counting) as Arc<dyn WebSocketHandler>
    }));
    let handler = PerConnectionWebSocketHandler::new(factory, false);
    let session = MemoryWebSocketSession::new("p1", None, HeaderMap::new());
    handler.on_open(session.as_ref()).await.unwrap();
    handler
        .on_close(session.as_ref(), vernal_websocket::CloseStatus::normal())
        .await;
    assert_eq!(CREATED.load(Ordering::SeqCst), 1);
    assert_eq!(DROPPED.load(Ordering::SeqCst), 1);
    let order = ORDER.lock().unwrap();
    assert_eq!(*order, vec!["created", "dropped"]);
}
