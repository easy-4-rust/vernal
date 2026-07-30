//! PropertyAccessor — 属性访问器。
use std::fmt;

/// 属性访问器 trait。
pub trait PropertyAccessor: Send + Sync + fmt::Debug {
    fn get_property_type(&self) -> Option<std::any::TypeId>;
    fn get_value(&self) -> Option<Box<dyn std::any::Any + Send + Sync>>;
    fn set_value(&self, value: Box<dyn std::any::Any + Send + Sync>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn is_readable(&self) -> bool;
    fn is_writable(&self) -> bool;
}
