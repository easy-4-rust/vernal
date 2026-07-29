//! StaticListableBeanFactory — Spring 风格的静态 Bean 工厂。
//! 对应 Java 类：`org.springframework.beans.factory.StaticListableBeanFactory`。
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use crate::bean_factory::BeanFactory;
use crate::component_key::ComponentKey;

/// Spring 风格的静态 Bean 工厂。
#[derive(Clone, Debug, Default)]
pub struct StaticListableBeanFactory {
    singletons: HashMap<String, Arc<dyn Any + Send + Sync>>,
}

impl StaticListableBeanFactory {
    pub fn new() -> Self { Self::default() }
    pub fn register_singleton(&mut self, name: impl Into<String>, bean: Arc<dyn Any + Send + Sync>) {
        self.singletons.insert(name.into(), bean);
    }
    pub fn contains_bean_name(&self, name: &str) -> bool { self.singletons.contains_key(name) }
    pub fn bean_names(&self) -> Vec<String> { self.singletons.keys().cloned().collect() }
}

impl BeanFactory for StaticListableBeanFactory {
    fn get_bean_by_key(&self, key: &ComponentKey) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        self.singletons.get(key.type_name()).cloned()
            .ok_or_else(|| format!("Bean '{}' not found", key).into())
    }
    fn get_bean_by_type_id(&self, type_id: std::any::TypeId) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let mut found = self.singletons.values()
            .filter(|v| (**v).type_id() == type_id);
        found.next().cloned().ok_or_else(|| "No bean found".into())
    }
    fn contains_bean(&self, key: &ComponentKey) -> bool { self.singletons.contains_key(key.type_name()) }
    fn is_singleton(&self, _key: &ComponentKey) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
    fn is_prototype(&self, _key: &ComponentKey) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(false) }
    fn get_type(&self, key: &ComponentKey) -> Result<Option<&'static str>, Box<dyn std::error::Error + Send + Sync>> { Ok(Some(key.type_name())) }
    fn get_aliases(&self, _key: &ComponentKey) -> Vec<ComponentKey> { vec![] }
    fn get_bean_provider_by_type_id(&self, _type_id: std::any::TypeId) -> Result<Box<dyn crate::object_provider::ObjectProvider<dyn Any + Send + Sync> + '_>, Box<dyn std::error::Error + Send + Sync>> { unimplemented!() }
    fn is_type_match(&self, _key: &ComponentKey, _type_id: std::any::TypeId) -> bool { false }
}
