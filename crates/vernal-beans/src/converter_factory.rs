//! ConverterFactory — 转换器工厂。
use std::any::TypeId;

/// 转换器工厂 trait。
pub trait ConverterFactory: Send + Sync {
    fn get_converter(&self, source_type: TypeId, target_type: TypeId) -> Option<Box<dyn Fn(Box<dyn std::any::Any + Send + Sync>) -> Result<Box<dyn std::any::Any + Send + Sync>, String>>>;
}
