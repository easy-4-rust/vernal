//! 默认资源加载器。
//!
//! 对标 Spring `org.springframework.core.io.DefaultResourceLoader`。

use std::io;
use std::path::PathBuf;

use super::class_path_resource::ClassPathResource;
use super::file_system_resource::FileSystemResource;
use super::protocol_resolver::ProtocolResolver;
use super::resource::Resource;

/// 默认资源加载器。
///
/// 对应 Java: org.springframework.core.io.DefaultResourceLoader
///
/// Spring 语义：按位置前缀选择资源类型——`classpath:` → `ClassPathResource`、
/// `file:` → `FileSystemResource`、其他 → 文件系统路径；并支持通过
/// [`ProtocolResolver`] 扩展自定义协议。
#[derive(Default)]
pub struct DefaultResourceLoader {
    protocol_resolvers: Vec<Box<dyn ProtocolResolver>>,
}

impl DefaultResourceLoader {
    /// 创建默认资源加载器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册自定义协议解析器（对标 Spring `addProtocolResolver`）。
    pub fn add_protocol_resolver(&mut self, resolver: Box<dyn ProtocolResolver>) {
        self.protocol_resolvers.push(resolver);
    }

    /// 按位置加载资源。
    ///
    /// # 错误
    ///
    /// 位置为空时返回 [`std::io::Error`]。
    pub fn get_resource(&self, location: &str) -> io::Result<Box<dyn Resource>> {
        if location.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "location must not be empty",
            ));
        }
        for resolver in &self.protocol_resolvers {
            if let Some(resource) = resolver.resolve(location) {
                return Ok(resource);
            }
        }
        if let Some(path) = location.strip_prefix("classpath:") {
            return Ok(Box::new(ClassPathResource::new(path.to_string())));
        }
        if let Some(path) = location.strip_prefix("file:") {
            return Ok(Box::new(FileSystemResource::new(PathBuf::from(path))));
        }
        Ok(Box::new(FileSystemResource::new(PathBuf::from(location))))
    }
}

impl super::ResourceLoader for DefaultResourceLoader {
    fn load(&self, location: &str) -> io::Result<Box<dyn Resource>> {
        self.get_resource(location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_classpath_resources() {
        // A 类（合同对齐）：对标 Spring `classpath:` 前缀
        let loader = DefaultResourceLoader::new();
        let resource = loader.get_resource("classpath:anything").unwrap();
        assert!(resource.description().contains("Classpath 资源"));
    }

    #[test]
    fn loads_file_resources() {
        // A 类（合同对齐）：对标 Spring `file:` 前缀
        let loader = DefaultResourceLoader::new();
        let resource = loader.get_resource("file:/tmp/nonexistent-xyz").unwrap();
        assert!(!resource.exists());
    }

    #[test]
    fn loads_plain_path_as_file() {
        // B 类（边界行为）：无前缀 → 文件系统
        let loader = DefaultResourceLoader::new();
        let resource = loader.get_resource("/tmp/nonexistent-xyz").unwrap();
        assert!(!resource.exists());
    }

    #[test]
    fn empty_location_returns_error() {
        // C 类（错误路径）
        let loader = DefaultResourceLoader::new();
        assert!(loader.get_resource("").is_err());
    }

    #[test]
    fn custom_protocol_resolver_wins() {
        // D 类（扩展点）：对标 Spring `addProtocolResolver`
        struct Custom;
        impl ProtocolResolver for Custom {
            fn resolve(&self, location: &str) -> Option<Box<dyn Resource>> {
                if location.starts_with("custom:") {
                    Some(Box::new(crate::io::ByteArrayResource::new(b"x".to_vec())))
                } else {
                    None
                }
            }
        }
        let mut loader = DefaultResourceLoader::new();
        loader.add_protocol_resolver(Box::new(Custom));
        let resource = loader.get_resource("custom:y").unwrap();
        assert_eq!(resource.read_bytes().unwrap(), b"x");
    }
}
