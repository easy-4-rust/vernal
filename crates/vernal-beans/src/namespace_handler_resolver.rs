//! NamespaceHandlerResolver — 命名空间处理器解析器。
use crate::namespace_handler::NamespaceHandler;

/// 命名空间处理器解析器 trait。
pub trait NamespaceHandlerResolver: Send + Sync {
    fn resolve(&self, namespace_uri: &str) -> Option<Box<dyn NamespaceHandler>>;
}
