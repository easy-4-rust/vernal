//! DefaultResourceLoaderImpl — 默认资源加载器实现。
use crate::default_resource_loader::ResourceLoader;
use crate::resource::Resource;

/// 默认资源加载器实现。
#[derive(Clone, Debug, Default)]
pub struct DefaultResourceLoaderImpl;
impl DefaultResourceLoaderImpl {
    pub fn new() -> Self { Self }
}
impl ResourceLoader for DefaultResourceLoaderImpl {
    fn get_resource(&self, location: &str) -> Result<Box<dyn Resource>, Box<dyn std::error::Error + Send + Sync>> {
        if location.starts_with("classpath:") {
            Ok(Box::new(crate::classpath_resource::ClassPathResource::new(&location[9..])))
        } else if location.starts_with("file:") {
            Ok(Box::new(crate::filesystem_resource::FileSystemResource::new(&location[5..])))
        } else {
            Ok(Box::new(crate::abstract_resource::AbstractResource::new(location)))
        }
    }
}
