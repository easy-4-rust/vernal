//! ConverterRegistry — 转换器注册表。
use std::any::TypeId;
use std::collections::HashMap;

/// 转换器注册表。
#[derive(Default)]
pub struct ConverterRegistry {
    converters: HashMap<(TypeId, TypeId), String>,
}
impl ConverterRegistry {
    pub fn new() -> Self { Self::default() }
    pub fn register(&mut self, source: TypeId, target: TypeId, name: impl Into<String>) {
        self.converters.insert((source, target), name.into());
    }
    pub fn get(&self, source: TypeId, target: TypeId) -> Option<&str> {
        self.converters.get(&(source, target)).map(|s| s.as_str())
    }
}
