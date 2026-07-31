//! AbstractBeanDefinitionParser — Spring 风格的抽象 Bean 定义解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.AbstractBeanDefinitionParser`。
//!
//! 在 Spring 中，`AbstractBeanDefinitionParser` 实现了 `BeanDefinitionParser` 接口，
//! 提供了 XML 元素解析的模板方法。子类只需实现 `getBeanClass` 和 `doParse` 方法。
//!
//! 解析流程：
//! 1. 从 XML 元素提取 id 和 name
//! 2. 创建 BeanDefinition
//! 3. 调用 `doParse` 子类自定义逻辑
//! 4. 注册到 BeanDefinitionRegistry


/// 抽象 Bean 定义解析器。
///
/// 对应 Spring 的 `AbstractBeanDefinitionParser`。
///
/// 提供 XML 元素解析的模板方法。
/// 子类实现 `parse_internal` 进行自定义解析逻辑。
pub trait AbstractBeanDefinitionParser: Send + Sync {
    /// 获取 Bean 类名。
    ///
    /// 对应 Spring 的 `getBeanClass(Element)`。
    fn bean_class_name(&self) -> &str;

    /// 解析 XML 元素的内部逻辑。
    ///
    /// 对应 Spring 的 `doParse(Element, ParserContext, BeanDefinitionBuilder)`。
    ///
    /// # 参数
    /// - `element_name` — XML 元素名
    /// - `attributes` — XML 属性
    ///
    /// # 返回
    /// 解析结果（Bean 定义名称）。
    fn parse_internal(
        &self,
        element_name: &str,
        attributes: &[(String, String)],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;

    /// 是否应生成 id。
    ///
    /// 对应 Spring 的 `shouldGenerateId(Element)`。
    /// 默认为 false，如果元素没有 id 则自动生成。
    fn should_generate_id(&self) -> bool {
        false
    }

    /// 解析 XML 元素（模板方法）。
    ///
    /// 对应 Spring 的 `parse(Element, ParserContext)`。
    fn parse(
        &self,
        element_name: &str,
        attributes: &[(String, String)],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // 提取 id
        let id = attributes.iter()
            .find(|(k, _)| k == "id")
            .map(|(_, v)| v.clone());

        // 如果没有 id 且需要生成，则生成一个
        let effective_id = match id {
            Some(id) => id,
            None if self.should_generate_id() => {
                format!("generated_{}", element_name)
            }
            None => String::new(),
        };

        // 调用子类解析逻辑
        let result = self.parse_internal(element_name, attributes)?;
        if result.is_empty() {
            Ok(effective_id)
        } else {
            Ok(result)
        }
    }
}

/// 测试用的简单解析器实现。
#[cfg(test)]
mod tests {
    use super::*;

    struct TestParser {
        class_name: String,
    }

    impl TestParser {
        fn new(class_name: &str) -> Self {
            Self { class_name: class_name.to_string() }
        }
    }

    impl AbstractBeanDefinitionParser for TestParser {
        fn bean_class_name(&self) -> &str {
            &self.class_name
        }

        fn parse_internal(
            &self,
            _element_name: &str,
            attributes: &[(String, String)],
        ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            let id = attributes.iter()
                .find(|(k, _)| k == "id")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "default".to_string());
            Ok(id)
        }
    }

    #[test]
    fn parser_extracts_id_from_attributes() {
        let parser = TestParser::new("com.example.MyBean");
        let attrs = vec![("id".to_string(), "myBean".to_string())];
        let result = parser.parse("bean", &attrs).unwrap();
        assert_eq!(result, "myBean");
    }

    #[test]
    fn parser_returns_default_when_no_id() {
        let parser = TestParser::new("com.example.MyBean");
        let attrs = vec![("name".to_string(), "test".to_string())];
        let result = parser.parse("bean", &attrs).unwrap();
        assert_eq!(result, "default");
    }

    #[test]
    fn parser_class_name() {
        let parser = TestParser::new("com.example.Service");
        assert_eq!(parser.bean_class_name(), "com.example.Service");
    }
}
