//! DefaultResourceLoader — 默认资源加载器。
use crate::resource::Resource;

/// 资源加载器 trait。
pub trait ResourceLoader: Send + Sync {
    fn get_resource(&self, location: &str) -> Result<Box<dyn Resource>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 默认资源加载器。
#[derive(Default)]
pub struct DefaultResourceLoader {
    pub protocol_resolvers: Vec<Box<dyn crate::protocol_resolver::ProtocolResolver>>,
}
impl DefaultResourceLoader {
    pub fn new() -> Self { Self::default() }
    pub fn add_protocol_resolver(&mut self, resolver: Box<dyn crate::protocol_resolver::ProtocolResolver>) {
        self.protocol_resolvers.push(resolver);
    }
}
impl ResourceLoader for DefaultResourceLoader {
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

impl std::fmt::Debug for DefaultResourceLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultResourceLoader")
            .field("resolvers", &self.protocol_resolvers.len())
            .finish()
    }
}
