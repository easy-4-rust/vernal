//! BeanDefinitionDocumentReader — Spring 风格的 Bean 定义文档读取器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.BeanDefinitionDocumentReader`。
//!
//! 在 Spring 中，`BeanDefinitionDocumentReader` 负责读取 XML 文档并将其
//! 解析为 Bean 定义。默认实现是 `DefaultBeanDefinitionDocumentReader`。
//!
//! 解析流程：
//! 1. 读取文档根元素的 `<beans>` 属性作为默认值
//! 2. 遍历子元素，委托给对应的解析器
//! 3. 处理 `<import>`、`<alias>`、`<bean>` 和自定义元素

/// Bean 定义文档读取器接口。
///
/// 对应 Spring 的 `BeanDefinitionDocumentReader`。
///
/// 负责从 XML 文档中读取 Bean 定义。
pub trait BeanDefinitionDocumentReader: Send + Sync {
    /// 注册 Bean 定义。
    ///
    /// 对应 Spring 的 `registerBeanDefinitions(Document, XmlReaderContext)`。
    ///
    /// # 参数
    /// - `root_element` — 文档根元素名
    /// - `attributes` — 根元素属性
    /// - `elements` — 子元素列表（元素名, 属性列表）
    ///
    /// # 返回
    /// 已注册的 Bean 数量。
    fn register_bean_definitions(
        &self,
        root_element: &str,
        attributes: &[(String, String)],
        elements: &[(String, Vec<(String, String)>)],
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
}

/// 文档默认值定义。
///
/// 对应 Spring 的 `<beans>` 根元素上的默认属性。
#[derive(Debug, Clone, Default)]
pub struct DocumentDefaultsDefinition {
    /// 默认作用域。
    pub default_scope: Option<String>,
    /// 默认是否延迟初始化。
    pub default_lazy_init: Option<bool>,
    /// 默认自动装配模式。
    pub default_autowire: Option<String>,
    /// 默认依赖检查模式。
    pub default_dependency_check: Option<String>,
    /// 默认初始化方法名。
    pub default_init_method: Option<String>,
    /// 默认销毁方法名。
    pub default_destroy_method: Option<String>,
    /// 默认 merge 标志。
    pub default_merge: Option<bool>,
}

/// 默认 Bean 定义文档读取器。
#[derive(Debug, Default)]
pub struct DefaultBeanDefinitionDocumentReader {
    defaults: std::sync::Mutex<DocumentDefaultsDefinition>,
}

impl DefaultBeanDefinitionDocumentReader {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取当前文档默认值。
    pub fn defaults(&self) -> DocumentDefaultsDefinition {
        self.defaults.lock().unwrap().clone()
    }
}

impl BeanDefinitionDocumentReader for DefaultBeanDefinitionDocumentReader {
    fn register_bean_definitions(
        &self,
        _root_element: &str,
        attributes: &[(String, String)],
        elements: &[(String, Vec<(String, String)>)],
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        // 解析根元素默认值
        let mut defaults = self.defaults.lock().unwrap();
        for (key, value) in attributes {
            match key.as_str() {
                "default-lazy-init" => defaults.default_lazy_init = Some(value == "true"),
                "default-autowire" => defaults.default_autowire = Some(value.clone()),
                "default-scope" => defaults.default_scope = Some(value.clone()),
                "default-init-method" => defaults.default_init_method = Some(value.clone()),
                "default-destroy-method" => defaults.default_destroy_method = Some(value.clone()),
                _ => {}
            }
        }

        // 统计 Bean 元素数量
        let count = elements.iter().filter(|(name, _)| name == "bean").count();
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_reader_counts_beans() {
        let reader = DefaultBeanDefinitionDocumentReader::new();
        let elements = vec![
            (
                "bean".to_string(),
                vec![("id".to_string(), "a".to_string())],
            ),
            (
                "bean".to_string(),
                vec![("id".to_string(), "b".to_string())],
            ),
            ("import".to_string(), vec![]),
        ];
        let count = reader
            .register_bean_definitions("beans", &[], &elements)
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn document_reader_parses_defaults() {
        let reader = DefaultBeanDefinitionDocumentReader::new();
        let attrs = vec![
            ("default-lazy-init".to_string(), "true".to_string()),
            ("default-autowire".to_string(), "byName".to_string()),
        ];
        reader
            .register_bean_definitions("beans", &attrs, &[])
            .unwrap();
        let defaults = reader.defaults();
        assert_eq!(defaults.default_lazy_init, Some(true));
        assert_eq!(defaults.default_autowire, Some("byName".to_string()));
    }

    #[test]
    fn empty_document_returns_zero() {
        let reader = DefaultBeanDefinitionDocumentReader::new();
        let count = reader.register_bean_definitions("beans", &[], &[]).unwrap();
        assert_eq!(count, 0);
    }
}
