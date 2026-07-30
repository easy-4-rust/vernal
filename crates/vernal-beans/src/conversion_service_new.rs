use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 扩展转换服务。
#[derive(Default)]
pub struct ConversionServiceNew {
    converters: Arc<Mutex<HashMap<(TypeId, TypeId), Box<dyn Fn(Box<dyn std::any::Any + Send + Sync>) -> Result<Box<dyn std::any::Any + Send + Sync>, String> + Send + Sync>>>>,
}
impl ConversionServiceNew {
    pub fn new() -> Self { Self::default() }
    pub fn convert(&self, _value: Box<dyn Any + Send + Sync>, _target_type: TypeId) -> Result<Box<dyn Any + Send + Sync>, String> {
        Err("No converter found".to_string())
    }
}
