use std::any::Any;
use std::collections::HashMap;
#[derive(Default)]
pub struct AttributeAccessorSupport {
    attributes: HashMap<String, Box<dyn Any + Send + Sync>>,
}
impl AttributeAccessorSupport {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_attribute<T: Any + Send + Sync>(
        &mut self,
        name: impl Into<String>,
        value: T,
    ) -> Option<Box<dyn Any + Send + Sync>> {
        self.attributes.insert(name.into(), Box::new(value))
    }
    pub fn get_attribute<T: Any>(&self, name: &str) -> Option<&T> {
        self.attributes.get(name)?.downcast_ref()
    }
    pub fn remove_attribute(&mut self, name: &str) -> Option<Box<dyn Any + Send + Sync>> {
        self.attributes.remove(name)
    }
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }
    pub fn attribute_names(&self) -> Vec<&str> {
        self.attributes.keys().map(String::as_str).collect()
    }
}
