//! NamespaceHandlerResolver — 命名空间处理器解析器 trait。
//!
//! 对应 Java 类：
//! `org.springframework.beans.factory.xml.NamespaceHandlerResolver`。
//!
//! 根据命名空间 URI 查找并返回对应的 [`NamespaceHandler`] 实例。

use std::sync::Arc;

use crate::namespace_handler::NamespaceHandler;

/// Spring 风格的命名空间处理器解析器 trait。
///
/// 对应 Spring 的 `NamespaceHandlerResolver`。
pub trait NamespaceHandlerResolver: Send + Sync {
    /// 返回命名空间 URI 对应的处理器（共享引用）。
    ///
    /// 对应 Spring 的
    /// `NamespaceHandler resolve(String namespaceUri)`。
    fn resolve(&self, namespace_uri: &str) -> Option<Arc<dyn NamespaceHandler>>;

    /// 返回已知的所有命名空间 URI。
    fn namespace_uris(&self) -> Vec<String>;
}

impl std::fmt::Debug for dyn NamespaceHandlerResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NamespaceHandlerResolver")
            .finish_non_exhaustive()
    }
}
