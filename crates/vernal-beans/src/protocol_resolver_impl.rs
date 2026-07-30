//! ProtocolResolverImpl — 协议解析器实现。
use crate::resource::Resource;

/// 协议解析器 trait。
pub trait ProtocolResolver: Send + Sync {
    fn resolve(&self, protocol: &str, location: &str) -> Option<Box<dyn Resource>>;
}

/// 闭包协议解析器。
pub struct ClosureProtocolResolver {
    pub resolver_fn: Box<dyn Fn(&str, &str) -> Option<Box<dyn Resource>> + Send + Sync>,
}
impl ClosureProtocolResolver {
    pub fn new(f: Box<dyn Fn(&str, &str) -> Option<Box<dyn Resource>> + Send + Sync>) -> Self { Self { resolver_fn: f } }
}
impl ProtocolResolver for ClosureProtocolResolver {
    fn resolve(&self, protocol: &str, location: &str) -> Option<Box<dyn Resource>> {
        (self.resolver_fn)(protocol, location)
    }
}
