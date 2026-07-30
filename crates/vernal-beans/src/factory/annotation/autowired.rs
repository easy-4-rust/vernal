//! Autowired — Spring 风格 @Autowired 注解标记。

use std::any::TypeId;
use std::collections::HashSet;
use std::sync::Mutex;

pub struct Autowired {
    required: bool,
    primary: bool,
    qualifier_types: Mutex<HashSet<TypeId>>,
}

impl Autowired {
    pub fn new() -> Self {
        Self { required: true, primary: false, qualifier_types: Mutex::new(HashSet::new()) }
    }
    pub fn required(&self) -> bool { self.required }
    pub fn is_primary(&self) -> bool { self.primary }
    pub fn set_primary(&mut self, v: bool) { self.primary = v; }
    pub fn set_required(&mut self, v: bool) { self.required = v; }
    pub fn add_qualifier_type(&self, type_id: TypeId) {
        self.qualifier_types.lock().unwrap().insert(type_id);
    }
    pub fn qualifier_count(&self) -> usize {
        self.qualifier_types.lock().unwrap().len()
    }
}
impl Default for Autowired { fn default() -> Self { Self::new() } }
