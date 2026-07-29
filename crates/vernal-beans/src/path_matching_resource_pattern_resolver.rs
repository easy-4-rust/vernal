//! PathMatchingResourcePatternResolver — 路径匹配资源模式解析器。
use crate::resource::Resource;
use crate::resource_pattern_resolver::ResourcePatternResolver;

/// 路径匹配资源模式解析器。
#[derive(Clone, Debug, Default)]
pub struct PathMatchingResourcePatternResolver;
impl PathMatchingResourcePatternResolver {
    pub fn new() -> Self { Self }
}
impl ResourcePatternResolver for PathMatchingResourcePatternResolver {
    fn get_resources(&self, _location_pattern: &str) -> Result<Vec<Box<dyn Resource>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Vec::new())
    }
}
