//! bean_factory_initialization_code — 对应 Java 类：org.springframework.beans.factory.aot.BeanFactoryInitializationCode。

use std::collections::HashMap;
use std::sync::Mutex;

/// BeanFactoryInitializationCode 类型。
pub struct BeanFactoryInitializationCode {
    data: Mutex<HashMap<String, String>>,
}

impl BeanFactoryInitializationCode {
    /// 创建一个新的实例。
    pub fn new() -> Self { Self { data: Mutex::new(HashMap::new()) } }
    /// 注册条目。
    pub fn register(&self, key: String, value: String) {
        self.data.lock().unwrap().insert(key, value);
    }
    /// 获取指定条目。
    pub fn get(&self, key: &str) -> Option<String> {
        self.data.lock().unwrap().get(key).cloned()
    }
    /// 获取条目数量。
    pub fn count(&self) -> usize { self.data.lock().unwrap().len() }
    /// 获取缓存大小。
    pub fn cache_size(&self) -> usize { self.count() }
    /// 移除。
    pub fn clear(&self) { self.data.lock().unwrap().clear(); }
    /// 判断是否包含指定条目。
    pub fn contains(&self, key: &str) -> bool {
        self.data.lock().unwrap().contains_key(key)
    }
    /// 处理输入并返回结果。
    pub fn process(&self, input: String) -> String { input }
}
impl Default for BeanFactoryInitializationCode { fn default() -> Self { Self::new() } }
