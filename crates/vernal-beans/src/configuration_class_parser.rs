//! ConfigurationClassParser — Spring 风格的配置类解析器。
//!
//! 对应 Java 类：`org.springframework.context.annotation.ConfigurationClassParser`。
//!
//! 解析一组候选类名，识别其中的 `@Configuration` 类并采集 `@Bean` 方法与 `@Import`。

use crate::configuration_class::ConfigurationClass;

/// Spring 风格的配置类解析器。
///
/// 对应 Spring 的 `ConfigurationClassParser`。
///
/// 接收一组候选类名，借助提供的配置识别函数判定每个类是否为配置类，
/// 并构造 `ConfigurationClass` 列表。
pub struct ConfigurationClassParser {
    /// 用于判定一个类是否为 `@Configuration` 的谓词。
    configuration_predicate: Box<dyn Fn(&str) -> bool + Send + Sync>,
    /// 用于为一个配置类返回其 `@Bean` 方法名列表的函数。
    bean_methods_provider: Box<dyn Fn(&str) -> Vec<String> + Send + Sync>,
    /// 用于为一个配置类返回其 `@Import` 类名列表的函数。
    imports_provider: Box<dyn Fn(&str) -> Vec<String> + Send + Sync>,
}

impl std::fmt::Debug for ConfigurationClassParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigurationClassParser")
            .finish_non_exhaustive()
    }
}

impl ConfigurationClassParser {
    /// 创建默认的配置类解析器。
    ///
    /// 默认情况下，类名以 `Config` 结尾被视为配置类，
    /// 没有预定义的 `@Bean` 方法或 `@Import`。
    pub fn new() -> Self {
        Self {
            configuration_predicate: Box::new(|name: &str| name.ends_with("Config")),
            bean_methods_provider: Box::new(|_| Vec::new()),
            imports_provider: Box::new(|_| Vec::new()),
        }
    }

    /// 自定义配置类识别谓词。
    pub fn with_configuration_predicate<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.configuration_predicate = Box::new(predicate);
        self
    }

    /// 自定义 `@Bean` 方法提供器。
    pub fn with_bean_methods_provider<F>(mut self, provider: F) -> Self
    where
        F: Fn(&str) -> Vec<String> + Send + Sync + 'static,
    {
        self.bean_methods_provider = Box::new(provider);
        self
    }

    /// 自定义 `@Import` 提供器。
    pub fn with_imports_provider<F>(mut self, provider: F) -> Self
    where
        F: Fn(&str) -> Vec<String> + Send + Sync + 'static,
    {
        self.imports_provider = Box::new(provider);
        self
    }

    /// 解析一组候选类名，返回识别到的 `ConfigurationClass` 列表。
    ///
    /// 对应 Spring 的 `ConfigurationClassParser.parse(Set<BeanDefinitionHolder>)`。
    pub fn parse_configuration_classes(
        &self,
        candidate_class_names: &[String],
    ) -> Vec<ConfigurationClass> {
        let mut result = Vec::new();
        for name in candidate_class_names {
            if !(self.configuration_predicate)(name) {
                continue;
            }
            let mut config = ConfigurationClass::new(name.clone());
            for method in (self.bean_methods_provider)(name) {
                config.add_bean_method(method);
            }
            for import in (self.imports_provider)(name) {
                config.add_import(import);
            }
            result.push(config);
        }
        result
    }
}

impl Default for ConfigurationClassParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_predicate() {
        let parser = ConfigurationClassParser::new();
        let classes = parser.parse_configuration_classes(&[
            "com.example.AppConfig".to_owned(),
            "com.example.PlainBean".to_owned(),
        ]);
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].class_name(), "com.example.AppConfig");
    }

    #[test]
    fn test_custom_providers() {
        let parser = ConfigurationClassParser::new()
            .with_configuration_predicate(|name| name.contains("Config"))
            .with_bean_methods_provider(|name| {
                if name == "com.example.AppConfig" {
                    vec!["createFoo".to_owned()]
                } else {
                    vec![]
                }
            })
            .with_imports_provider(|_| vec!["com.extra.Util".to_owned()]);

        let classes = parser.parse_configuration_classes(&["com.example.AppConfig".to_owned()]);
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].bean_methods(), &["createFoo".to_owned()]);
        assert_eq!(classes[0].imported(), &["com.extra.Util".to_owned()]);
    }
}
