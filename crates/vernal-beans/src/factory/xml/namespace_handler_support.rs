//! NamespaceHandlerSupport — Spring 风格的命名空间处理器支持基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.NamespaceHandlerSupport`。
//!
//! 在 Spring 中，`NamespaceHandlerSupport` 是 `NamespaceHandler` 的抽象基类，
//! 维护了元素名到 `BeanDefinitionParser` 和属性名到 `BeanDefinitionDecorator` 的映射。
//! 子类只需在 `init` 方法中注册解析器和装饰器。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`NamespaceHandlerSupport` 提供了注册表和查找功能。

use std::collections::HashMap;
use std::sync::Mutex;

use crate::factory::xml::namespace_handler::NamespaceHandler;

/// 命名空间处理器支持基类。
///
/// 对应 Spring 的 `NamespaceHandlerSupport`。
///
/// 维护元素名到解析器、属性名到装饰器的映射。
#[derive(Default)]
pub struct NamespaceHandlerSupport {
    /// 元素名 -> 解析器描述映射。
    parsers: Mutex<HashMap<String, String>>,
    /// 属性名 -> 装饰器描述映射。
    decorators: Mutex<HashMap<String, String>>,
    /// 是否已初始化。
    initialized: std::sync::atomic::AtomicBool,
}

impl NamespaceHandlerSupport {
    /// 创建命名空间处理器支持基类。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册 Bean 定义解析器。
    ///
    /// 对应 Spring 的 `registerBeanDefinitionParser(String, BeanDefinitionParser)`。
    pub fn register_bean_definition_parser(
        &self,
        element_name: impl Into<String>,
        parser_description: impl Into<String>,
    ) {
        self.parsers.lock().unwrap().insert(element_name.into(), parser_description.into());
    }

    /// 注册 Bean 定义装饰器。
    ///
    /// 对应 Spring 的 `registerBeanDefinitionDecorator(String, BeanDefinitionDecorator)`。
    pub fn register_bean_definition_decorator(
        &self,
        attribute_name: impl Into<String>,
        decorator_description: impl Into<String>,
    ) {
        self.decorators.lock().unwrap().insert(attribute_name.into(), decorator_description.into());
    }

    /// 查找元素对应的解析器描述。
    pub fn find_parser_description(&self, element_name: &str) -> Option<String> {
        self.parsers.lock().unwrap().get(element_name).cloned()
    }

    /// 查找属性对应的装饰器描述。
    pub fn find_decorator_description(&self, attribute_name: &str) -> Option<String> {
        self.decorators.lock().unwrap().get(attribute_name).cloned()
    }

    /// 获取已注册的解析器数量。
    pub fn parser_count(&self) -> usize {
        self.parsers.lock().unwrap().len()
    }

    /// 获取已注册的装饰器数量。
    pub fn decorator_count(&self) -> usize {
        self.decorators.lock().unwrap().len()
    }

    /// 标记为已初始化。
    pub fn mark_initialized(&self) {
        self.initialized.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// 检查是否已初始化。
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl NamespaceHandler for NamespaceHandlerSupport {
    fn init(&mut self) {
        self.mark_initialized();
    }

    fn parse(
        &self,
        element_name: &str,
        _delegate: &dyn crate::factory::xml::bean_definition_parser_delegate::BeanDefinitionParserDelegate,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.find_parser_description(element_name).is_some() {
            Ok(())
        } else {
            Err(format!("No parser registered for element '{}'", element_name).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_find_parser() {
        let handler = NamespaceHandlerSupport::new();
        handler.register_bean_definition_parser("component-scan", "ComponentScanParser");
        assert_eq!(handler.parser_count(), 1);
        assert_eq!(handler.find_parser_description("component-scan"), Some("ComponentScanParser".to_string()));
    }

    #[test]
    fn register_and_find_decorator() {
        let handler = NamespaceHandlerSupport::new();
        handler.register_bean_definition_decorator("scoped-proxy", "ScopedProxyDecorator");
        assert_eq!(handler.decorator_count(), 1);
        assert_eq!(handler.find_decorator_description("scoped-proxy"), Some("ScopedProxyDecorator".to_string()));
    }

    #[test]
    fn find_unknown_parser_returns_none() {
        let handler = NamespaceHandlerSupport::new();
        assert!(handler.find_parser_description("unknown").is_none());
    }

    #[test]
    fn initialization_tracking() {
        let mut handler = NamespaceHandlerSupport::new();
        assert!(!handler.is_initialized());
        handler.init();
        assert!(handler.is_initialized());
    }
}
