//! AutowiredFieldElement — Spring 风格的 @Autowired 字段元素。
use std::any::Any;
use std::sync::Arc;

/// Spring 风格的 @Autowired 字段元素。
#[derive(Clone, Debug)]
pub struct AutowiredFieldElement {
    pub name: String,
    pub required: bool,
}

impl AutowiredFieldElement {
    pub fn new(name: impl Into<String>, required: bool) -> Self {
        Self { name: name.into(), required }
    }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn is_required(&self) -> bool { self.required }
    pub fn resolve(&self) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> { Ok(None) }
}
