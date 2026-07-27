//! 对应 Java 类：org.springframework.web.socket.server.support.WebSocketHandlerMapping
//!
//! 路径 → handler 注册表（对标 Spring `WebSocketHandlerMapping`）。

use std::sync::Arc;

use crate::WebSocketHandler;

/// WebSocket handler 映射条目。
#[derive(Clone)]
pub struct MappingEntry {
    /// 路径模式。
    pub pattern: String,
    /// handler。
    pub handler: Arc<dyn WebSocketHandler>,
}

/// 路径匹配映射，对标 Spring `WebSocketHandlerMapping`。
#[derive(Default)]
pub struct WebSocketHandlerMapping {
    entries: Vec<MappingEntry>,
}

impl WebSocketHandlerMapping {
    /// 创建空映射。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加映射。
    pub fn register(&mut self, pattern: impl Into<String>, handler: Arc<dyn WebSocketHandler>) {
        self.entries.push(MappingEntry {
            pattern: pattern.into(),
            handler,
        });
    }

    /// 按路径查找 handler。
    #[must_use]
    pub fn lookup(&self, path: &str) -> Option<Arc<dyn WebSocketHandler>> {
        self.entries
            .iter()
            .find(|entry| crate::registry::path_matches_public(&entry.pattern, path))
            .map(|entry| Arc::clone(&entry.handler))
    }

    /// 返回注册数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
