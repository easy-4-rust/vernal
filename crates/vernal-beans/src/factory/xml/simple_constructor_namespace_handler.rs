//! SimpleConstructorNamespaceHandler — Spring 风格的简单构造器命名空间处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.SimpleConstructorNamespaceHandler`。
//!
//! 在 Spring 中，`SimpleConstructorNamespaceHandler` 处理构造器参数的
//! 简写语法，如：
//! ```xml
//! <bean class="ExampleBean">
//!     <constructor-arg value="42"/>
//!     <constructor-arg ref="anotherBean"/>
//! </bean>
//! ```
//!
//! 它支持按索引、类型和名称匹配构造器参数。
//!
//! ## 设计说明
//!
//! 在 vernal 中，此处理器解析构造器参数的各种表示形式。

use std::collections::HashMap;

use crate::factory::xml::bean_definition_parser_delegate::BeanDefinitionParserDelegate;
use crate::factory::xml::namespace_handler::NamespaceHandler;

/// 构造器参数定义。
#[derive(Debug, Clone)]
pub struct ConstructorArgDefinition {
    /// 参数索引。
    pub index: Option<usize>,
    /// 参数类型名。
    pub type_name: Option<String>,
    /// 参数名。
    pub name: Option<String>,
    /// 参数值（字符串形式）。
    pub value: Option<String>,
    /// 参数引用（Bean 名称）。
    pub ref_bean: Option<String>,
}

/// 简单构造器命名空间处理器。
///
/// 对应 Spring 的 `SimpleConstructorNamespaceHandler`。
///
/// 处理构造器参数的简写语法。
#[derive(Debug, Default)]
pub struct SimpleConstructorNamespaceHandler {
    /// 已解析的构造器参数（bean_name -> args）。
    parsed_args: std::sync::Mutex<HashMap<String, Vec<ConstructorArgDefinition>>>,
}

impl SimpleConstructorNamespaceHandler {
    /// 创建简单构造器命名空间处理器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 解析构造器参数。
    pub fn parse_constructor_arg(
        &self,
        bean_name: &str,
        attrs: &[(String, String)],
    ) -> Result<ConstructorArgDefinition, Box<dyn std::error::Error + Send + Sync>> {
        let mut arg = ConstructorArgDefinition {
            index: None,
            type_name: None,
            name: None,
            value: None,
            ref_bean: None,
        };

        for (key, value) in attrs {
            match key.as_str() {
                "index" => arg.index = Some(value.parse().map_err(|_| "invalid index")?),
                "type" => arg.type_name = Some(value.clone()),
                "name" => arg.name = Some(value.clone()),
                "value" => arg.value = Some(value.clone()),
                "ref" => arg.ref_bean = Some(value.clone()),
                _ => {}
            }
        }

        self.parsed_args
            .lock()
            .unwrap()
            .entry(bean_name.to_string())
            .or_default()
            .push(arg.clone());

        Ok(arg)
    }

    /// 获取指定 Bean 的构造器参数。
    pub fn get_constructor_args(&self, bean_name: &str) -> Vec<ConstructorArgDefinition> {
        self.parsed_args
            .lock()
            .unwrap()
            .get(bean_name)
            .cloned()
            .unwrap_or_default()
    }
}

impl NamespaceHandler for SimpleConstructorNamespaceHandler {
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
    fn parse_constructor_arg_with_value() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let attrs = vec![
            ("index".to_string(), "0".to_string()),
            ("value".to_string(), "42".to_string()),
        ];
        let arg = handler.parse_constructor_arg("myBean", &attrs).unwrap();
        assert_eq!(arg.index, Some(0));
        assert_eq!(arg.value, Some("42".to_string()));
    }

    #[test]
    fn parse_constructor_arg_with_ref() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let attrs = vec![
            ("ref".to_string(), "otherBean".to_string()),
            ("type".to_string(), "com.example.Service".to_string()),
        ];
        let arg = handler.parse_constructor_arg("myBean", &attrs).unwrap();
        assert_eq!(arg.ref_bean, Some("otherBean".to_string()));
        assert_eq!(arg.type_name, Some("com.example.Service".to_string()));
    }

    #[test]
    fn multiple_constructor_args() {
        let handler = SimpleConstructorNamespaceHandler::new();
        handler.parse_constructor_arg("bean", &[("value".to_string(), "a".to_string())]).unwrap();
        handler.parse_constructor_arg("bean", &[("value".to_string(), "b".to_string())]).unwrap();
        let args = handler.get_constructor_args("bean");
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn unknown_bean_returns_empty() {
        let handler = SimpleConstructorNamespaceHandler::new();
        assert!(handler.get_constructor_args("unknown").is_empty());
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn parse_constructor_arg_with_name() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let attrs = vec![
            ("name".to_string(), "myParam".to_string()),
            ("value".to_string(), "42".to_string()),
        ];
        let arg = handler.parse_constructor_arg("myBean", &attrs).unwrap();
        assert_eq!(arg.name, Some("myParam".to_string()));
        assert_eq!(arg.value, Some("42".to_string()));
    }

    #[test]
    fn parse_constructor_arg_empty_attrs() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let attrs = vec![];
        let arg = handler.parse_constructor_arg("myBean", &attrs).unwrap();
        assert!(arg.index.is_none());
        assert!(arg.type_name.is_none());
        assert!(arg.name.is_none());
        assert!(arg.value.is_none());
        assert!(arg.ref_bean.is_none());
    }

    #[test]
    fn parse_constructor_arg_invalid_index() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let attrs = vec![
            ("index".to_string(), "not_a_number".to_string()),
        ];
        let result = handler.parse_constructor_arg("myBean", &attrs);
        assert!(result.is_err());
    }

    #[test]
    fn parse_constructor_arg_unknown_attr() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let attrs = vec![
            ("unknown".to_string(), "value".to_string()),
            ("value".to_string(), "42".to_string()),
        ];
        let arg = handler.parse_constructor_arg("myBean", &attrs).unwrap();
        assert_eq!(arg.value, Some("42".to_string()));
    }

    #[test]
    fn namespace_handler_init() {
        let mut handler = SimpleConstructorNamespaceHandler::new();
        handler.init();
    }

    #[test]
    fn namespace_handler_parse() {
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

        let handler = SimpleConstructorNamespaceHandler::new();
        let delegate = StubDelegate;
        let result = handler.parse("element", &delegate);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_constructor_arg_with_all_attrs() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let attrs = vec![
            ("index".to_string(), "0".to_string()),
            ("type".to_string(), "String".to_string()),
            ("name".to_string(), "param".to_string()),
            ("value".to_string(), "hello".to_string()),
        ];
        let arg = handler.parse_constructor_arg("myBean", &attrs).unwrap();
        assert_eq!(arg.index, Some(0));
        assert_eq!(arg.type_name, Some("String".to_string()));
        assert_eq!(arg.name, Some("param".to_string()));
        assert_eq!(arg.value, Some("hello".to_string()));
    }

    #[test]
    fn debug_format() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let debug = format!("{:?}", handler);
        assert!(debug.contains("SimpleConstructorNamespaceHandler"));
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn parse_constructor_arg_with_ref_only() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let attrs = vec![
            ("ref".to_string(), "anotherBean".to_string()),
        ];
        let arg = handler.parse_constructor_arg("myBean", &attrs).unwrap();
        assert_eq!(arg.ref_bean, Some("anotherBean".to_string()));
        assert!(arg.index.is_none());
        assert!(arg.type_name.is_none());
        assert!(arg.name.is_none());
        assert!(arg.value.is_none());
    }

    #[test]
    fn parse_constructor_arg_with_type_only() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let attrs = vec![
            ("type".to_string(), "com.example.Service".to_string()),
        ];
        let arg = handler.parse_constructor_arg("myBean", &attrs).unwrap();
        assert_eq!(arg.type_name, Some("com.example.Service".to_string()));
    }

    #[test]
    fn parse_constructor_arg_with_index_only() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let attrs = vec![
            ("index".to_string(), "5".to_string()),
        ];
        let arg = handler.parse_constructor_arg("myBean", &attrs).unwrap();
        assert_eq!(arg.index, Some(5));
    }

    #[test]
    fn parse_constructor_args_for_different_beans() {
        let handler = SimpleConstructorNamespaceHandler::new();
        handler.parse_constructor_arg("bean1", &[("value".to_string(), "a".to_string())]).unwrap();
        handler.parse_constructor_arg("bean2", &[("value".to_string(), "b".to_string())]).unwrap();
        let args1 = handler.get_constructor_args("bean1");
        let args2 = handler.get_constructor_args("bean2");
        assert_eq!(args1.len(), 1);
        assert_eq!(args2.len(), 1);
        assert_eq!(args1[0].value, Some("a".to_string()));
        assert_eq!(args2[0].value, Some("b".to_string()));
    }

    #[test]
    fn default_trait_creates_instance() {
        let handler = SimpleConstructorNamespaceHandler::default();
        assert!(handler.get_constructor_args("any").is_empty());
    }

    #[test]
    fn parse_constructor_arg_index_zero() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let attrs = vec![
            ("index".to_string(), "0".to_string()),
        ];
        let arg = handler.parse_constructor_arg("myBean", &attrs).unwrap();
        assert_eq!(arg.index, Some(0));
    }

    #[test]
    fn parse_constructor_arg_large_index() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let attrs = vec![
            ("index".to_string(), "999".to_string()),
        ];
        let arg = handler.parse_constructor_arg("myBean", &attrs).unwrap();
        assert_eq!(arg.index, Some(999));
    }

    #[test]
    fn constructor_arg_definition_clone() {
        let handler = SimpleConstructorNamespaceHandler::new();
        let attrs = vec![
            ("value".to_string(), "test".to_string()),
        ];
        let arg = handler.parse_constructor_arg("myBean", &attrs).unwrap();
        let cloned = arg.clone();
        assert_eq!(cloned.value, Some("test".to_string()));
    }

    #[test]
    fn constructor_arg_definition_debug() {
        let arg = ConstructorArgDefinition {
            index: Some(0),
            type_name: Some("String".to_string()),
            name: Some("param".to_string()),
            value: Some("hello".to_string()),
            ref_bean: None,
        };
        let debug = format!("{:?}", arg);
        assert!(debug.contains("ConstructorArgDefinition"));
        assert!(debug.contains("hello"));
    }

    #[test]
    fn namespace_handler_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SimpleConstructorNamespaceHandler>();
    }
}
