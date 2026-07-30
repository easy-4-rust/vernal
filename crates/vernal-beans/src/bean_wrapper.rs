//! BeanWrapper — Bean 包装器。
use std::any::Any;
use std::sync::Arc;

/// Bean 包装器 trait。
pub trait BeanWrapper: Send + Sync {
    fn get_wrapped_instance(&self) -> Arc<dyn Any + Send + Sync>;
    fn get_wrapped_class(&self) -> std::any::TypeId;
    fn set_property_value(&self, property_name: &str, value: Box<dyn Any + Send + Sync>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn get_property_value(&self, property_name: &str) -> Option<Box<dyn Any + Send + Sync>>;
}
