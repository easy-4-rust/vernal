//! GenericApplicationContext — 通用应用上下文。
use crate::abstract_application_context::AbstractApplicationContext;
use crate::application_context::ApplicationContext;

/// 通用应用上下文。
#[derive(Clone, Debug, Default)]
pub struct GenericApplicationContext {
    inner: AbstractApplicationContext,
}
impl GenericApplicationContext {
    pub fn new() -> Self { Self { inner: AbstractApplicationContext::new("GenericApplicationContext") } }
    pub fn refresh(&mut self) { self.inner.refresh(); }
}
impl ApplicationContext for GenericApplicationContext {
    fn get_bean(&self, _name: &str) -> Result<std::sync::Arc<dyn std::any::Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> { self.inner.get_bean(_name) }
    fn get_display_name(&self) -> &str { self.inner.get_display_name() }
    fn get_startup_date(&self) -> u64 { self.inner.get_startup_date() }
    fn is_active(&self) -> bool { self.inner.is_active() }
}
