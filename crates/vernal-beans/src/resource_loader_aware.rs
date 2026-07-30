//! ResourceLoaderAware — 资源加载器感知 trait。
use crate::default_resource_loader::ResourceLoader;

/// 资源加载器感知 trait。
pub trait ResourceLoaderAware: Send + Sync {
    fn set_resource_loader(&mut self, loader: Box<dyn ResourceLoader>);
}
