use crate::generic_application_context::GenericApplicationContext;
use std::any::Any;
pub struct StaticApplicationContext {
    context: GenericApplicationContext,
}
impl StaticApplicationContext {
    pub fn new() -> Self {
        Self {
            context: GenericApplicationContext::with_id("staticApplicationContext"),
        }
    }
    pub fn register_singleton<T: Any + Send + Sync>(&mut self, name: impl Into<String>, bean: T) {
        self.context.register_bean(name, bean)
    }
    pub fn context(&self) -> &GenericApplicationContext {
        &self.context
    }
    pub fn context_mut(&mut self) -> &mut GenericApplicationContext {
        &mut self.context
    }
    pub fn into_context(self) -> GenericApplicationContext {
        self.context
    }
}
impl Default for StaticApplicationContext {
    fn default() -> Self {
        Self::new()
    }
}
impl std::ops::Deref for StaticApplicationContext {
    type Target = GenericApplicationContext;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
impl std::ops::DerefMut for StaticApplicationContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}
