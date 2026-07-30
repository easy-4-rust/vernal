//! DefaultNamespaceHandlerResolver — 默认命名空间处理器解析器。
use crate::namespace_handler::NamespaceHandler;
use crate::namespace_handler_resolver::NamespaceHandlerResolver;

/// 默认命名空间处理器解析器。
#[derive(Clone, Debug, Default)]
pub struct DefaultNamespaceHandlerResolver;
impl DefaultNamespaceHandlerResolver {
    pub fn new() -> Self { Self }
}
impl NamespaceHandlerResolver for DefaultNamespaceHandlerResolver {
    fn resolve(&self, _namespace_uri: &str) -> Option<Box<dyn NamespaceHandler>> { None }
}
