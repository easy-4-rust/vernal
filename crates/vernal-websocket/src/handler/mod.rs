//! Handler 装饰器与辅助类型。

pub mod abstract_websocket_handler;
pub mod bean_creating_handler_provider;
pub mod binary_websocket_handler;
pub mod concurrent_websocket_session_decorator;
pub mod exception_websocket_handler_decorator;
pub mod logging_websocket_handler_decorator;
pub mod per_connection_websocket_handler;
pub mod session_limit_exceeded_error;
pub mod text_websocket_handler;
pub mod websocket_handler;
pub mod websocket_handler_decorator;
pub mod websocket_handler_decorator_factory;
pub mod websocket_session_decorator;

pub use abstract_websocket_handler::{
    AbstractWebSocketHandler, MessageHandlerHooks, NOT_ACCEPTABLE_REASON_BINARY,
    NOT_ACCEPTABLE_REASON_TEXT, not_acceptable,
};
pub use bean_creating_handler_provider::{
    BeanCreatingHandlerProvider, HandlerCreationError, HandlerFactory,
};
pub use binary_websocket_handler::BinaryWebSocketHandler;
pub use concurrent_websocket_session_decorator::{
    ConcurrentWebSocketSessionDecorator, OverflowStrategy, session_not_reliable, to_close_status,
};
pub use exception_websocket_handler_decorator::ExceptionWebSocketHandlerDecorator;
pub use logging_websocket_handler_decorator::LoggingWebSocketHandlerDecorator;
pub use per_connection_websocket_handler::PerConnectionWebSocketHandler;
pub use session_limit_exceeded_error::SessionLimitExceededError;
pub use text_websocket_handler::TextWebSocketHandler;
pub use websocket_handler::{HandlerFuture, WebSocketHandler};
pub use websocket_handler_decorator::WebSocketHandlerDecorator;
pub use websocket_handler_decorator_factory::{
    IdentityHandlerDecoratorFactory, WebSocketHandlerDecoratorFactory,
};
pub use websocket_session_decorator::WebSocketSessionDecorator;
