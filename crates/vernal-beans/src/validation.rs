//! Validation — 验证系统。
use std::fmt;

/// 验证器 trait。
pub trait Validator: Send + Sync + fmt::Debug {
    fn validate(&self, value: &dyn std::any::Any) -> Result<(), Vec<String>>;
}

/// 验证结果。
#[derive(Clone, Debug)]
pub struct ValidationResult {
    pub errors: Vec<String>,
}
impl ValidationResult {
    pub fn new() -> Self { Self { errors: Vec::new() } }
    pub fn add_error(&mut self, error: impl Into<String>) { self.errors.push(error.into()); }
    pub fn is_valid(&self) -> bool { self.errors.is_empty() }
}
