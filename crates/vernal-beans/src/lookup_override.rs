//! LookupOverride — Spring 风格的 lookup 方法覆盖。
use crate::method_override::MethodOverride;
use std::fmt;

/// Spring 风格的 lookup 方法覆盖。
#[derive(Clone, Debug)]
pub struct LookupOverride {
    pub method_name: String,
    pub bean_name: String,
}

impl LookupOverride {
    pub fn new(method_name: impl Into<String>, bean_name: impl Into<String>) -> Self {
        Self { method_name: method_name.into(), bean_name: bean_name.into() }
    }
    pub fn get_method_name(&self) -> &str { &self.method_name }
    pub fn get_bean_name(&self) -> &str { &self.bean_name }
}

impl MethodOverride for LookupOverride {
    fn get_method_name(&self) -> &str { &self.method_name }
    fn is_applicable(&self) -> bool { true }
}
