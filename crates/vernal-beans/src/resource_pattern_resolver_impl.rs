//! ResourcePatternResolverImpl — 资源模式解析器实现。
use crate::resource::Resource;
use crate::resource_pattern_resolver::ResourcePatternResolver;

/// 资源模式解析器实现。
#[derive(Clone, Debug, Default)]
pub struct ResourcePatternResolverImpl;
impl ResourcePatternResolverImpl {
    pub fn new() -> Self { Self }
}
impl ResourcePatternResolver for ResourcePatternResolverImpl {
    fn get_resources(&self, _pattern: &str) -> Result<Vec<Box<dyn Resource>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Vec::new())
    }
}
