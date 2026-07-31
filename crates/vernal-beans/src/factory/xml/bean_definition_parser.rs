//! BeanDefinitionParser — Spring 风格的 Bean 定义解析器接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.BeanDefinitionParser`。
//!
//! 在 Spring 中，`BeanDefinitionParser` 用于解析自定义命名空间中的 XML 元素。
//! 每个自定义命名空间元素（如 `<context:component-scan>`）都由一个
//! `BeanDefinitionParser` 实现来解析。
//!
//! ## 与 NamespaceHandler 的关系
//!
//! `NamespaceHandler` 负责根据元素名分发到对应的 `BeanDefinitionParser`。
//! `BeanDefinitionParser` 负责具体的解析逻辑。

use std::collections::HashMap;

/// Bean 定义解析器接口。
///
/// 对应 Spring 的 `BeanDefinitionParser`。
///
/// 解析自定义命名空间中的 XML 元素，注册 Bean 定义。
pub trait BeanDefinitionParser: Send + Sync {
    /// 解析 XML 元素。
    ///
    /// 对应 Spring 的 `BeanDefinition parse(Element, ParserContext)`。
    ///
    /// # 参数
    /// - `element_name` — XML 元素名
    /// - `attributes` — XML 属性
    ///
    /// # 返回
    /// 解析结果，包含 Bean 名称和属性映射。
    fn parse(
        &self,
        element_name: &str,
        attributes: &[(String, String)],
    ) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>;
}

/// 解析结果。
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// 注册的 Bean 名称（可能多个）。
    pub bean_names: Vec<String>,
    /// Bean 属性映射（bean_name -> properties）。
    pub bean_properties: HashMap<String, HashMap<String, String>>,
}

impl ParseResult {
    /// 创建单 Bean 解析结果。
    pub fn single(bean_name: impl Into<String>) -> Self {
        let name = bean_name.into();
        let mut bean_properties = HashMap::new();
        bean_properties.insert(name.clone(), HashMap::new());
        Self {
            bean_names: vec![name],
            bean_properties,
        }
    }

    /// 创建多 Bean 解析结果。
    pub fn multiple(names: Vec<String>) -> Self {
        let bean_properties = names.iter()
            .map(|n| (n.clone(), HashMap::new()))
            .collect();
        Self {
            bean_names: names,
            bean_properties,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestParser;

    impl BeanDefinitionParser for TestParser {
        fn parse(
            &self,
            _element_name: &str,
            attributes: &[(String, String)],
        ) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>> {
            let id = attributes.iter()
                .find(|(k, _)| k == "id")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "default".to_string());
            Ok(ParseResult::single(id))
        }
    }

    #[test]
    fn parser_returns_single_result() {
        let parser = TestParser;
        let attrs = vec![("id".to_string(), "myBean".to_string())];
        let result = parser.parse("test", &attrs).unwrap();
        assert_eq!(result.bean_names.len(), 1);
        assert_eq!(result.bean_names[0], "myBean");
    }

    #[test]
    fn parse_result_multiple() {
        let result = ParseResult::multiple(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(result.bean_names.len(), 2);
        assert!(result.bean_properties.contains_key("a"));
        assert!(result.bean_properties.contains_key("b"));
    }

    #[test]
    fn parser_default_id() {
        let parser = TestParser;
        let result = parser.parse("test", &[]).unwrap();
        assert_eq!(result.bean_names[0], "default");
    }
}
