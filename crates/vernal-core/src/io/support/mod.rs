//! io/support 支持包。
//!
//! 对标 Spring `org.springframework.core.io.support` 包：属性加载工具、
//! 资源模式解析、工厂加载器与资源属性源等。

mod default_property_source_factory;
mod encoded_resource;
mod localized_resource_helper;
mod properties_loader_support;
mod properties_loader_utils;
mod property_source_factory;
mod resource_pattern_resolver;
mod resource_pattern_utils;
mod resource_property_source;
mod resource_region;
mod spring_factories_loader;

pub use default_property_source_factory::DefaultPropertySourceFactory;
pub use encoded_resource::EncodedResource;
pub use localized_resource_helper::LocalizedResourceHelper;
pub use properties_loader_support::PropertiesLoaderSupport;
pub use properties_loader_utils::PropertiesLoaderUtils;
pub use property_source_factory::PropertySourceFactory;
pub use resource_pattern_resolver::ResourcePatternResolver;
pub use resource_pattern_utils::ResourcePatternUtils;
pub use resource_property_source::ResourcePropertySource;
pub use resource_region::ResourceRegion;
pub use spring_factories_loader::SpringFactoriesLoader;
