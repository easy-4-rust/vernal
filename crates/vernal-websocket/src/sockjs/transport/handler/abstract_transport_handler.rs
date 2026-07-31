//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.handler.AbstractTransportHandler
//!
//! 提供 TransportHandler 的默认实现：持有 SockJsServiceConfig 引用。

use std::sync::Arc;

use crate::sockjs::transport::SockJsServiceConfig;

/// 共享配置持有者。
pub struct AbstractTransportHandler {
    config: Arc<std::sync::Mutex<Option<SockJsServiceConfig>>>,
}

impl AbstractTransportHandler {
    /// 创建 handler。
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// 设置配置。
    pub fn set_config(&self, config: SockJsServiceConfig) {
        *self.config.lock().expect("config mutex") = Some(config);
    }

    /// 读取配置克隆。
    pub fn config(&self) -> Option<SockJsServiceConfig> {
        self.config.lock().expect("config mutex").clone()
    }
}

impl Default for AbstractTransportHandler {
    fn default() -> Self {
        Self::new()
    }
}
