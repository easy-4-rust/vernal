//! 属性源解析器抽象。
//!
//! 对标 Spring `org.springframework.core.env.AbstractPropertyResolver`。

use super::configurable_property_resolver::ConfigurablePropertyResolver;
use super::missing_required_properties_exception::MissingRequiredPropertiesException;
use super::property_resolver::LookupAdapter;
use super::PropertyResolver;
use crate::util::PropertyPlaceholderHelper;

/// 属性源解析器抽象。
///
/// 对应 Java: org.springframework.core.env.AbstractPropertyResolver
///
/// Spring 语义：`ConfigurablePropertyResolver` 的基础实现——占位符语法配置
/// （前缀/后缀/值分隔符）、必填属性校验与占位符解析（委托
/// `PropertyPlaceholderHelper`）。
#[derive(Debug, Clone)]
pub struct AbstractPropertyResolver {
    placeholder_prefix: String,
    placeholder_suffix: String,
    value_separator: Option<String>,
    ignore_unresolvable_nested_placeholders: bool,
    required_properties: Vec<String>,
}

impl Default for AbstractPropertyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl AbstractPropertyResolver {
    /// 创建使用默认语法的解析器（`${` / `}` / `:`）。
    #[must_use]
    pub fn new() -> Self {
        Self {
            placeholder_prefix: "${".to_string(),
            placeholder_suffix: "}".to_string(),
            value_separator: Some(":".to_string()),
            ignore_unresolvable_nested_placeholders: false,
            required_properties: Vec::new(),
        }
    }

    /// 返回占位符前缀。
    #[must_use]
    pub fn placeholder_prefix(&self) -> &str {
        &self.placeholder_prefix
    }

    /// 返回占位符后缀。
    #[must_use]
    pub fn placeholder_suffix(&self) -> &str {
        &self.placeholder_suffix
    }

    /// 返回值分隔符。
    #[must_use]
    pub fn value_separator(&self) -> Option<&str> {
        self.value_separator.as_deref()
    }

    /// 返回必填属性列表（供子类校验使用）。
    #[must_use]
    pub fn required_properties(&self) -> &[String] {
        &self.required_properties
    }

    /// 构建占位符解析器（按当前语法配置）。
    fn build_helper(&self) -> PropertyPlaceholderHelper {
        let mut helper = PropertyPlaceholderHelper::new(
            self.placeholder_prefix.clone(),
            self.placeholder_suffix.clone(),
            self.value_separator
                .clone()
                .unwrap_or_else(|| ":".to_string()),
        );
        helper.set_ignore_unresolvable(self.ignore_unresolvable_nested_placeholders);
        helper
    }

    /// 按配置解析占位符（`ignore_unresolvable` 由调用方决定）。
    fn resolve_inner(&self, text: &str, ignore_unresolvable: bool) -> String {
        let helper = self.build_helper();
        let adapter = LookupAdapter::new(self);
        helper.replace_placeholders_lenient(text, adapter)
    }

    /// 占位符查找钩子（由子类提供属性来源；抽象基类无来源）。
    fn lookup(_key: &str) -> Option<String> {
        None
    }
}

impl PropertyResolver for AbstractPropertyResolver {
    fn get_property(&self, _key: &str) -> Option<String> {
        None
    }

    fn resolve_placeholders_inner(&self, text: &str, ignore_unresolvable: bool) -> String {
        self.resolve_inner(text, ignore_unresolvable)
    }
}

impl ConfigurablePropertyResolver for AbstractPropertyResolver {
    fn set_placeholder_prefix(&mut self, prefix: &str) {
        self.placeholder_prefix = prefix.to_string();
    }

    fn set_placeholder_suffix(&mut self, suffix: &str) {
        self.placeholder_suffix = suffix.to_string();
    }

    fn set_value_separator(&mut self, separator: Option<String>) {
        self.value_separator = separator;
    }

    fn set_ignore_unresolvable_nested_placeholders(&mut self, ignore: bool) {
        self.ignore_unresolvable_nested_placeholders = ignore;
    }

    fn set_required_properties(&mut self, required: Vec<String>) {
        self.required_properties = required;
    }

    fn validate_required_properties(&self) -> Result<(), MissingRequiredPropertiesException> {
        let missing: Vec<String> = self
            .required_properties
            .iter()
            .filter(|key| self.get_property(key).is_none())
            .cloned()
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(MissingRequiredPropertiesException::new(missing))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试实现：从内部 map 提供属性（对标 Spring 子类覆盖 getProperty）。
    struct MapBackedResolver {
        inner: AbstractPropertyResolver,
        map: std::collections::HashMap<String, String>,
    }

    impl PropertyResolver for MapBackedResolver {
        fn get_property(&self, key: &str) -> Option<String> {
            self.map.get(key).cloned()
        }

        fn resolve_placeholders_inner(&self, text: &str, ignore_unresolvable: bool) -> String {
            let mut helper = crate::util::PropertyPlaceholderHelper::new(
                self.inner.placeholder_prefix().to_string(),
                self.inner.placeholder_suffix().to_string(),
                self.inner.value_separator().unwrap_or(":").to_string(),
            );
            helper.set_ignore_unresolvable(ignore_unresolvable);
            let adapter = LookupAdapter::new(self);
            helper.replace_placeholders_lenient(text, adapter)
        }
    }

    impl ConfigurablePropertyResolver for MapBackedResolver {
        fn set_placeholder_prefix(&mut self, prefix: &str) {
            self.inner.set_placeholder_prefix(prefix);
        }
        fn set_placeholder_suffix(&mut self, suffix: &str) {
            self.inner.set_placeholder_suffix(suffix);
        }
        fn set_value_separator(&mut self, separator: Option<String>) {
            self.inner.set_value_separator(separator);
        }
        fn set_ignore_unresolvable_nested_placeholders(&mut self, ignore: bool) {
            self.inner.set_ignore_unresolvable_nested_placeholders(ignore);
        }
        fn set_required_properties(&mut self, required: Vec<String>) {
            self.inner.set_required_properties(required);
        }
        fn validate_required_properties(&self) -> Result<(), MissingRequiredPropertiesException> {
            let missing: Vec<String> = self
                .inner
                .required_properties()
                .iter()
                .filter(|key| self.get_property(key).is_none())
                .cloned()
                .collect();
            if missing.is_empty() {
                Ok(())
            } else {
                Err(MissingRequiredPropertiesException::new(missing))
            }
        }
    }

    fn resolver_with(map: &[(&str, &str)]) -> MapBackedResolver {
        let mut resolver = MapBackedResolver {
            inner: AbstractPropertyResolver::new(),
            map: std::collections::HashMap::new(),
        };
        for (k, v) in map {
            resolver.map.insert((*k).to_string(), (*v).to_string());
        }
        resolver
    }

    #[test]
    fn resolves_placeholders_with_default_separator() {
        // A 类（合同对齐）：对标 Spring `${key:default}` 语义
        let resolver = resolver_with(&[("name", "vernal")]);
        let output = resolver.resolve_placeholders("hello ${name} v${version:1.0}");
        assert_eq!(output, "hello vernal v1.0");
    }

    #[test]
    fn custom_prefix_suffix_apply() {
        // B 类（边界行为）：对标 Spring setPlaceholderPrefix/Suffix
        let mut resolver = resolver_with(&[("k", "v")]);
        resolver.inner.set_placeholder_prefix("%{");
        resolver.inner.set_placeholder_suffix("}");
        let output = resolver.resolve_placeholders("%{k}");
        assert_eq!(output, "v");
    }

    #[test]
    fn validate_required_properties() {
        // C 类（错误路径）：对标 Spring validateRequiredProperties
        let mut resolver = resolver_with(&[("present", "1")]);
        resolver.set_required_properties(vec!["present".to_string(), "missing".to_string()]);
        let err = resolver.validate_required_properties().unwrap_err();
        assert_eq!(err.missing_properties(), &["missing".to_string()]);
    }
}
