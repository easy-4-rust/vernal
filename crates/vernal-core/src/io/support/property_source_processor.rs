//! 属性源处理器。
//!
//! 对标 Spring `org.springframework.core.io.support.PropertySourceProcessor`。

use std::collections::HashMap;
use std::io;

use crate::env::{ConfigurableEnvironment, PropertySource};
use crate::io::Resource;

use super::default_property_source_factory::DefaultPropertySourceFactory;
use super::property_source_descriptor::PropertySourceDescriptor;
use super::property_source_factory::PropertySourceFactory;
use crate::io::default_resource_loader::DefaultResourceLoader;

/// 属性源处理器。
///
/// 对应 Java: org.springframework.core.io.support.PropertySourceProcessor
///
/// Spring 语义：把 [`PropertySourceDescriptor`] 落地到环境——加载资源、
/// 按 `ignoreResourceNotFound` 容忍缺失、按名查找工厂（vernal 中为注册表，
/// 对标 Spring 的 `BeanFactory` 查找）并把属性源追加到环境末尾。
pub struct PropertySourceProcessor {
    resource_loader: DefaultResourceLoader,
    factories: HashMap<String, Box<dyn PropertySourceFactory>>,
}

impl PropertySourceProcessor {
    /// 创建处理器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            resource_loader: DefaultResourceLoader::new(),
            factories: HashMap::new(),
        }
    }

    /// 注册命名工厂（对标 Spring 按 `factoryBeanName` 从容器取工厂 Bean）。
    pub fn register_factory(&mut self, name: impl Into<String>, factory: Box<dyn PropertySourceFactory>) {
        self.factories.insert(name.into(), factory);
    }

    /// 处理描述符并把属性源追加到环境。
    ///
    /// 对标 Spring `processPropertySource(PropertySourceDescriptor)`：
    /// 1. 描述符未携带资源时按首个位置加载；
    /// 2. 资源缺失且 `ignoreResourceNotFound` 时静默跳过；
    /// 3. 资源缺失且未忽略时返回错误；
    /// 4. 工厂解析后创建属性源并 `addLast`。
    ///
    /// # 错误
    ///
    /// 资源加载、工厂查找或解析失败时返回 [`std::io::Error`]。
    pub fn process(
        &self,
        environment: &mut dyn ConfigurableEnvironment,
        descriptor: &PropertySourceDescriptor,
    ) -> io::Result<()> {
        let Some(resource) = descriptor.resource() else {
            let location = descriptor.locations().first().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "属性源缺少位置与资源")
            })?;
            let loaded = self.resource_loader.get_resource(location)?;
            if !loaded.exists() {
                return Self::handle_missing(descriptor, location);
            }
            // 解引用为 trait 对象后走统一处理路径
            return self.process_resource(environment, descriptor, &*loaded);
        };
        self.process_resource(environment, descriptor, resource)
    }

/// 缺失资源处置：按开关忽略或报错。
fn handle_missing(descriptor: &PropertySourceDescriptor, location: &str) -> io::Result<()> {
    if descriptor.ignore_resource_not_found() {
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("属性源资源未找到: {location}"),
    ))
}

    /// 通过工厂创建属性源并追加到环境末尾。
    fn process_resource(
        &self,
        environment: &mut dyn ConfigurableEnvironment,
        descriptor: &PropertySourceDescriptor,
        resource: &dyn Resource,
    ) -> io::Result<()> {
        let source: Box<dyn PropertySource> = match descriptor.factory_bean_name() {
            Some(factory_name) => {
                let factory = self.factories.get(factory_name).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("未注册的属性源工厂: {factory_name}"),
                    )
                })?;
                factory.create_property_source(descriptor.name(), resource)?
            }
            None => {
                DefaultPropertySourceFactory
                    .create_property_source(descriptor.name(), resource)?
            }
        };
        environment.property_sources().add_last(source);
        Ok(())
    }
}

impl Default for PropertySourceProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::{AbstractEnvironment, MapPropertySource, PropertyResolver};
    use crate::io::ByteArrayResource;

    fn descriptor_with_resource(ignore: bool) -> PropertySourceDescriptor {
        PropertySourceDescriptor::new(
            "config",
            vec!["classpath:config.properties".to_string()],
            Some(Box::new(ByteArrayResource::with_description(
                b"host=localhost\nport=8080\n".to_vec(),
                "config.properties",
            ))),
            None,
            ignore,
        )
    }

    #[test]
    fn adds_property_source_to_environment() {
        // A 类（合同对齐）：对标 Spring 处理流程——资源→属性源→环境
        let processor = PropertySourceProcessor::new();
        let mut environment = AbstractEnvironment::default();
        let descriptor = descriptor_with_resource(false);
        processor.process(&mut environment, &descriptor).unwrap();
        assert!(environment.property_sources().contains("config"));
        assert_eq!(environment.get_property("host"), Some("localhost".to_string()));
    }

    #[test]
    fn missing_resource_ignored_when_requested() {
        // B 类（边界行为）：ignoreResourceNotFound=true 时缺失资源被跳过
        let processor = PropertySourceProcessor::new();
        let mut environment = AbstractEnvironment::default();
        let descriptor = PropertySourceDescriptor::new(
            "missing",
            vec!["classpath:does-not-exist.properties".to_string()],
            None,
            None,
            true,
        );
        processor.process(&mut environment, &descriptor).unwrap();
        assert!(!environment.property_sources().contains("missing"));
    }

    #[test]
    fn missing_resource_errors_when_not_ignored() {
        // C 类（错误路径）：缺失且未忽略时返回 NotFound
        let processor = PropertySourceProcessor::new();
        let mut environment = AbstractEnvironment::default();
        let descriptor = PropertySourceDescriptor::new(
            "missing",
            vec!["classpath:does-not-exist.properties".to_string()],
            None,
            None,
            false,
        );
        let result = processor.process(&mut environment, &descriptor);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn uses_registered_factory_by_name() {
        // D 类（重构安全/生命周期）：工厂注册表按名解析
        struct UpperFactory;
        impl PropertySourceFactory for UpperFactory {
            fn create_property_source(
                &self,
                name: &str,
                resource: &dyn Resource,
            ) -> io::Result<Box<dyn PropertySource>> {
                let text = resource.read_string()?;
                let mut map = HashMap::new();
                map.insert(
                    "raw".to_string(),
                    text.trim().to_uppercase(),
                );
                Ok(Box::new(MapPropertySource::new(name.to_string(), map)))
            }
        }

        let mut processor = PropertySourceProcessor::new();
        processor.register_factory("upperFactory", Box::new(UpperFactory));
        let mut environment = AbstractEnvironment::default();
        let descriptor = PropertySourceDescriptor::new(
            "upper",
            vec!["classpath:upper.properties".to_string()],
            Some(Box::new(ByteArrayResource::new(b"abc".to_vec()))),
            Some("upperFactory".to_string()),
            false,
        );
        processor.process(&mut environment, &descriptor).unwrap();
        assert_eq!(environment.get_property("raw"), Some("ABC".to_string()));
    }

    #[test]
    fn unknown_factory_name_errors() {
        // C 类（错误路径）：未注册工厂名被拒绝
        let processor = PropertySourceProcessor::new();
        let mut environment = AbstractEnvironment::default();
        let descriptor = PropertySourceDescriptor::new(
            "upper",
            vec!["classpath:upper.properties".to_string()],
            Some(Box::new(ByteArrayResource::new(b"abc".to_vec()))),
            Some("nope".to_string()),
            false,
        );
        assert!(processor.process(&mut environment, &descriptor).is_err());
    }
}
