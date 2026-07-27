//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.TransportHandler
//!
//! 处理 SockJS session URL（即 transport 特定请求）。

use std::{future::Future, pin::Pin, sync::Arc};

use crate::WebSocketHandler;
use crate::sockjs::sockjs_error::SockJsError;
use crate::sockjs::transport::{SockJsServiceConfig, TransportType, sockjs_session::SockJsSession};

/// transport handler 返回的 future。
pub type TransportHandleFuture = Pin<Box<dyn Future<Output = Result<(), SockJsError>> + Send>>;

/// transport handler SPI。
pub trait TransportHandler: Send + Sync {
    /// 用 SockJS 服务配置初始化。
    fn initialize(&self, config: SockJsServiceConfig);

    /// 返回 transport 类型。
    fn transport_type(&self) -> TransportType;

    /// 校验 session 类型是否与 transport 匹配。
    fn check_session_type(&self, session: &dyn SockJsSession) -> bool;

    /// 处理 transport 特定请求。
    fn handle_request(
        &self,
        handler: Arc<dyn WebSocketHandler>,
        session: Arc<dyn SockJsSession>,
    ) -> TransportHandleFuture;
}
