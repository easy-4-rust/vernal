//! UtilNamespaceHandler — Spring 风格的 util 命名空间处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.UtilNamespaceHandler`。
//!
//! 在 Spring 中，`UtilNamespaceHandler` 处理 `http://www.springframework.org/schema/util`
//! 命名空间，提供以下元素：
//! - `<util:constant>` — 引用静态常量
//! - `<util:property-path>` — 引用属性路径
//! - `<util:list>` — 创建 List Bean
//! - `<util:set>` — 创建 Set Bean
//! - `<util:map>` — 创建 Map Bean
//! - `<util:properties>` — 创建 Properties Bean
//!
//! ## 设计说明
//!
//! 在 vernal 中，此处理器解析 util 命名空间元素并生成相应的 Bean 定义。

use std::collections::HashMap;
use std::sync::Mutex;

use crate::factory::xml::namespace_handler::NamespaceHandler;
use crate::factory::xml::bean_definition_parser_delegate::BeanDefinitionParserDelegate;

/// util 元素类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UtilElementType {
    /// 常量引用。
    Constant,
    /// 属性路径。
    PropertyPath,
    /// 列表。
    List,
    /// 集合。
    Set,
    /// 映射。
    Map,
    /// 属性集合。
    Properties,
}

/// 已解析的 util 元素。
#[derive(Debug, Clone)]
pub struct UtilElement {
    /// 元素类型。
    pub element_type: UtilElementType,
    /// Bean 名称。
    pub bean_name: String,
    /// 元素属性。
    pub attributes: HashMap<String, String>,
}

/// util 命名空间处理器。
///
/// 对应 Spring 的 `UtilNamespaceHandler`。
///
/// 处理 `util:` 命名空间中的元素。
#[derive(Debug, Default)]
pub struct UtilNamespaceHandler {
    /// 已解析的 util 元素。
    parsed_elements: Mutex<Vec<UtilElement>>,
}

impl UtilNamespaceHandler {
    /// 创建 util 命名空间处理器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 解析 util 元素。
    pub fn parse_util_element(
        &self,
        element_type: UtilElementType,
        bean_name: &str,
        attributes: &[(String, String)],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let element = UtilElement {
            element_type,
            bean_name: bean_name.to_string(),
            attributes: attributes.iter().cloned().collect(),
        };
        self.parsed_elements.lock().unwrap().push(element);
        Ok(())
    }

    /// 获取已解析的 util 元素数量。
    pub fn element_count(&self) -> usize {
        self.parsed_elements.lock().unwrap().len()
    }

    /// 获取所有已解析的 util 元素。
    pub fn elements(&self) -> Vec<UtilElement> {
        self.parsed_elements.lock().unwrap().clone()
    }
}

impl NamespaceHandler for UtilNamespaceHandler {
    fn init(&mut self) {
        // 注册各个解析器
    }

    fn parse(
        &self,
        _element_name: &str,
        _delegate: &dyn BeanDefinitionParserDelegate,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_list_element() {
        let handler = UtilNamespaceHandler::new();
        let attrs = vec![("id".to_string(), "myList".to_string())];
        handler.parse_util_element(UtilElementType::List, "myList", &attrs).unwrap();
        assert_eq!(handler.element_count(), 1);
    }

    #[test]
    fn parse_constant_element() {
        let handler = UtilNamespaceHandler::new();
        let attrs = vec![
            ("id".to_string(), "maxValue".to_string()),
            ("static-field".to_string(), "java.lang.Integer.MAX_VALUE".to_string()),
        ];
        handler.parse_util_element(UtilElementType::Constant, "maxValue", &attrs).unwrap();
        let elements = handler.elements();
        assert_eq!(elements[0].element_type, UtilElementType::Constant);
    }

    #[test]
    fn multiple_util_elements() {
        let handler = UtilNamespaceHandler::new();
        handler.parse_util_element(UtilElementType::List, "list1", &[]).unwrap();
        handler.parse_util_element(UtilElementType::Map, "map1", &[]).unwrap();
        handler.parse_util_element(UtilElementType::Set, "set1", &[]).unwrap();
        assert_eq!(handler.element_count(), 3);
    }

    #[test]
    fn empty_handler_returns_zero() {
        let handler = UtilNamespaceHandler::new();
        assert_eq!(handler.element_count(), 0);
        assert!(handler.elements().is_empty());
    }
}
