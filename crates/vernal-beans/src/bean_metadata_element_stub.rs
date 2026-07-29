use std::any::Any;
pub trait BeanMetadataElement: Send + Sync {
    fn get_source(&self) -> Option<&(dyn Any + Send + Sync)>;
}
