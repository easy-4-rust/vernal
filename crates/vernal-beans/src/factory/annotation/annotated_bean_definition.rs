//! AnnotatedBeanDefinition — Spring 风格注解 Bean 定义接口。

use std::any::TypeId;

pub trait AnnotatedBeanDefinition: Send + Sync {
    fn get_metadata(&self) -> Option<BeanMetadata>;
    fn is_factory_method(&self, name: &str) -> bool;
    fn get_factory_method_name(&self) -> Option<String>;
    fn get_bean_type(&self) -> TypeId;
}

#[derive(Debug, Clone)]
pub struct BeanMetadata {
    pub class_name: String,
    pub annotation_count: usize,
}

impl BeanMetadata {
    pub fn new(class_name: String, annotation_count: usize) -> Self {
        Self { class_name, annotation_count }
    }
}

pub struct GenericAnnotatedBeanDefinition {
    bean_class_name: String,
    factory_method: Option<String>,
    metadata: BeanMetadata,
}

impl GenericAnnotatedBeanDefinition {
    pub fn new(bean_class_name: String) -> Self {
        Self {
            bean_class_name: bean_class_name.clone(),
            factory_method: None,
            metadata: BeanMetadata::new(bean_class_name, 0),
        }
    }
    pub fn with_factory_method(mut self, name: String) -> Self {
        self.factory_method = Some(name);
        self
    }
    pub fn with_annotation_count(mut self, count: usize) -> Self {
        self.metadata.annotation_count = count;
        self
    }
    pub fn bean_class_name(&self) -> &str { &self.bean_class_name }
}

impl AnnotatedBeanDefinition for GenericAnnotatedBeanDefinition {
    fn get_metadata(&self) -> Option<BeanMetadata> { Some(self.metadata.clone()) }
    fn is_factory_method(&self, name: &str) -> bool {
        self.factory_method.as_deref() == Some(name)
    }
    fn get_factory_method_name(&self) -> Option<String> { self.factory_method.clone() }
    fn get_bean_type(&self) -> TypeId { TypeId::of::<Self>() }
}
