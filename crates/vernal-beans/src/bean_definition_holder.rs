//! BeanDefinitionHolder — Spring 风格的 Bean 定义持有者。
//! 对应 Java 类：`org.springframework.beans.factory.config.BeanDefinitionHolder`。
use crate::bean_definition::BeanDefinition;
use std::sync::Arc;

/// Spring 风格的 Bean 定义持有者。
#[derive(Clone)]
pub struct BeanDefinitionHolder {
    pub bean_name: String,
    pub bean_definition: Arc<dyn BeanDefinition>,
}

impl BeanDefinitionHolder {
    pub fn new(bean_name: impl Into<String>, definition: Arc<dyn BeanDefinition>) -> Self {
        Self { bean_name: bean_name.into(), bean_definition: definition }
    }
    pub fn bean_name(&self) -> &str { &self.bean_name }
    pub fn bean_definition(&self) -> &dyn BeanDefinition { &*self.bean_definition }
}
