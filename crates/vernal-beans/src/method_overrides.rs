//! MethodOverrides — Spring 风格的方法覆盖集合。
use crate::method_override::MethodOverride;
use std::collections::HashMap;

/// Spring 风格的方法覆盖集合。
#[derive(Debug, Default)]
pub struct MethodOverrides {
    overrides: HashMap<String, Box<dyn MethodOverride>>,
}

impl MethodOverrides {
    pub fn new() -> Self { Self::default() }
    pub fn add(&mut self, name: impl Into<String>, override_obj: Box<dyn MethodOverride>) { self.overrides.insert(name.into(), override_obj); }
    pub fn is_empty(&self) -> bool { self.overrides.is_empty() }
    pub fn len(&self) -> usize { self.overrides.len() }
    pub fn get(&self, name: &str) -> Option<&(dyn MethodOverride + 'static)> { self.overrides.get(name).map(|b| b.as_ref()) }
}
