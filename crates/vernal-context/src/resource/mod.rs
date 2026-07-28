//! 资源加载模块。
//!
//! 对标 Spring `org.springframework.core.io` 包。

mod byte_array_resource;
mod class_path_resource;
mod file_system_resource;
mod resource;
mod resource_loader;

pub use byte_array_resource::ByteArrayResource;
pub use class_path_resource::ClassPathResource;
pub use file_system_resource::FileSystemResource;
pub use resource::{Resource, ResourceError};
pub use resource_loader::{ResourceLoader, SimpleResourceLoader};
