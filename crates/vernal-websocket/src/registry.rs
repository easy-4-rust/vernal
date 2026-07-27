//! WebSocket 路由注册。

use std::sync::Arc;

use crate::WebSocketHandler;

/// 显式 WebSocket handler 注册表。
#[derive(Default)]
pub struct WebSocketRegistry {
    entries: Vec<(String, Arc<dyn WebSocketHandler>)>,
}

impl WebSocketRegistry {
    /// 创建空注册表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册路径与 handler。
    pub fn register(&mut self, path: impl Into<String>, handler: Arc<dyn WebSocketHandler>) {
        self.entries.push((path.into(), handler));
    }

    /// 按路径查找 handler。
    #[must_use]
    pub fn get(&self, path: &str) -> Option<Arc<dyn WebSocketHandler>> {
        self.entries
            .iter()
            .find(|(pattern, _)| path_matches(pattern, path))
            .map(|(_, handler)| Arc::clone(handler))
    }

    /// 返回注册数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 返回注册表是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// 公共路径匹配：精确匹配 / `/*` / `prefix/*`。
#[must_use]
pub fn path_matches_public(pattern: &str, path: &str) -> bool {
    path_matches(pattern, path)
}

fn path_matches(pattern: &str, path: &str) -> bool {
    if pattern == path || pattern == "/*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("/*") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    false
}
