//! SimplePropertyNamespaceHandler — Spring 风格的简单属性命名空间处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.SimplePropertyNamespaceHandler`。
//!
//! 在 Spring 中，`SimplePropertyNamespaceHandler` 处理属性的简写语法：
//! ```xml
//! <bean class="ExampleBean" p:email="test@example.com"/>
//! ```
//!
//! `p:` 命名空间前缀允许直接在 `<bean>` 元素上声明属性，
//! 而无需嵌套 `<property>` 子元素。
//!
//! ## 设计说明
//!
//! 在 vernal 中，此处理器解析 `p:` 前缀的属性并转换为属性定义。

use std::collections::HashMap;

use crate::factory::xml::bean_definition_parser_delegate::BeanDefinitionParserDelegate;
use crate::factory::xml::namespace_handler::NamespaceHandler;

/// 简单属性命名空间处理器。
///
/// 对应 Spring 的 `SimplePropertyNamespaceHandler`。
///
/// 处理 `p:` 前缀的属性简写语法。
#[derive(Debug, Default)]
pub struct SimplePropertyNamespaceHandler {
    /// 已解析的属性映射（bean_name -> properties）。
    parsed_properties: std::sync::Mutex<HashMap<String, HashMap<String, String>>>,
}

impl SimplePropertyNamespaceHandler {
    /// 创建简单属性命名空间处理器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 解析 p: 前缀属性。
    ///
    /// # 参数
    /// - `bean_name` — Bean 名称
    /// - `p_attrs` — p: 前缀的属性（去掉 "p:" 前缀后的键值对）
    pub fn parse_p_attributes(
        &self,
        bean_name: &str,
        p_attrs: &[(String, String)],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut properties = self.parsed_properties
            .lock()
            .unwrap()
            .entry(bean_name.to_string())
            .or_default()
            .clone();

        for (key, value) in p_attrs {
            // 处理 -ref 后缀（引用其他 Bean）
            if key.ends_with("-ref") {
                let prop_name = key.strip_suffix("-ref").unwrap();
                properties.insert(prop_name.to_string(), format!("ref:{}", value));
            } else {
                properties.insert(key.clone(), value.clone());
            }
        }

        self.parsed_properties.lock().unwrap().insert(bean_name.to_string(), properties);
        Ok(())
    }

    /// 获取指定 Bean 的属性。
    pub fn get_properties(&self, bean_name: &str) -> HashMap<String, String> {
        self.parsed_properties
            .lock()
            .unwrap()
            .get(bean_name)
            .cloned()
            .unwrap_or_default()
    }
}

impl NamespaceHandler for SimplePropertyNamespaceHandler {
    fn init(&mut self) {
        // 无需额外初始化
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
    fn parse_simple_property() {
        let handler = SimplePropertyNamespaceHandler::new();
        let attrs = vec![("email".to_string(), "test@example.com".to_string())];
        handler.parse_p_attributes("myBean", &attrs).unwrap();
        let props = handler.get_properties("myBean");
        assert_eq!(props.get("email"), Some(&"test@example.com".to_string()));
    }

    #[test]
    fn parse_ref_property() {
        let handler = SimplePropertyNamespaceHandler::new();
        let attrs = vec![("service-ref".to_string(), "myService".to_string())];
        handler.parse_p_attributes("myBean", &attrs).unwrap();
        let props = handler.get_properties("myBean");
        assert_eq!(props.get("service"), Some(&"ref:myService".to_string()));
    }

    #[test]
    fn multiple_properties() {
        let handler = SimplePropertyNamespaceHandler::new();
        let attrs = vec![
            ("name".to_string(), "John".to_string()),
            ("age".to_string(), "30".to_string()),
        ];
        handler.parse_p_attributes("person", &attrs).unwrap();
        let props = handler.get_properties("person");
        assert_eq!(props.len(), 2);
    }

    #[test]
    fn unknown_bean_returns_empty() {
        let handler = SimplePropertyNamespaceHandler::new();
        assert!(handler.get_properties("unknown").is_empty());
    }
}
