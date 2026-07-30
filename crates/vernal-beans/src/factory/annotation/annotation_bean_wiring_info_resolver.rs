//! AnnotationBeanWiringInfoResolver — Spring 风格注解 Bean 装配信息解析器。

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct AnnotationBeanWiringInfoResolver {
    wiring_infos: Mutex<HashMap<TypeId, Vec<String>>>,
}

impl AnnotationBeanWiringInfoResolver {
    pub fn new() -> Self {
        Self { wiring_infos: Mutex::new(HashMap::new()) }
    }
    pub fn register_wiring_info(&self, type_id: TypeId, info: Vec<String>) {
        self.wiring_infos.lock().unwrap().insert(type_id, info);
    }
    pub fn get_wiring_info(&self, type_id: TypeId) -> Option<Vec<String>> {
        self.wiring_infos.lock().unwrap().get(&type_id).cloned()
    }
    pub fn registered_count(&self) -> usize {
        self.wiring_infos.lock().unwrap().len()
    }
}
impl Default for AnnotationBeanWiringInfoResolver { fn default() -> Self { Self::new() } }
