//! ApplicationContextAware — 应用上下文感知 trait。
use crate::application_context::ApplicationContext;
use std::sync::Arc;

/// 应用上下文感知 trait。
pub trait ApplicationContextAware: Send + Sync {
    fn set_application_context(&mut self, context: Arc<dyn ApplicationContext>);
}
