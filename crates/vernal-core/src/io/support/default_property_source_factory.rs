//! 默认属性源工厂。
//!
//! 对标 Spring `org.springframework.core.io.support.DefaultPropertySourceFactory`。

use std::io;

use crate::env::{MapPropertySource, PropertySource};
use crate::io::Resource;
use crate::properties_file::parse_properties;

use super::PropertySourceFactory;

/// 默认属性源工厂。
///
/// 对应 Java: org.springframework.core.io.support.DefaultPropertySourceFactory
///
/// Spring 语义：把 `.properties` 资源解析为属性源（对标
/// `ResourcePropertySource` 的默认行为）。
pub struct DefaultPropertySourceFactory;

impl PropertySourceFactory for DefaultPropertySourceFactory {
    fn create_property_source(
        &self,
        name: &str,
        resource: &dyn Resource,
    ) -> io::Result<Box<dyn PropertySource>> {
        let content = resource.read_string()?;
        let mut map = std::collections::HashMap::new();
        parse_properties(&content, &mut map);
        Ok(Box::new(MapPropertySource::new(name.to_string(), map)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_properties_resource() {
        // A 类（合同对齐）：对标 Spring 默认工厂解析 properties
        let factory = DefaultPropertySourceFactory;
        let resource = crate::io::ByteArrayResource::new(b"k=v\n".to_vec());
        let source = factory.create_property_source("config", &resource).unwrap();
        assert_eq!(source.name(), "config");
        assert_eq!(source.get_property("k").as_deref(), Some("v"));
    }

    #[test]
    fn invalid_utf8_returns_error() {
        // C 类（错误路径）
        let factory = DefaultPropertySourceFactory;
        let resource = crate::io::ByteArrayResource::new(vec![0xFF, 0x00]);
        assert!(factory.create_property_source("config", &resource).is_err());
    }
}
