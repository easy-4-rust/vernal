//! ResourcePatternResolver — 资源模式解析器。
use crate::resource::Resource;

/// 资源模式解析器 trait。
pub trait ResourcePatternResolver: Send + Sync {
    fn get_resources(&self, location_pattern: &str) -> Result<Vec<Box<dyn Resource>>, Box<dyn std::error::Error + Send + Sync>>;
}
