//! NamespaceHandlerImpl — 命名空间处理器实现。
use crate::namespace_handler::NamespaceHandler;

/// 命名空间处理器实现。
#[derive(Clone, Debug, Default)]
pub struct NamespaceHandlerImpl;
impl NamespaceHandlerImpl {
    pub fn new() -> Self { Self }
}
impl NamespaceHandler for NamespaceHandlerImpl {
    fn init(&self) {}
    fn parse(&self, _element: &str) -> Option<String> { None }
}
