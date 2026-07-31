//! XmlReaderContext — Spring 风格的 XML 读取器上下文。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.XmlReaderContext`。
//!
//! 在 Spring 中，`XmlReaderContext` 封装了 XML Bean 定义读取过程中
//! 需要的上下文信息，包括资源描述、实体解析器和命名空间处理器解析器。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`XmlReaderContext` 存储 XML 解析的配置和状态。

use std::sync::atomic::AtomicU32;

/// XML 读取器上下文。
///
/// 对应 Spring 的 `XmlReaderContext`。
///
/// 封装 XML Bean 定义读取的上下文信息。
#[derive(Debug)]
pub struct XmlReaderContext {
    /// 资源描述（文件路径或标识）。
    resource_description: String,
    /// 实体解析器名称。
    entity_resolver_name: String,
    /// 命名空间处理器解析器名称。
    namespace_handler_resolver_name: String,
    /// 已解析的 Bean 数量。
    bean_count: AtomicU32,
    /// 验证模式。
    validation_mode: ValidationMode,
}

/// XML 验证模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// 不验证。
    None,
    /// 自动检测。
    Auto,
    /// DTD 验证。
    Dtd,
    /// XSD 验证。
    Xsd,
}

impl XmlReaderContext {
    /// 创建 XML 读取器上下文。
    pub fn new(resource_description: impl Into<String>) -> Self {
        Self {
            resource_description: resource_description.into(),
            entity_resolver_name: "DelegatingEntityResolver".to_string(),
            namespace_handler_resolver_name: "DefaultNamespaceHandlerResolver".to_string(),
            bean_count: AtomicU32::new(0),
            validation_mode: ValidationMode::Auto,
        }
    }

    /// 设置实体解析器名称。
    pub fn with_entity_resolver(mut self, name: impl Into<String>) -> Self {
        self.entity_resolver_name = name.into();
        self
    }

    /// 设置命名空间处理器解析器名称。
    pub fn with_namespace_handler_resolver(mut self, name: impl Into<String>) -> Self {
        self.namespace_handler_resolver_name = name.into();
        self
    }

    /// 设置验证模式。
    pub fn with_validation_mode(mut self, mode: ValidationMode) -> Self {
        self.validation_mode = mode;
        self
    }

    /// 获取资源描述。
    pub fn resource_description(&self) -> &str {
        &self.resource_description
    }

    /// 获取实体解析器名称。
    pub fn entity_resolver_name(&self) -> &str {
        &self.entity_resolver_name
    }

    /// 获取命名空间处理器解析器名称。
    pub fn namespace_handler_resolver_name(&self) -> &str {
        &self.namespace_handler_resolver_name
    }

    /// 获取验证模式。
    pub fn validation_mode(&self) -> ValidationMode {
        self.validation_mode
    }

    /// 增加已解析的 Bean 数量。
    pub fn increment_bean_count(&self) {
        self.bean_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// 获取已解析的 Bean 数量。
    pub fn bean_count(&self) -> u32 {
        self.bean_count.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_reader_context_default_values() {
        let ctx = XmlReaderContext::new("classpath:beans.xml");
        assert_eq!(ctx.resource_description(), "classpath:beans.xml");
        assert_eq!(ctx.entity_resolver_name(), "DelegatingEntityResolver");
        assert_eq!(ctx.bean_count(), 0);
    }

    #[test]
    fn bean_count_tracking() {
        let ctx = XmlReaderContext::new("test.xml");
        ctx.increment_bean_count();
        ctx.increment_bean_count();
        ctx.increment_bean_count();
        assert_eq!(ctx.bean_count(), 3);
    }

    #[test]
    fn validation_mode_settings() {
        let ctx = XmlReaderContext::new("test.xml").with_validation_mode(ValidationMode::Xsd);
        assert_eq!(ctx.validation_mode(), ValidationMode::Xsd);
    }

    #[test]
    fn builder_pattern_with_names() {
        let ctx = XmlReaderContext::new("beans.xml")
            .with_entity_resolver("CustomResolver")
            .with_namespace_handler_resolver("CustomNSResolver");
        assert_eq!(ctx.entity_resolver_name(), "CustomResolver");
        assert_eq!(ctx.namespace_handler_resolver_name(), "CustomNSResolver");
    }
}
