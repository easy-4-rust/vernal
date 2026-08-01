//! 属性源工厂契约。
//!
//! 对标 Spring `org.springframework.core.io.support.PropertySourceFactory`。

use std::io;

use crate::env::PropertySource;
use crate::io::Resource;

/// 属性源工厂契约。
///
/// 对应 Java: org.springframework.core.io.support.PropertySourceFactory
///
/// Spring 语义：把编码资源转换为 `PropertySource`（如 YAML/Properties 源）。
pub trait PropertySourceFactory: Send + Sync {
    /// 从资源创建命名属性源。
    ///
    /// # 错误
    ///
    /// 资源解析失败时返回 [`std::io::Error`]。
    fn create_property_source(
        &self,
        name: &str,
        resource: &dyn Resource,
    ) -> io::Result<Box<dyn PropertySource>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestFactory;

    impl PropertySourceFactory for TestFactory {
        fn create_property_source(
            &self,
            name: &str,
            resource: &dyn Resource,
        ) -> io::Result<Box<dyn PropertySource>> {
            let content = resource.read_string()?;
            let source = crate::env::MapPropertySource::new(
                name.to_string(),
                std::collections::HashMap::from([("raw".to_string(), content)]),
            );
            Ok(Box::new(source))
        }
    }

    #[test]
    fn factory_creates_named_source() {
        // A 类（合同对齐）：对标 Spring 工厂创建属性源
        let factory = TestFactory;
        let resource = crate::io::ByteArrayResource::new(b"content".to_vec());
        let source = factory
            .create_property_source("config", &resource)
            .unwrap();
        assert_eq!(source.name(), "config");
        assert_eq!(source.get_property("raw").as_deref(), Some("content"));
    }
}
