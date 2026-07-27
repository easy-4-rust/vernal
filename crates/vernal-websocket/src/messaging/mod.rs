//! Messaging 子模块：SubProtocol 层与 STOMP bridge。

pub mod stomp_sub_protocol_error_handler;
pub mod stomp_sub_protocol_handler;
pub mod sub_protocol_error_handler;
pub mod sub_protocol_event;
pub mod sub_protocol_handler;
pub mod sub_protocol_websocket_handler;
pub mod websocket_annotation_method_message_handler;
pub mod websocket_stomp_client;

pub use stomp_sub_protocol_error_handler::{StompErrorMessage, StompSubProtocolErrorHandler};
pub use stomp_sub_protocol_handler::StompSubProtocolHandler;
pub use sub_protocol_error_handler::{SubProtocolErrorFuture, SubProtocolErrorHandler};
pub use sub_protocol_event::{
    SessionConnectEvent, SessionConnectedEvent, SessionDisconnectEvent, SessionSubscribeEvent,
    SessionUnsubscribeEvent, SubProtocolEvent,
};
pub use sub_protocol_handler::{SubProtocolFuture, SubProtocolHandler};
pub use sub_protocol_websocket_handler::{
    OutboundMessageDispatcher, SubProtocolStats, SubProtocolWebSocketHandler,
    SubProtocolWebSocketHandlerConfig, WebSocketSessionHolder,
};
pub use websocket_annotation_method_message_handler::{
    DestinationHandler, WebSocketAnnotationMethodMessageHandler,
};
pub use websocket_stomp_client::{
    StompSession, StompSessionState, StompSubscription, WebSocketStompClient, stomp_close_status,
};
