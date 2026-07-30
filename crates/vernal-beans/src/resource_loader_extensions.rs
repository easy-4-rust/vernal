//! ResourceLoaderExtensions — 资源加载器扩展。
use crate::default_resource_loader::ResourceLoader;

/// 资源加载器扩展 trait。
pub trait ResourceLoaderExtensions: ResourceLoader {
    fn get_all_resources(&self, _pattern: &str) -> Vec<Box<dyn crate::resource::Resource>> { Vec::new() }
}
