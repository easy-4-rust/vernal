//! 资源加载器 trait。
//!
//! 对标 Spring `org.springframework.core.io.ResourceLoader`。
//!
//! # 注意
//!
//! vernal-context 中的 `ResourceLoader` 使用关联类型 + Box<dyn> 模式，
//! 因为单个 loader 难以在编译期返回不同具体类型。

use super::resource::Resource;
use std::io;

/// 资源加载器 trait。
///
/// 对应 Java: org.springframework.core.io.ResourceLoader
pub trait ResourceLoader: Send + Sync {
    /// 根据 location 加载资源。
    fn load(&self, location: &str) -> io::Result<Box<dyn Resource>>;
}

/// 简单资源加载器（默认 protocol resolver）。
///
/// 对应 Spring `DefaultResourceLoader`（简化版）
#[derive(Debug, Default, Clone, Copy)]
pub struct SimpleResourceLoader;

impl ResourceLoader for SimpleResourceLoader {
    fn load(&self, location: &str) -> io::Result<Box<dyn Resource>> {
        if let Some(path) = location.strip_prefix("file:") {
            let path = if let Some(rest) = path.strip_prefix("//") {
                // file://path -> /path
                format!("/{rest}")
            } else {
                path.to_string()
            };
            Ok(Box::new(FileSystemResource::new(path)))
        } else if let Some(content) = location.strip_prefix("classpath:") {
            Ok(Box::new(ClassPathResource::new(content)))
        } else {
            // 默认为文件系统路径
            Ok(Box::new(FileSystemResource::new(location)))
        }
    }
}

use super::class_path_resource::ClassPathResource;
use super::file_system_resource::FileSystemResource;
