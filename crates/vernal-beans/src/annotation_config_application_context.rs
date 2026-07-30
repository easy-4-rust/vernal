//! AnnotationConfigApplicationContext — 注解配置应用上下文。
use crate::abstract_application_context::AbstractApplicationContext;
use crate::application_context::ApplicationContext;

/// 注解配置应用上下文。
#[derive(Clone, Debug, Default)]
pub struct AnnotationConfigApplicationContext {
    inner: AbstractApplicationContext,
    config_classes: Vec<String>,
}
impl AnnotationConfigApplicationContext {
    pub fn new() -> Self { Self { inner: AbstractApplicationContext::new("AnnotationConfigApplicationContext"), config_classes: Vec::new() } }
    pub fn register(&mut self, config_class: impl Into<String>) { self.config_classes.push(config_class.into()); }
    pub fn config_classes(&self) -> &[String] { &self.config_classes }
    pub fn refresh(&mut self) { self.inner.refresh(); }
}
impl ApplicationContext for AnnotationConfigApplicationContext {
    fn get_bean(&self, _name: &str) -> Result<std::sync::Arc<dyn std::any::Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> { self.inner.get_bean(_name) }
    fn get_display_name(&self) -> &str { self.inner.get_display_name() }
    fn get_startup_date(&self) -> u64 { self.inner.get_startup_date() }
    fn is_active(&self) -> bool { self.inner.is_active() }
}
