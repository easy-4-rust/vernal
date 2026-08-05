//! ManagedArray — Spring 风格管理数组。

use std::any::Any;
use std::sync::{Arc, Mutex};

/// ManagedArray 类型。
pub struct ManagedArray {
    items: Mutex<Vec<Arc<dyn Any + Send + Sync>>>,
}

impl ManagedArray {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
        }
    }
    /// 执行add操作。
    pub fn add(&self, item: Arc<dyn Any + Send + Sync>) {
        self.items.lock().unwrap().push(item);
    }
    /// 获取指定条目。
    pub fn get(&self, index: usize) -> Option<Arc<dyn Any + Send + Sync>> {
        self.items.lock().unwrap().get(index).map(Arc::clone)
    }
    /// 执行len操作。
    pub fn len(&self) -> usize {
        self.items.lock().unwrap().len()
    }
    /// 判断是否empty。
    pub fn is_empty(&self) -> bool {
        self.items.lock().unwrap().is_empty()
    }
}
impl Default for ManagedArray {
    fn default() -> Self {
        Self::new()
    }
}
