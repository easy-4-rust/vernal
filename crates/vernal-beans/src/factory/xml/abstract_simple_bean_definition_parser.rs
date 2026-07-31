//! AbstractSimpleBeanDefinitionParser — Spring 风格的简单 Bean 定义解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.AbstractSimpleBeanDefinitionParser`。
//!
//! 在 Spring 中，`AbstractSimpleBeanDefinitionParser` 继承 `AbstractBeanDefinitionParser`，
//! 针对只有少量属性的简单 XML 元素提供便利。
//! 子类只需重写 `getBeanClass` 和可选的属性映射。

use std::collections::HashMap;

/// 简单 Bean 定义解析器。
///
/// 对应 Spring 的 `AbstractSimpleBeanDefinitionParser`。
///
/// 为只有少量属性的简单 XML 元素提供解析便利。
/// 通过属性映射表自动将 XML 属性绑定到 Bean 定义。
pub trait AbstractSimpleBeanDefinitionParser: Send + Sync {
    /// 获取 Bean 类名。
    fn bean_class_name(&self) -> &str;

    /// 获取属性映射表。
    ///
    /// 返回 XML 属性名到 Bean 属性名的映射。
    /// 例如：`"url" => "endpointUrl"` 表示 XML 属性 `url` 映射到 Bean 属性 `endpointUrl`。
    fn attribute_mappings(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// 是否应生成 id。
    fn should_generate_id(&self) -> bool {
        true
    }

    /// 解析 XML 元素。
    ///
    /// 根据属性映射表自动提取 XML 属性并绑定到 Bean 定义。
    fn parse(
        &self,
        element_name: &str,
        attributes: &[(String, String)],
    ) -> Result<SimpleParseResult, Box<dyn std::error::Error + Send + Sync>> {
        let mappings = self.attribute_mappings();
        let mut properties = HashMap::new();

        for (attr_name, attr_value) in attributes {
            // 检查是否有映射
            let bean_prop = mappings.get(attr_name).cloned().unwrap_or_else(|| attr_name.clone());
            properties.insert(bean_prop, attr_value.clone());
        }

        let id = attributes.iter()
            .find(|(k, _)| k == "id")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| {
                if self.should_generate_id() {
                    format!("generated_{}", element_name)
                } else {
                    String::new()
                }
            });

        Ok(SimpleParseResult {
            id,
            bean_class: self.bean_class_name().to_string(),
            properties,
        })
    }
}

/// 简单解析结果。
#[derive(Debug, Clone)]
pub struct SimpleParseResult {
    /// Bean id。
    pub id: String,
    /// Bean 类名。
    pub bean_class: String,
    /// 解析出的属性。
    pub properties: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleServiceParser;

    impl AbstractSimpleBeanDefinitionParser for SimpleServiceParser {
        fn bean_class_name(&self) -> &str {
            "com.example.SimpleService"
        }

        fn attribute_mappings(&self) -> HashMap<String, String> {
            let mut m = HashMap::new();
            m.insert("url".to_string(), "endpointUrl".to_string());
            m
        }
    }

    #[test]
    fn simple_parser_maps_attributes() {
        let parser = SimpleServiceParser;
        let attrs = vec![
            ("id".to_string(), "svc".to_string()),
            ("url".to_string(), "http://example.com".to_string()),
        ];
        let result = parser.parse("simple-service", &attrs).unwrap();
        assert_eq!(result.id, "svc");
        assert_eq!(result.bean_class, "com.example.SimpleService");
        assert_eq!(result.properties.get("endpointUrl"), Some(&"http://example.com".to_string()));
    }

    #[test]
    fn simple_parser_generates_id() {
        let parser = SimpleServiceParser;
        let attrs = vec![("url".to_string(), "http://test".to_string())];
        let result = parser.parse("svc", &attrs).unwrap();
        assert_eq!(result.id, "generated_svc");
    }

    #[test]
    fn unmapped_attrs_use_original_name() {
        let parser = SimpleServiceParser;
        let attrs = vec![("timeout".to_string(), "30".to_string())];
        let result = parser.parse("svc", &attrs).unwrap();
        assert_eq!(result.properties.get("timeout"), Some(&"30".to_string()));
    }
}
