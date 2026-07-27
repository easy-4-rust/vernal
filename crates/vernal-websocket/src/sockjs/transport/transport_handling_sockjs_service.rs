//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.TransportHandlingSockJsService
//!
//! 维护 transport handler 集合、session 注册表，并按 path 后缀分发请求。
//! 对标 Spring `TransportHandlingSockJsService` 的核心分发逻辑（不含 Servlet 集成）。

use std::{collections::HashMap, sync::Arc};

use crate::WebSocketHandler;
use crate::sockjs::sockjs_error::SockJsError;
use crate::sockjs::sockjs_service::{
    SockJsRequest, SockJsResponseWriter, SockJsService, SockJsServiceFuture,
};
use crate::sockjs::transport::{SockJsServiceConfig, TransportHandler, TransportType};

/// 按 transport 类型查找 handler 的服务实现。
pub struct TransportHandlingSockJsService {
    config: SockJsServiceConfig,
    handlers: HashMap<TransportType, Arc<dyn TransportHandler>>,
}

impl TransportHandlingSockJsService {
    /// 创建服务。
    #[must_use]
    pub fn new(config: SockJsServiceConfig, handlers: Vec<Arc<dyn TransportHandler>>) -> Self {
        let mut map = HashMap::new();
        let mut config_owned = config.clone();
        for handler in handlers {
            let transport_type = handler.transport_type();
            handler.initialize(config_owned.clone());
            config_owned = config_owned.clone();
            map.insert(transport_type, handler);
        }
        Self {
            config,
            handlers: map,
        }
    }

    /// 返回配置。
    #[must_use]
    pub fn config(&self) -> &SockJsServiceConfig {
        &self.config
    }

    /// 按 transport 类型查找 handler。
    #[must_use]
    pub fn handler(&self, transport_type: TransportType) -> Option<Arc<dyn TransportHandler>> {
        self.handlers.get(&transport_type).cloned()
    }
}

impl SockJsService for TransportHandlingSockJsService {
    fn handle_request(
        &self,
        _request: SockJsRequest,
        _response: Arc<dyn SockJsResponseWriter>,
        _handler: Arc<dyn WebSocketHandler>,
    ) -> SockJsServiceFuture {
        // 完整路径解析（session id/transport 提取）依赖具体 HTTP 框架适配；
        // 此处保留 SPI 形态，真实请求处理在 adapter 层完成。
        Box::pin(async {
            Err(SockJsError::new(
                "SockJS request dispatch requires HTTP adapter",
                None,
                None,
            ))
        })
    }
}
