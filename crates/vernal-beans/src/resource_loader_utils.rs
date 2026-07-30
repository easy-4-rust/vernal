//! ResourceLoaderUtils — 资源加载器工具。
use crate::resource::Resource;

/// 资源加载器工具。
pub struct ResourceLoaderUtils;
impl ResourceLoaderUtils {
    pub fn get_url_as_string(url: &str) -> String { url.to_string() }
    pub fn is_url(location: &str) -> bool { location.contains("://") }
}
