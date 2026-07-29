//! ReplaceOverride — Spring 风格的替换方法覆盖。
use crate::method_override::MethodOverride;
use std::fmt;

/// Spring 风格的替换方法覆盖。
#[derive(Clone, Debug)]
pub struct ReplaceOverride {
    pub method_name: String,
    pub replacer_name: String,
}

impl ReplaceOverride {
    pub fn new(method_name: impl Into<String>, replacer_name: impl Into<String>) -> Self {
        Self { method_name: method_name.into(), replacer_name: replacer_name.into() }
    }
    pub fn get_method_name(&self) -> &str { &self.method_name }
    pub fn get_replacer_name(&self) -> &str { &self.replacer_name }
}

impl MethodOverride for ReplaceOverride {
    fn get_method_name(&self) -> &str { &self.method_name }
    fn is_applicable(&self) -> bool { true }
}
