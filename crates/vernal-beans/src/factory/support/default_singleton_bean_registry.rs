//! DefaultSingletonBeanRegistry — Spring 风格默认单例注册表。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.DefaultSingletonBeanRegistry`。
//!
//! 实现三级 Singleton 缓存。

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct DefaultSingletonBeanRegistry {
    singletonObjects: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    earlySingletonObjects: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    singletonFactories: Mutex<HashMap<String, Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>>>,
    singletonsCurrentlyInCreation: Mutex<HashMap<String, bool>>,
}

impl DefaultSingletonBeanRegistry {
    pub fn new() -> Self {
        Self {
            singletonObjects: Mutex::new(HashMap::new()),
            earlySingletonObjects: Mutex::new(HashMap::new()),
            singletonFactories: Mutex::new(HashMap::new()),
            singletonsCurrentlyInCreation: Mutex::new(HashMap::new()),
        }
    }
    pub fn getSingleton(&self, name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        let map = self.singletonObjects.lock().unwrap();
        map.get(name).map(Arc::clone)
    }
    pub fn containsSingleton(&self, name: &str) -> bool {
        self.singletonObjects.lock().unwrap().contains_key(name)
    }
    pub fn getSingletonCount(&self) -> usize {
        self.singletonObjects.lock().unwrap().len()
    }
    pub fn getSingletonNames(&self) -> Vec<String> {
        self.singletonObjects.lock().unwrap().keys().cloned().collect()
    }
    pub fn registerSingleton(&self, name: String, obj: Arc<dyn Any + Send + Sync>) {
        self.singletonObjects.lock().unwrap().insert(name, obj);
    }
    pub fn addSingletonFactory(&self, name: String, factory: Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>) {
        self.singletonFactories.lock().unwrap().insert(name, factory);
    }
    pub fn getEarlyBeanReference(&self, name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        let mut factories = self.singletonFactories.lock().unwrap();
        if let Some(f) = factories.remove(name) {
            let bean = f();
            self.earlySingletonObjects.lock().unwrap().insert(name.to_string(), Arc::clone(&bean));
            return Some(bean);
        }
        None
    }
    pub fn markAsInCreation(&self, name: &str) {
        self.singletonsCurrentlyInCreation.lock().unwrap().insert(name.to_string(), true);
    }
    pub fn isCurrentlyInCreation(&self, name: &str) -> bool {
        self.singletonsCurrentlyInCreation.lock().unwrap().get(name).copied().unwrap_or(false)
    }
    pub fn destroySingletons(&self) {
        self.singletonObjects.lock().unwrap().clear();
        self.earlySingletonObjects.lock().unwrap().clear();
        self.singletonFactories.lock().unwrap().clear();
    }
}
impl Default for DefaultSingletonBeanRegistry { fn default() -> Self { Self::new() } }
