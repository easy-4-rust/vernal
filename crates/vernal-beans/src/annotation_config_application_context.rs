use crate::{
    annotation_metadata::AnnotationMetadata, generic_application_context::GenericApplicationContext,
};
use std::any::Any;
pub trait AnnotatedBeanClass: Any + Send + Sync {
    fn annotation_metadata(&self) -> &dyn AnnotationMetadata;
    fn bean_name(&self) -> String {
        std::any::type_name::<Self>()
            .rsplit("::")
            .next()
            .unwrap_or("bean")
            .to_owned()
    }
}
pub struct AnnotationConfigApplicationContext {
    context: GenericApplicationContext,
    registered: Vec<String>,
}
impl AnnotationConfigApplicationContext {
    pub fn new() -> Self {
        Self {
            context: GenericApplicationContext::with_id("annotationConfigApplicationContext"),
            registered: Vec::new(),
        }
    }
    pub fn register<T: AnnotatedBeanClass>(&mut self, bean: T) -> bool {
        if !bean.annotation_metadata().has_annotation("Component")
            && !bean.annotation_metadata().has_annotation("Configuration")
        {
            return false;
        }
        let name = bean.bean_name();
        self.context.register_bean(name.clone(), bean);
        self.registered.push(name);
        true
    }
    pub fn registered_bean_names(&self) -> &[String] {
        &self.registered
    }
    pub fn context(&self) -> &GenericApplicationContext {
        &self.context
    }
    pub fn context_mut(&mut self) -> &mut GenericApplicationContext {
        &mut self.context
    }
    pub fn into_context(self) -> GenericApplicationContext {
        self.context
    }
}
impl Default for AnnotationConfigApplicationContext {
    fn default() -> Self {
        Self::new()
    }
}
impl std::ops::Deref for AnnotationConfigApplicationContext {
    type Target = GenericApplicationContext;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
impl std::ops::DerefMut for AnnotationConfigApplicationContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}
