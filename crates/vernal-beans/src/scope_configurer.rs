use crate::{bean_scope::BeanScope, configurable_bean_factory::ConfigurableBeanFactory};
use std::collections::HashMap;
#[derive(Default)]
pub struct ScopeConfigurer {
    scopes: HashMap<String, Box<dyn BeanScope>>,
}
impl ScopeConfigurer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_scope(
        &mut self,
        name: impl Into<String>,
        scope: Box<dyn BeanScope>,
    ) -> Option<Box<dyn BeanScope>> {
        self.scopes.insert(name.into(), scope)
    }
    pub fn len(&self) -> usize {
        self.scopes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }
    pub fn configure(mut self, factory: &mut dyn ConfigurableBeanFactory) {
        for (name, scope) in self.scopes.drain() {
            factory.register_scope(&name, scope);
        }
    }
}
