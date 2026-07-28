//! 资源加载器类加载助手 — 对标 `ResourceLoaderClassLoadHelper`。
use std::io::Read;

/// 资源加载器 trait。
pub trait ResourceLoader: Send + Sync {
    /// 获取资源。
    fn get_resource(&self, location: &str) -> Option<Vec<u8>>;
    /// 获取资源作为输入流。
    fn get_resource_as_stream(&self, location: &str) -> Option<Box<dyn Read + Send>>;
    /// 加载类。
    fn load_class(&self, name: &str) -> Result<Box<dyn std::any::Any + Send + Sync>, String>;
}

/// 基于文件系统的资源加载器。
pub struct FileSystemResourceLoader {
    base_path: std::path::PathBuf,
}

impl FileSystemResourceLoader {
    pub fn new(base_path: std::path::PathBuf) -> Self {
        Self { base_path }
    }
}

impl ResourceLoader for FileSystemResourceLoader {
    fn get_resource(&self, location: &str) -> Option<Vec<u8>> {
        let path = self.base_path.join(location);
        std::fs::read(path).ok()
    }
    fn get_resource_as_stream(&self, location: &str) -> Option<Box<dyn Read + Send>> {
        let path = self.base_path.join(location);
        std::fs::File::open(path)
            .ok()
            .map(|f| Box::new(f) as Box<dyn Read + Send>)
    }
    fn load_class(&self, name: &str) -> Result<Box<dyn std::any::Any + Send + Sync>, String> {
        Err(format!("类加载不支持：{name}"))
    }
}

/// 资源加载器类加载助手。
pub struct ResourceLoaderClassLoadHelper {
    resource_loader: Box<dyn ResourceLoader>,
}
impl ResourceLoaderClassLoadHelper {
    pub fn new(resource_loader: Box<dyn ResourceLoader>) -> Self {
        Self { resource_loader }
    }
    pub fn get_resource(&self, name: &str) -> Option<Vec<u8>> {
        self.resource_loader.get_resource(name)
    }
    pub fn load_class(&self, name: &str) -> Result<Box<dyn std::any::Any + Send + Sync>, String> {
        self.resource_loader.load_class(name)
    }
}
