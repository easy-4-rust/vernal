//! 资源属性源。
//!
//! 对标 Spring `org.springframework.core.io.support.ResourcePropertySource`。

use std::io;

use crate::env::{MapPropertySource, PropertySource};
use crate::io::Resource;
use crate::properties_file::parse_properties;

/// 资源属性源。
///
/// 对应 Java: org.springframework.core.io.support.ResourcePropertySource
///
/// Spring 语义：`PropertySource` 的 `.properties` 资源形态——名称默认取
/// 资源文件名。
pub struct ResourcePropertySource;

impl ResourcePropertySource {
    /// 从资源创建属性源（名称取自文件名）。
    ///
    /// # 错误
    ///
    /// 资源读取失败时返回 [`std::io::Error`]。
    pub fn from_resource(resource: &dyn Resource) -> io::Result<Box<dyn PropertySource>> {
        let name = resource
            .filename()
            .map_or_else(|| "resource".to_string(), str::to_string);
        Self::from_resource_with_name(&name, resource)
    }

    /// 从资源创建属性源（显式名称）。
    ///
    /// # 错误
    ///
    /// 资源读取失败时返回 [`std::io::Error`]。
    pub fn from_resource_with_name(
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
    use crate::io::ByteArrayResource;

    #[test]
    fn name_defaults_to_filename() {
        // A 类（合同对齐）：对标 Spring 文件名命名
        let dir = std::env::temp_dir();
        let path = dir.join("vernal-rps-test.properties");
        std::fs::write(&path, b"k=v\n").unwrap();
        let resource = crate::io::PathResource::new(&path);
        let source = ResourcePropertySource::from_resource(&resource).unwrap();
        assert_eq!(source.name(), "vernal-rps-test.properties");
        assert_eq!(source.get_property("k").as_deref(), Some("v"));
    }

    #[test]
    fn explicit_name_wins() {
        // B 类（边界行为）
        let resource = ByteArrayResource::new(b"a=1\n".to_vec());
        let source = ResourcePropertySource::from_resource_with_name("custom", &resource).unwrap();
        assert_eq!(source.name(), "custom");
    }
}
