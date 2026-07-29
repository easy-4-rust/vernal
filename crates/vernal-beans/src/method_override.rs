//! MethodOverride — Spring 风格的方法覆盖 trait。
use std::fmt;

/// Spring 风格的方法覆盖 trait。
pub trait MethodOverride: Send + Sync + fmt::Debug {
    fn get_method_name(&self) -> &str;
    fn is_applicable(&self) -> bool;
}

/// 简单的方法覆盖实现。
#[derive(Debug, Clone)]
pub struct SimpleMethodOverride {
    pub method_name: String,
    pub is_applicable: bool,
}

impl SimpleMethodOverride {
    pub fn new(method_name: impl Into<String>) -> Self {
        Self { method_name: method_name.into(), is_applicable: true }
    }
}

impl MethodOverride for SimpleMethodOverride {
    fn get_method_name(&self) -> &str { &self.method_name }
    fn is_applicable(&self) -> bool { self.is_applicable }
}
