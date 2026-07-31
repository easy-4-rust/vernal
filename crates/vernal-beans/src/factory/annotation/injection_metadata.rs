//! InjectionMetadata — Spring 风格的注入元数据。
use std::any::Any;

/// Spring 风格的注入元数据。
#[derive(Debug, Default)]
pub struct InjectionMetadata {
    pub elements: Vec<Box<dyn Any + Send + Sync>>,
}

impl InjectionMetadata {
    pub fn new() -> Self { Self::default() }
    pub fn add_element(&mut self, element: Box<dyn Any + Send + Sync>) { self.elements.push(element); }
    pub fn element_count(&self) -> usize { self.elements.len() }
    pub fn is_empty(&self) -> bool { self.elements.is_empty() }
    pub fn clear(&mut self) { self.elements.clear(); }
}
