//! 属性源描述符。
//!
//! 对标 Spring `org.springframework.core.io.support.PropertySourceDescriptor`。

use crate::io::Resource;

/// 属性源描述符（值对象）。
///
/// 对应 Java: org.springframework.core.io.support.PropertySourceDescriptor
///
/// Spring 语义：`@PropertySource` 注解的处理结果载体——记录属性源名称、
/// 位置列表、已解析资源、工厂 Bean 名与“资源缺失时忽略”开关。
/// vernal 中以不可变 struct 表达该 record。
pub struct PropertySourceDescriptor {
    name: String,
    locations: Vec<String>,
    resource: Option<Box<dyn Resource>>,
    factory_bean_name: Option<String>,
    ignore_resource_not_found: bool,
}

impl PropertySourceDescriptor {
    /// 创建描述符。
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        locations: Vec<String>,
        resource: Option<Box<dyn Resource>>,
        factory_bean_name: Option<String>,
        ignore_resource_not_found: bool,
    ) -> Self {
        Self {
            name: name.into(),
            locations,
            resource,
            factory_bean_name,
            ignore_resource_not_found,
        }
    }

    /// 属性源名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 位置列表（对标 record 的 `locations` 组件）。
    #[must_use]
    pub fn locations(&self) -> &[String] {
        &self.locations
    }

    /// 已解析资源（可能为空，需由处理器加载）。
    #[must_use]
    pub fn resource(&self) -> Option<&dyn Resource> {
        self.resource.as_deref()
    }

    /// 工厂 Bean 名（vernal 中对应注册表键）。
    #[must_use]
    pub fn factory_bean_name(&self) -> Option<&str> {
        self.factory_bean_name.as_deref()
    }

    /// 资源缺失时是否忽略。
    #[must_use]
    pub fn ignore_resource_not_found(&self) -> bool {
        self.ignore_resource_not_found
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::ByteArrayResource;

    #[test]
    fn exposes_record_components() {
        // A 类（合同对齐）：对标 Spring record 组件访问
        let descriptor = PropertySourceDescriptor::new(
            "db",
            vec!["classpath:db.properties".to_string()],
            Some(Box::new(ByteArrayResource::new(b"k=v".to_vec()))),
            Some("customFactory".to_string()),
            true,
        );
        assert_eq!(descriptor.name(), "db");
        assert_eq!(descriptor.locations(), &["classpath:db.properties"]);
        assert!(descriptor.resource().is_some());
        assert_eq!(descriptor.factory_bean_name(), Some("customFactory"));
        assert!(descriptor.ignore_resource_not_found());
    }

    #[test]
    fn defaults_without_resource_or_factory() {
        // B 类（边界行为）：资源与工厂均可缺省
        let descriptor = PropertySourceDescriptor::new(
            "plain",
            vec!["file:./x.properties".to_string()],
            None,
            None,
            false,
        );
        assert!(descriptor.resource().is_none());
        assert_eq!(descriptor.factory_bean_name(), None);
        assert!(!descriptor.ignore_resource_not_found());
    }
}
