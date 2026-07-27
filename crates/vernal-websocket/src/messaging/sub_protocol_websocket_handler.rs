//! 对应 Java 类：org.springframework.web.socket.messaging.SubProtocolWebSocketHandler
//!
//! 维护 WebSocket session 注册表；收到 WebSocket 消息时按 session 的协议
//! 选择对应 `SubProtocolHandler` 并把消息送入 `clientInboundChannel`；
//! 订阅 `clientOutboundChannel`，把出站 `Message` 编码后发回客户端。

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use vernal_messaging::{Message, MessageChannel, MessageHandler, SendFuture};

use crate::handler::HandlerFuture;
use crate::messaging::sub_protocol_handler::SubProtocolHandler;
use crate::{CloseStatus, WebSocketError, WebSocketHandler, WebSocketMessage, WebSocketSession};

/// SubProtocolWebSocketHandler 持有的会话条目。
#[derive(Clone)]
pub struct WebSocketSessionHolder {
    /// session。
    pub session: Arc<dyn WebSocketSession>,
    /// 该 session 协商的协议。
    pub protocol: Option<String>,
}

/// SubProtocolWebSocketHandler 配置。
pub struct SubProtocolWebSocketHandlerConfig {
    /// 入站通道（client → server）。
    pub client_inbound_channel: Arc<dyn MessageChannel>,
    /// 出站通道（server → client）。
    pub client_outbound_channel: Arc<dyn vernal_messaging::SubscribableChannel>,
    /// 注册的协议处理器列表。
    pub protocol_handlers: Vec<Arc<dyn SubProtocolHandler>>,
    /// 默认处理器（无协议匹配时使用）。
    pub default_protocol_handler: Option<Arc<dyn SubProtocolHandler>>,
}

/// SubProtocolWebSocketHandler。
pub struct SubProtocolWebSocketHandler {
    config: Arc<SubProtocolWebSocketHandlerConfig>,
    sessions: Mutex<HashMap<String, WebSocketSessionHolder>>,
    stats: Mutex<SubProtocolStats>,
}

/// 统计信息。对标 Spring `SubProtocolWebSocketHandler.Stats`。
#[derive(Debug, Default, Clone, Copy)]
pub struct SubProtocolStats {
    /// 累计 session 数。
    pub total_sessions: u64,
    /// 当前 WebSocket session 数。
    pub web_socket_sessions: u64,
    /// 因限制超限关闭的 session 数。
    pub limit_exceeded_sessions: u64,
    /// 因 transport 错误关闭的 session 数。
    pub transport_error_sessions: u64,
}

impl SubProtocolWebSocketHandler {
    /// 创建 handler。
    #[must_use]
    pub fn new(config: Arc<SubProtocolWebSocketHandlerConfig>) -> Self {
        Self {
            config,
            sessions: Mutex::new(HashMap::new()),
            stats: Mutex::new(SubProtocolStats::default()),
        }
    }

    /// 返回支持的子协议（汇总所有 handler）。
    pub fn supported_protocols(&self) -> Vec<String> {
        self.config
            .protocol_handlers
            .iter()
            .flat_map(|handler| handler.supported_protocols())
            .collect()
    }

    async fn find_protocol_handler(
        &self,
        holder: &WebSocketSessionHolder,
    ) -> Arc<dyn SubProtocolHandler> {
        if let Some(protocol) = &holder.protocol {
            for handler in &self.config.protocol_handlers {
                if handler
                    .supported_protocols()
                    .iter()
                    .any(|supported| supported == protocol)
                {
                    return Arc::clone(handler);
                }
            }
        }
        self.config
            .default_protocol_handler
            .clone()
            .unwrap_or_else(|| Arc::clone(&self.config.protocol_handlers[0]))
    }
}

impl WebSocketHandler for SubProtocolWebSocketHandler {
    fn on_open<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async move {
            let mut sessions = self.sessions.lock().await;
            sessions.insert(
                session.id().to_string(),
                WebSocketSessionHolder {
                    session: Arc::new(SessionRef::new(session)) as Arc<dyn WebSocketSession>,
                    protocol: session.accepted_protocol().map(str::to_owned),
                },
            );
            self.stats.lock().await.total_sessions += 1;
            self.stats.lock().await.web_socket_sessions += 1;
            let holder = sessions.get(session.id()).expect("just inserted").clone();
            drop(sessions);
            let handler = self.find_protocol_handler(&holder).await;
            handler
                .after_session_started(
                    holder.session.as_ref(),
                    Arc::clone(&self.config.client_inbound_channel),
                )
                .await
        })
    }

    fn on_message<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        message: WebSocketMessage,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async move {
            let sessions = self.sessions.lock().await;
            let Some(holder) = sessions.get(session.id()).cloned() else {
                return Err(WebSocketError::protocol(
                    crate::CloseCode::ProtocolError,
                    format!("Unknown session: {}", session.id()),
                ));
            };
            drop(sessions);
            let handler = self.find_protocol_handler(&holder).await;
            handler
                .handle_message_from_client(
                    holder.session.as_ref(),
                    message,
                    Arc::clone(&self.config.client_inbound_channel),
                )
                .await
        })
    }

    fn on_error<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        error: &'a WebSocketError,
    ) -> HandlerFuture<'a> {
        Box::pin(async move {
            tracing::warn!(session_id = %session.id(), error = %error, "WebSocket transport error");
            self.stats.lock().await.transport_error_sessions += 1;
        })
    }

    fn on_close<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        status: CloseStatus,
    ) -> HandlerFuture<'a> {
        Box::pin(async move {
            let removed = self.sessions.lock().await.remove(session.id());
            self.stats.lock().await.web_socket_sessions = self
                .stats
                .lock()
                .await
                .web_socket_sessions
                .saturating_sub(1);
            if let Some(holder) = removed {
                let handler = self.find_protocol_handler(&holder).await;
                let _ = handler
                    .after_session_ended(
                        holder.session.as_ref(),
                        status,
                        Arc::clone(&self.config.client_inbound_channel),
                    )
                    .await;
            }
        })
    }
}

/// 出站 MessageHandler：把 clientOutboundChannel 上的消息通过对应 SubProtocolHandler 发回客户端。
pub struct OutboundMessageDispatcher {
    config: Arc<SubProtocolWebSocketHandlerConfig>,
    sessions: Arc<Mutex<HashMap<String, WebSocketSessionHolder>>>,
}

impl OutboundMessageDispatcher {
    /// 创建 dispatcher。
    #[must_use]
    pub fn new(
        config: Arc<SubProtocolWebSocketHandlerConfig>,
        sessions: Arc<Mutex<HashMap<String, WebSocketSessionHolder>>>,
    ) -> Self {
        Self { config, sessions }
    }
}

impl MessageHandler for OutboundMessageDispatcher {
    fn handle_message<'a>(&'a self, message: Arc<dyn Message>) -> SendFuture<'a> {
        Box::pin(async move {
            let Some(session_id) = self
                .config
                .protocol_handlers
                .iter()
                .find_map(|handler| handler.resolve_session_id(message.as_ref()))
            else {
                return Ok(());
            };
            let sessions = self.sessions.lock().await;
            let Some(holder) = sessions.get(&session_id).cloned() else {
                return Ok(());
            };
            drop(sessions);
            let handler = self
                .config
                .protocol_handlers
                .iter()
                .find(|handler| {
                    holder.protocol.as_ref().map_or(false, |protocol| {
                        handler.supported_protocols().contains(protocol)
                    })
                })
                .cloned()
                .or_else(|| self.config.default_protocol_handler.clone());
            if let Some(handler) = handler {
                return handler
                    .handle_message_to_client(holder.session.as_ref(), message)
                    .await
                    .map_err(|err| vernal_messaging::MessageError {
                        message: err.to_string(),
                    });
            }
            Ok(())
        })
    }
}

/// 内部辅助：把 `&dyn WebSocketSession` 包装为 Arc 以便存入 holder。
struct SessionRef {
    id: String,
    protocol: Option<String>,
    headers: http::HeaderMap,
}

impl SessionRef {
    fn new(session: &dyn WebSocketSession) -> Self {
        Self {
            id: session.id().to_string(),
            protocol: session.accepted_protocol().map(str::to_owned),
            headers: http::HeaderMap::new(),
        }
    }
}

impl WebSocketSession for SessionRef {
    fn id(&self) -> &str {
        &self.id
    }
    fn uri(&self) -> Option<&http::Uri> {
        None
    }
    fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }
    fn state(&self) -> crate::SessionState {
        crate::SessionState::Open
    }
    fn accepted_protocol(&self) -> Option<&str> {
        self.protocol.as_deref()
    }
    fn send<'a>(
        &'a self,
        _message: WebSocketMessage,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async { Ok(()) })
    }
    fn close<'a>(&'a self, _status: CloseStatus) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async { Ok(()) })
    }
}
