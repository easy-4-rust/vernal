//! ParameterResolutionDelegate — Spring 风格参数解析委托。

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct ParameterResolutionDelegate {
    dependencies: Mutex<HashMap<String, TypeId>>,
    resolved_count: usize,
}

impl ParameterResolutionDelegate {
    pub fn new() -> Self {
        Self {
            dependencies: Mutex::new(HashMap::new()),
            resolved_count: 0,
        }
    }
    pub fn register_dependency(&self, name: String, type_id: TypeId) {
        self.dependencies.lock().unwrap().insert(name, type_id);
    }
    pub fn get_dependency(&self, name: &str) -> Option<TypeId> {
        self.dependencies.lock().unwrap().get(name).copied()
    }
    pub fn dependency_count(&self) -> usize {
        self.dependencies.lock().unwrap().len()
    }
    pub fn increment_resolved(&mut self) { self.resolved_count += 1; }
    pub fn resolved_count(&self) -> usize { self.resolved_count }
}
impl Default for ParameterResolutionDelegate { fn default() -> Self { Self::new() } }
