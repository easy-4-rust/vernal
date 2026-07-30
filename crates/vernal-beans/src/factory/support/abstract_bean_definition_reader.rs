//! AbstractBeanDefinitionReader — Spring 风格 Bean 定义读取器抽象基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AbstractBeanDefinitionReader`。

use std::collections::HashMap;
use std::sync::Mutex;

pub struct AbstractBeanDefinitionReader {
    bean_class_count: usize,
    resource_count: Mutex<HashMap<String, usize>>,
}

impl AbstractBeanDefinitionReader {
    pub fn new() -> Self {
        Self { bean_class_count: 0, resource_count: Mutex::new(HashMap::new()) }
    }
    pub fn increment_bean_class_count(&mut self) { self.bean_class_count += 1; }
    pub fn get_bean_class_count(&self) -> usize { self.bean_class_count }
    pub fn reset_bean_class_count(&mut self) { self.bean_class_count = 0; }
    pub fn record_resource(&self, resource: String, count: usize) {
        self.resource_count.lock().unwrap().insert(resource, count);
    }
    pub fn resource_count(&self, resource: &str) -> Option<usize> {
        self.resource_count.lock().unwrap().get(resource).copied()
    }
}
impl Default for AbstractBeanDefinitionReader { fn default() -> Self { Self::new() } }
