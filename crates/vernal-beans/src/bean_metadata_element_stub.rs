//! BeanMetadataElement — Bean 元数据元素 trait。
use std::any::Any;

/// Bean 元数据元素 trait。
pub trait BeanMetadataElement: Send + Sync {
    fn get_source(&self) -> Option<&dyn Any> { None }
}
