//! AutowiredMethodElement — Spring 风格的 @Autowired 方法元素。
use std::any::Any;
use std::sync::Arc;

/// Spring 风格的 @Autowired 方法元素。
#[derive(Clone, Debug)]
pub struct AutowiredMethodElement {
    pub method_name: String,
    pub parameter_type_ids: Vec<std::any::TypeId>,
}

impl AutowiredMethodElement {
    pub fn new(method_name: impl Into<String>, params: Vec<std::any::TypeId>) -> Self {
        Self { method_name: method_name.into(), parameter_type_ids: params }
    }
    pub fn get_method_name(&self) -> &str { &self.method_name }
    pub fn get_parameter_type_ids(&self) -> &[std::any::TypeId] { &self.parameter_type_ids }
    pub fn invoke(&self, _target: &mut dyn Any, _args: &[Arc<dyn Any + Send + Sync>]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { Ok(()) }
}
