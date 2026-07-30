//! CustomAutowireConfigurer — Spring 风格自定义自动装配配置器。

use std::any::TypeId;
use std::collections::HashSet;
use std::sync::Mutex;

pub struct CustomAutowireConfigurer {
    custom_qualifiers: Mutex<HashSet<TypeId>>,
    required: Mutex<bool>,
}

impl CustomAutowireConfigurer {
    pub fn new() -> Self {
        Self {
            custom_qualifiers: Mutex::new(HashSet::new()),
            required: Mutex::new(true),
        }
    }
    pub fn add_custom_qualifier(&self, type_id: TypeId) {
        self.custom_qualifiers.lock().unwrap().insert(type_id);
    }
    pub fn custom_qualifier_count(&self) -> usize {
        self.custom_qualifiers.lock().unwrap().len()
    }
    pub fn set_required(&self, v: bool) {
        *self.required.lock().unwrap() = v;
    }
    pub fn is_required(&self) -> bool { *self.required.lock().unwrap() }
}
impl Default for CustomAutowireConfigurer { fn default() -> Self { Self::new() } }
