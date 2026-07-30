//! ConstructorResolver — 构造函数解析器。
use std::any::Any;
use std::sync::Arc;

/// 构造函数解析器。
#[derive(Clone, Debug, Default)]
pub struct ConstructorResolver;
impl ConstructorResolver {
    pub fn new() -> Self { Self }
    pub fn resolve_constructor_arguments(&self, _args: &[Arc<dyn Any + Send + Sync>]) -> Vec<Arc<dyn Any + Send + Sync>> {
        Vec::new()
    }
}
