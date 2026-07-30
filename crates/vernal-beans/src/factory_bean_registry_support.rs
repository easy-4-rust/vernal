//! FactoryBeanRegistrySupport — FactoryBean 注册支持。
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// FactoryBean 注册支持。
#[derive(Clone, Debug, Default)]
pub struct FactoryBeanRegistrySupport {
    factory_bean_types: Arc<Mutex<HashMap<TypeId, String>>>,
}
impl FactoryBeanRegistrySupport {
    pub fn new() -> Self { Self::default() }
    pub fn register_factory_bean_type(&self, type_id: TypeId, bean_name: String) {
        self.factory_bean_types.lock().unwrap().insert(type_id, bean_name);
    }
    pub fn get_factory_bean_type(&self, type_id: TypeId) -> Option<String> {
        self.factory_bean_types.lock().unwrap().get(&type_id).cloned()
    }
    pub fn is_factory_bean(&self, type_id: TypeId) -> bool {
        self.factory_bean_types.lock().unwrap().contains_key(&type_id)
    }
}
