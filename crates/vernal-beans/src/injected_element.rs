//! InjectedElement — Spring 风格的注入元素基类。
use std::any::Any;
use std::sync::Arc;

/// Spring 风格的注入元素基类。
#[derive(Clone, Debug)]
pub struct InjectedElement {
    pub name: String,
    pub required: bool,
}

impl InjectedElement {
    pub fn new(name: impl Into<String>, required: bool) -> Self {
        Self { name: name.into(), required }
    }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn is_required(&self) -> bool { self.required }
    pub fn inject(&mut self, _target: &mut dyn Any) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { Ok(()) }
}
