use crate::{
    application_context::ApplicationContext, bean_factory_post_processor::BeanFactoryPostProcessor,
};
use std::sync::Arc;

pub trait ConfigurableApplicationContext: ApplicationContext {
    fn add_bean_factory_post_processor(&mut self, processor: Arc<dyn BeanFactoryPostProcessor>);
    fn remove_bean_factory_post_processor(
        &mut self,
        index: usize,
    ) -> Option<Arc<dyn BeanFactoryPostProcessor>>;
    fn bean_factory_post_processor_count(&self) -> usize;
    fn refresh(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn close(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
