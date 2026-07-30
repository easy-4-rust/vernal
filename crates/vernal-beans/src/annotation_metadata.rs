//! AnnotationMetadata — 注解元数据。
use std::any::Any;
use std::collections::HashMap;
use std::fmt;

/// 注解元数据。
#[derive(Clone, Debug)]
pub struct AnnotationMetadata {
    pub type_name: String,
    pub annotations: HashMap<String, HashMap<String, String>>,
}
impl AnnotationMetadata {
    pub fn new(type_name: impl Into<String>) -> Self {
        Self { type_name: type_name.into(), annotations: HashMap::new() }
    }
    pub fn type_name(&self) -> &str { &self.type_name }
    pub fn add_annotation(&mut self, name: impl Into<String>, attributes: HashMap<String, String>) {
        self.annotations.insert(name.into(), attributes);
    }
    pub fn get_annotation(&self, name: &str) -> Option<&HashMap<String, String>> {
        self.annotations.get(name)
    }
    pub fn has_annotation(&self, name: &str) -> bool {
        self.annotations.contains_key(name)
    }
}
