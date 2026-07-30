//! ContextLoader — 上下文加载器。
use crate::application_context::ApplicationContext;

/// 上下文加载器。
#[derive(Clone, Debug, Default)]
pub struct ContextLoader;
impl ContextLoader {
    pub fn new() -> Self { Self }
    pub fn load(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { Ok(()) }
}
