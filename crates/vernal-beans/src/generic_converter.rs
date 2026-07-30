//! GenericConverter — 通用转换器。
use std::any::TypeId;

/// 通用转换器 trait。
pub trait GenericConverter: Send + Sync {
    fn convert(&self, source: Box<dyn std::any::Any + Send + Sync>, source_type: TypeId, target_type: TypeId) -> Result<Box<dyn std::any::Any + Send + Sync>, String>;
    fn matches(&self, source_type: TypeId, target_type: TypeId) -> bool;
}
