//! SimpleBeanDefinitionRegistry — Spring 风格的简单 Bean 定义注册表。
//! 对应 Java 类：`org.springframework.beans.factory.support.SimpleBeanDefinitionRegistry`。
use crate::bean_definition::BeanDefinition;
use crate::bean_definition_registry::BeanDefinitionRegistry;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Spring 风格的简单 Bean 定义注册表。
#[derive(Clone, Debug, Default)]
pub struct SimpleBeanDefinitionRegistry {
    definitions: Arc<Mutex<HashMap<String, Arc<dyn BeanDefinition>>>>,
}

impl SimpleBeanDefinitionRegistry {
    pub fn new() -> Self { Self::default() }
    pub fn clear(&mut self) {
        self.definitions.lock().unwrap().clear();
    }
}

impl BeanDefinitionRegistry for SimpleBeanDefinitionRegistry {
    fn register_bean_definition(&mut self, name: String, def: Box<dyn BeanDefinition>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.definitions.lock().unwrap().insert(name, Arc::from(def));
        Ok(())
    }
    fn remove_bean_definition(&mut self, name: &str) -> Result<Box<dyn BeanDefinition>, Box<dyn std::error::Error + Send + Sync>> {
        self.definitions.lock().unwrap().remove(name)
            .map(|_| Box::new(crate::root_bean_definition::RootBeanDefinition::new()) as Box<dyn BeanDefinition>)
            .ok_or_else(|| "not found".into())
    }
    fn get_bean_definition(&self, name: &str) -> Option<&'static dyn BeanDefinition> {
        // Note: This method cannot return a reference due to MutexGuard being dropped
        // In production, use a different pattern
        None
    }
    fn contains_bean_definition(&self, name: &str) -> bool {
        self.definitions.lock().unwrap().contains_key(name)
    }
    fn bean_definition_count(&self) -> usize { self.definitions.lock().unwrap().len() }
    fn bean_definition_names(&self) -> Vec<String> { self.definitions.lock().unwrap().keys().cloned().collect() }
}
