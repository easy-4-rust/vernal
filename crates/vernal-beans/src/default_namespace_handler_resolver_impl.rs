//! DefaultNamespaceHandlerResolverImpl — 默认命名空间处理器解析器实现。
use crate::namespace_handler::NamespaceHandler;
use crate::namespace_handler_resolver::NamespaceHandlerResolver;

/// 默认命名空间处理器解析器实现。
#[derive(Clone, Debug, Default)]
pub struct DefaultNamespaceHandlerResolverImpl;
impl DefaultNamespaceHandlerResolverImpl {
    pub fn new() -> Self { Self }
}
impl NamespaceHandlerResolver for DefaultNamespaceHandlerResolverImpl {
    fn resolve(&self, _namespace_uri: &str) -> Option<Box<dyn NamespaceHandler>> { None }
}
