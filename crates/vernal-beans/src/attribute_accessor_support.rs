//! AttributeAccessorSupport — 属性访问器支持。
use std::any::Any;
use std::collections::HashMap;
use std::fmt;

/// 属性访问器支持。
#[derive(Debug, Default)]
pub struct AttributeAccessorSupport {
    attributes: HashMap<String, Box<dyn Any + Send + Sync>>,
}
impl AttributeAccessorSupport {
    pub fn new() -> Self { Self::default() }
    pub fn set_attribute(&mut self, name: impl Into<String>, value: Box<dyn Any + Send + Sync>) { self.attributes.insert(name.into(), value); }
    pub fn get_attribute(&self, name: &str) -> Option<&(dyn Any + Send + Sync)> { self.attributes.get(name).map(|v| v.as_ref()) }
    pub fn has_attribute(&self, name: &str) -> bool { self.attributes.contains_key(name) }
    pub fn remove_attribute(&mut self, name: &str) -> Option<Box<dyn Any + Send + Sync>> { self.attributes.remove(name) }
    pub fn attribute_names(&self) -> Vec<&str> { self.attributes.keys().map(|s| s.as_str()).collect() }
    pub fn is_empty(&self) -> bool { self.attributes.is_empty() }
    pub fn len(&self) -> usize { self.attributes.len() }
}
