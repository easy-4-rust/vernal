//! 注解操作源 — 对标 `AnnotationJCacheOperationSource`。
use super::jcache_operation_source::{JCacheOperation, JCacheOperationSource};
use std::any::Any;

pub struct AnnotationJCacheOperationSource;
impl JCacheOperationSource for AnnotationJCacheOperationSource {
    fn get_jcache_operation(&self, _method: &dyn Any) -> Option<Box<dyn JCacheOperation>> {
        None
    }
}
