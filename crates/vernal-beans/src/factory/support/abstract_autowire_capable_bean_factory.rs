//! AbstractAutowireCapableBeanFactory — Spring 风格自动装配 Bean 工厂抽象基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AbstractAutowireCapableBeanFactory`。

use std::any::TypeId;
use std::collections::HashSet;
use std::sync::Mutex;

pub struct AbstractAutowireCapableBeanFactory {
    ignored_dependency_types: Mutex<HashSet<TypeId>>,
}

impl AbstractAutowireCapableBeanFactory {
    pub fn new() -> Self { Self { ignored_dependency_types: Mutex::new(HashSet::new()) } }
    pub fn ignore_dependency_type(&self, type_id: TypeId) {
        self.ignored_dependency_types.lock().unwrap().insert(type_id);
    }
    pub fn unignore_dependency_type(&self, type_id: TypeId) {
        self.ignored_dependency_types.lock().unwrap().remove(&type_id);
    }
    pub fn is_dependency_ignored(&self, type_id: TypeId) -> bool {
        self.ignored_dependency_types.lock().unwrap().contains(&type_id)
    }
    pub fn ignored_count(&self) -> usize {
        self.ignored_dependency_types.lock().unwrap().len()
    }
}
impl Default for AbstractAutowireCapableBeanFactory { fn default() -> Self { Self::new() } }
