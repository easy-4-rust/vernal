//! BeanRegistryAdapter — Spring 风格 Bean 注册表适配器。

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct BeanRegistryAdapter {
    beans: Mutex<HashMap<String, TypeId>>,
}

impl BeanRegistryAdapter {
    pub fn new() -> Self { Self { beans: Mutex::new(HashMap::new()) } }
    pub fn register(&self, name: String, type_id: TypeId) {
        self.beans.lock().unwrap().insert(name, type_id);
    }
    pub fn unregister(&self, name: &str) {
        self.beans.lock().unwrap().remove(name);
    }
    pub fn get_type_id(&self, name: &str) -> Option<TypeId> {
        self.beans.lock().unwrap().get(name).copied()
    }
    pub fn count(&self) -> usize {
        self.beans.lock().unwrap().len()
    }
    pub fn contains(&self, name: &str) -> bool {
        self.beans.lock().unwrap().contains_key(name)
    }
}
impl Default for BeanRegistryAdapter { fn default() -> Self { Self::new() } }
