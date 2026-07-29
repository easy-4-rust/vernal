//! MethodDescriptor — Spring 风格的方法描述符。
use std::fmt;

/// Spring 风格的方法描述符。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MethodDescriptor {
    name: String,
    return_type: String,
    parameter_types: Vec<String>,
}

impl MethodDescriptor {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), return_type: String::new(), parameter_types: Vec::new() }
    }
    pub fn with_return_type(mut self, rt: impl Into<String>) -> Self { self.return_type = rt.into(); self }
    pub fn with_parameters(mut self, params: Vec<String>) -> Self { self.parameter_types = params; self }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn get_return_type(&self) -> &str { &self.return_type }
    pub fn get_parameter_types(&self) -> &[String] { &self.parameter_types }
    pub fn parameter_count(&self) -> usize { self.parameter_types.len() }
}
