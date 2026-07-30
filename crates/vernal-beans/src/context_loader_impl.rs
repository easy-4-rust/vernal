//! ContextLoaderImpl — 上下文加载器实现。
use crate::context_loader::ContextLoader;

/// 上下文加载器实现。
#[derive(Clone, Debug, Default)]
pub struct ContextLoaderImpl;
impl ContextLoaderImpl {
    pub fn new() -> Self { Self }
    pub fn load(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { Ok(()) }
}
