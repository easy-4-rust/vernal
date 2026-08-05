//! DefaultBeanDefinitionDocumentReader — 默认 Bean 定义文档读取器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.DefaultBeanDefinitionDocumentReader`。
//!
//! 负责遍历 XML 文档，提取 Bean 定义并委托给解析器。

use crate::factory::xml::bean_definition_parser_delegate::BeanDefinitionParserDelegate;
use crate::factory::xml::document_loader::Document;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::xml::bean_definition_parser_delegate::BeanDefinitionParserDelegate;

    struct StubDelegate;
    impl BeanDefinitionParserDelegate for StubDelegate {
        fn parse_bean_element(
            &self,
            _element_name: &str,
            _attributes: &[(String, String)],
        ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok("test_bean".to_string())
        }
        fn parse_constructor_arg_element(
            &self,
            _element_name: &str,
            _attributes: &[(String, String)],
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn parse_property_element(
            &self,
            _element_name: &str,
            _attributes: &[(String, String)],
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn parse_qualifier_element(
            &self,
            _element_name: &str,
            _attributes: &[(String, String)],
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }

    #[test]
    fn new_creates_instance() {
        let reader = DefaultBeanDefinitionDocumentReader::new();
        let _ = reader;
    }

    #[test]
    fn default_creates_instance() {
        let reader = DefaultBeanDefinitionDocumentReader::default();
        let _ = reader;
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn register_bean_definitions_returns_zero() {
        let reader = DefaultBeanDefinitionDocumentReader::new();
        let document = Document {
            root_element: None,
            system_id: None,
        };
        let delegate = StubDelegate;
        let result = reader
            .register_bean_definitions(&document, &delegate)
            .unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn trait_object_usage() {
        let reader: Box<dyn BeanDefinitionDocumentReader> =
            Box::new(DefaultBeanDefinitionDocumentReader::new());
        let document = Document {
            root_element: None,
            system_id: Some("test.xml".to_string()),
        };
        let delegate = StubDelegate;
        let result = reader
            .register_bean_definitions(&document, &delegate)
            .unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn multiple_calls_return_same_result() {
        let reader = DefaultBeanDefinitionDocumentReader::new();
        let document = Document {
            root_element: None,
            system_id: None,
        };
        let delegate = StubDelegate;
        let r1 = reader
            .register_bean_definitions(&document, &delegate)
            .unwrap();
        let r2 = reader
            .register_bean_definitions(&document, &delegate)
            .unwrap();
        assert_eq!(r1, r2);
    }
}
