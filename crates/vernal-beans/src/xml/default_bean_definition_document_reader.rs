//! DefaultBeanDefinitionDocumentReader — 默认 Bean 定义文档读取器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.DefaultBeanDefinitionDocumentReader`。
//!
//! 负责遍历 XML 文档，提取 Bean 定义并委托给解析器。

use crate::xml::bean_definition_parser_delegate::BeanDefinitionParserDelegate;
use crate::xml::document_loader::Document;

/// Bean 定义文档读取器接口。
///
/// 对应 Spring 的 `BeanDefinitionDocumentReader`。
///
/// 遍历 XML 文档，提取 `<bean>` 元素并委托给
/// `BeanDefinitionParserDelegate` 解析。
pub trait BeanDefinitionDocumentReader: Send + Sync {
    /// 读取文档中的 Bean 定义。
    ///
    /// # 参数
    /// - `document` — 已加载的 XML 文档
    /// - `delegate` — Bean 定义解析委托
    fn register_bean_definitions(
        &self,
        document: &Document,
        delegate: &dyn BeanDefinitionParserDelegate,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
}

/// 默认的 Bean 定义文档读取器实现。
///
/// 对应 Spring 的 `DefaultBeanDefinitionDocumentReader`。
pub struct DefaultBeanDefinitionDocumentReader;

impl DefaultBeanDefinitionDocumentReader {
    /// 创建新的实例。
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultBeanDefinitionDocumentReader {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanDefinitionDocumentReader for DefaultBeanDefinitionDocumentReader {
    fn register_bean_definitions(
        &self,
        _document: &Document,
        _delegate: &dyn BeanDefinitionParserDelegate,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 实现 XML 文档遍历和 Bean 定义提取
        Ok(0)
    }
}
