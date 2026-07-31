//! AbstractSingleBeanDefinitionParser — Spring 风格的单 Bean 定义解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.AbstractSingleBeanDefinitionParser`。
//!
//! 在 Spring 中，`AbstractSingleBeanDefinitionParser` 继承 `AbstractBeanDefinitionParser`，
//! 用于解析定义单个 Bean 的 XML 元素。
//! 它自动处理 id/name 提取、Bean 类名解析和属性设置。

use std::collections::HashMap;

/// 单 Bean 定义解析结果。
#[derive(Debug, Clone)]
pub struct SingleBeanParseResult {
    /// Bean id。
    pub id: String,
    /// Bean 类名。
    pub bean_class: String,
    /// 属性。
    pub properties: HashMap<String, String>,
    /// 构造器参数。
    pub constructor_args: Vec<String>,
    /// 父 Bean 名称。
    pub parent: Option<String>,
}

/// 单 Bean 定义解析器。
///
/// 对应 Spring 的 `AbstractSingleBeanDefinitionParser`。
///
/// 解析定义单个 Bean 的 XML 元素，自动处理 id/name 和类名。
pub trait AbstractSingleBeanDefinitionParser: Send + Sync {
    /// 获取 Bean 类名（从元素属性中提取）。
    ///
    /// 对应 Spring 的 `getBeanClass(Element)`。
    fn resolve_bean_class(&self, attributes: &[(String, String)]) -> Option<String>;

    /// 解析自定义属性。
    ///
    /// 对应 Spring 的 `doParse(Element, BeanDefinitionBuilder)`。
    fn do_parse(
        &self,
        _element_name: &str,
        attributes: &[(String, String)],
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
        // 默认实现：跳过 id/name/class 属性，其余作为 Bean 属性
        let skip = ["id", "name", "class", "parent"];
        Ok(attributes.iter()
            .filter(|(k, _)| !skip.contains(&k.as_str()))
            .cloned()
            .collect())
    }

    /// 解析元素（模板方法）。
    fn parse(
        &self,
        element_name: &str,
        attributes: &[(String, String)],
    ) -> Result<SingleBeanParseResult, Box<dyn std::error::Error + Send + Sync>> {
        let id = attributes.iter()
            .find(|(k, _)| k == "id" || k == "name")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| format!("generated_{}", element_name));

        let bean_class = self.resolve_bean_class(attributes)
            .ok_or("Bean class not specified")?;

        let parent = attributes.iter()
            .find(|(k, _)| k == "parent")
            .map(|(_, v)| v.clone());

        let properties = self.do_parse(element_name, attributes)?;

        Ok(SingleBeanParseResult {
            id,
            bean_class,
            properties,
            constructor_args: Vec::new(),
            parent,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleBeanParser;

    impl AbstractSingleBeanDefinitionParser for SimpleBeanParser {
        fn resolve_bean_class(&self, attributes: &[(String, String)]) -> Option<String> {
            attributes.iter()
                .find(|(k, _)| k == "class")
                .map(|(_, v)| v.clone())
        }
    }

    #[test]
    fn parser_extracts_id_and_class() {
        let parser = SimpleBeanParser;
        let attrs = vec![
            ("id".to_string(), "myBean".to_string()),
            ("class".to_string(), "com.example.MyBean".to_string()),
        ];
        let result = parser.parse("bean", &attrs).unwrap();
        assert_eq!(result.id, "myBean");
        assert_eq!(result.bean_class, "com.example.MyBean");
    }

    #[test]
    fn parser_extracts_parent() {
        let parser = SimpleBeanParser;
        let attrs = vec![
            ("id".to_string(), "child".to_string()),
            ("class".to_string(), "ChildClass".to_string()),
            ("parent".to_string(), "parentBean".to_string()),
        ];
        let result = parser.parse("bean", &attrs).unwrap();
        assert_eq!(result.parent, Some("parentBean".to_string()));
    }

    #[test]
    fn parser_error_when_no_class() {
        let parser = SimpleBeanParser;
        let attrs = vec![("id".to_string(), "bean".to_string())];
        let result = parser.parse("bean", &attrs);
        assert!(result.is_err());
    }

    #[test]
    fn parser_custom_properties() {
        let parser = SimpleBeanParser;
        let attrs = vec![
            ("id".to_string(), "svc".to_string()),
            ("class".to_string(), "Service".to_string()),
            ("timeout".to_string(), "30".to_string()),
        ];
        let result = parser.parse("bean", &attrs).unwrap();
        assert_eq!(result.properties.get("timeout"), Some(&"30".to_string()));
    }
}
