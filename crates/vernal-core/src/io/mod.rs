//! 资源加载模块。
//!
//! 对标 Spring `org.springframework.core.io` 包。

mod abstract_file_resolving_resource;
mod abstract_resource;
mod byte_array_resource;
mod class_path_resource;
mod class_relative_resource_loader;
mod context_resource;
mod default_resource_loader;
mod descriptive_resource;
mod file_system_resource;
mod file_system_resource_loader;
mod file_url_resource;
mod input_stream_resource;
mod input_stream_source;
mod path_resource;
mod protocol_resolver;
mod resource;
mod resource_loader;
pub mod support;
mod writable_resource;

#[cfg(feature = "convert-url")]
mod url_resource;

#[cfg(feature = "convert-url")]
pub use url_resource::UrlResource;

pub use abstract_file_resolving_resource::AbstractFileResolvingResource;
pub use abstract_resource::AbstractResource;
pub use byte_array_resource::ByteArrayResource;
pub use class_path_resource::ClassPathResource;
pub use class_relative_resource_loader::ClassRelativeResourceLoader;
pub use context_resource::ContextResource;
pub use default_resource_loader::DefaultResourceLoader;
pub use descriptive_resource::DescriptiveResource;
pub use file_system_resource::FileSystemResource;
pub use file_system_resource_loader::FileSystemResourceLoader;
pub use file_url_resource::FileUrlResource;
pub use input_stream_resource::InputStreamResource;
pub use input_stream_source::InputStreamSource;
pub use path_resource::PathResource;
pub use protocol_resolver::ProtocolResolver;
pub use resource::{Resource, ResourceError};
pub use resource_loader::{ResourceLoader, SimpleResourceLoader};
pub use writable_resource::WritableResource;
