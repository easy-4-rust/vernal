//! BeanWrapperImpl — Bean 包装器实现。
use crate::bean_wrapper::BeanWrapper;
use std::any::{Any, TypeId};
use std::sync::Arc;

/// Bean 包装器实现。
#[derive(Clone, Debug)]
pub struct BeanWrapperImpl {
    instance: Arc<dyn Any + Send + Sync>,
}
impl BeanWrapperImpl {
    pub fn new(instance: Arc<dyn Any + Send + Sync>) -> Self { Self { instance } }
}
impl BeanWrapper for BeanWrapperImpl {
    fn get_wrapped_instance(&self) -> Arc<dyn Any + Send + Sync> { self.instance.clone() }
    fn get_wrapped_class(&self) -> TypeId { (*self.instance).type_id() }
    fn set_property_value(&self, _property_name: &str, _value: Box<dyn Any + Send + Sync>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
    fn get_property_value(&self, _property_name: &str) -> Option<Box<dyn Any + Send + Sync>> { None }
}
