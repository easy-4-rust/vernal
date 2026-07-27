//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.SockJsSessionFactory
//!
//! 创建 SockJS session 的工厂。TransportHandler 通常兼任。

use std::{collections::BTreeMap, sync::Arc};

use crate::WebSocketHandler;
use crate::sockjs::transport::sockjs_session::SockJsSession;

/// SockJS session 工厂 SPI。
pub trait SockJsSessionFactory: Send + Sync {
    /// 创建新 session。
    fn create_session(
        &self,
        session_id: impl Into<String>,
        handler: Arc<dyn WebSocketHandler>,
        attributes: BTreeMap<String, String>,
    ) -> Arc<dyn SockJsSession>;
}
