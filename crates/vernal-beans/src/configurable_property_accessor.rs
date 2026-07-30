//! ConfigurablePropertyAccessor — 可配置属性访问器。
use crate::property_accessor::PropertyAccessor;

/// 可配置属性访问器 trait。
pub trait ConfigurablePropertyAccessor: PropertyAccessor {
    fn set_property_name(&mut self, name: &str);
    fn get_property_name(&self) -> &str;
}
