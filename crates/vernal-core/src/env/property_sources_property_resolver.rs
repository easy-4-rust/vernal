//! 属性源解析器。
//!
//! 对标 Spring `org.springframework.core.env.PropertySourcesPropertyResolver`。

use super::PropertyResolver;
use super::abstract_property_resolver::AbstractPropertyResolver;
use super::configurable_property_resolver::ConfigurablePropertyResolver;
use super::missing_required_properties_exception::MissingRequiredPropertiesException;
use super::property_resolver::LookupAdapter;
use super::property_sources::PropertySources;

/// 属性源解析器。
///
/// 对应 Java: org.springframework.core.env.PropertySourcesPropertyResolver
///
/// Spring 语义：`AbstractPropertyResolver` 的属性源形态——`getProperty` 按
/// 优先级遍历 `PropertySources`，首个命中返回；占位符解析引用相同来源。
pub struct PropertySourcesPropertyResolver<'a> {
    inner: AbstractPropertyResolver,
    property_sources: &'a dyn PropertySources,
}

impl<'a> PropertySourcesPropertyResolver<'a> {
    /// 创建解析器（默认占位符语法）。
    #[must_use]
    pub fn new(property_sources: &'a dyn PropertySources) -> Self {
        Self {
            inner: AbstractPropertyResolver::new(),
            property_sources,
        }
    }
}

impl PropertyResolver for PropertySourcesPropertyResolver<'_> {
    fn get_property(&self, key: &str) -> Option<String> {
        for source in self.property_sources.sources() {
            if let Some(value) = source.get_property(key) {
                return Some(value);
            }
        }
        None
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

impl ConfigurablePropertyResolver for PropertySourcesPropertyResolver<'_> {
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
        self.inner
            .set_ignore_unresolvable_nested_placeholders(ignore);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::{MapPropertySource, MutablePropertySources};

    fn sources_with(entries: &[(&str, &str, &str)]) -> MutablePropertySources {
        let mut sources = MutablePropertySources::new();
        for (name, key, value) in entries {
            let mut map = std::collections::HashMap::new();
            map.insert((*key).to_string(), (*value).to_string());
            sources.add_last(Box::new(MapPropertySource::new(*name, map)));
        }
        sources
    }

    #[test]
    fn first_source_wins() {
        // A 类（合同对齐）：对标 Spring 优先级解析
        let sources = sources_with(&[("low", "key", "low"), ("high", "key", "high")]);
        let resolver = PropertySourcesPropertyResolver::new(&sources);
        assert_eq!(resolver.get_property("key").as_deref(), Some("low"));
    }

    #[test]
    fn resolves_placeholders_from_sources() {
        // A 类（合同对齐）：对标 Spring 占位符解析走属性源
        let sources = sources_with(&[("app", "name", "vernal")]);
        let resolver = PropertySourcesPropertyResolver::new(&sources);
        assert_eq!(
            resolver.resolve_placeholders("hi ${name} ${missing:default}"),
            "hi vernal default"
        );
    }

    #[test]
    fn required_property_missing_returns_error() {
        // C 类（错误路径）：对标 Spring getRequiredProperty
        let sources = sources_with(&[]);
        let resolver = PropertySourcesPropertyResolver::new(&sources);
        assert!(resolver.get_required_property("nope").is_err());
    }

    #[test]
    fn validate_required_properties_reports_missing() {
        // C 类（错误路径）：对标 Spring validateRequiredProperties
        let sources = sources_with(&[("app", "present", "1")]);
        let mut resolver = PropertySourcesPropertyResolver::new(&sources);
        resolver.set_required_properties(vec!["present".to_string(), "absent".to_string()]);
        let err = resolver.validate_required_properties().unwrap_err();
        assert_eq!(err.missing_properties(), &["absent".to_string()]);
    }

    #[test]
    fn custom_placeholder_syntax_applies() {
        // B 类（边界行为）：对标 Spring 自定义语法
        let sources = sources_with(&[("app", "k", "v")]);
        let mut resolver = PropertySourcesPropertyResolver::new(&sources);
        resolver.set_placeholder_prefix("%{");
        resolver.set_placeholder_suffix("}");
        assert_eq!(resolver.resolve_placeholders("%{k}"), "v");
    }
}
