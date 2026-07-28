//! 占位符解析器。
//!
//! 对标 Spring 7 `org.springframework.util.PlaceholderParser`（since 6.2）。

use super::property_placeholder_helper::{PlaceholderResolver, PropertyPlaceholderHelper};

/// 占位符解析器。对标 Spring `PlaceholderParser`。
pub struct PlaceholderParser {
    helper: PropertyPlaceholderHelper,
}

impl PlaceholderParser {
    /// 创建新的占位符解析器。
    #[must_use]
    pub fn new(
        prefix: impl Into<String>,
        suffix: impl Into<String>,
        separator: Option<impl Into<String>>,
    ) -> Self {
        let helper = match separator {
            Some(sep) => PropertyPlaceholderHelper::new(prefix, suffix, sep),
            None => PropertyPlaceholderHelper::new(prefix, suffix, ""),
        };
        Self { helper }
    }

    /// 使用默认配置（`${` / `}` / `:`）创建解析器。
    #[must_use]
    pub fn with_default() -> Self {
        Self {
            helper: PropertyPlaceholderHelper::with_default(),
        }
    }

    /// 设置是否忽略无法解析的占位符。
    pub fn set_ignore_unresolvable(&mut self, ignore: bool) {
        self.helper.set_ignore_unresolvable(ignore);
    }

    /// 替换字符串中的所有占位符。
    pub fn replace_placeholders<R: PlaceholderResolver>(
        &self,
        value: &str,
        resolver: R,
    ) -> Result<String, super::property_placeholder_helper::PlaceholderError> {
        self.helper.replace_placeholders(value, resolver)
    }

    /// 替换占位符，忽略无法解析的占位符（保留原样）。
    pub fn replace_placeholders_lenient<R: PlaceholderResolver>(
        &self,
        value: &str,
        resolver: R,
    ) -> String {
        self.helper.replace_placeholders_lenient(value, resolver)
    }
}

impl Default for PlaceholderParser {
    fn default() -> Self {
        Self::with_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_resolver(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn simple_replacement() {
        let resolver = make_resolver(&[("name", "world")]);
        let parser = PlaceholderParser::with_default();
        assert_eq!(
            parser.replace_placeholders("Hello ${name}!", resolver).unwrap(),
            "Hello world!"
        );
    }

    #[test]
    fn default_value() {
        let resolver = HashMap::<String, String>::new();
        let parser = PlaceholderParser::with_default();
        assert_eq!(
            parser.replace_placeholders("${missing:fallback}", resolver).unwrap(),
            "fallback"
        );
    }

    #[test]
    fn multiple_placeholders() {
        let resolver = make_resolver(&[("a", "1"), ("b", "2")]);
        let parser = PlaceholderParser::with_default();
        assert_eq!(
            parser.replace_placeholders("${a} + ${b} = 3", resolver).unwrap(),
            "1 + 2 = 3"
        );
    }

    #[test]
    fn nested_placeholders() {
        let resolver = make_resolver(&[("base", "key"), ("key", "value")]);
        let parser = PlaceholderParser::with_default();
        assert_eq!(
            parser.replace_placeholders("${${base}}", resolver).unwrap(),
            "value"
        );
    }

    #[test]
    fn lenient_keeps_unresolvable() {
        let resolver = HashMap::<String, String>::new();
        let parser = PlaceholderParser::with_default();
        assert_eq!(
            parser.replace_placeholders_lenient("${unknown}", resolver),
            "${unknown}"
        );
    }

    #[test]
    fn custom_separators() {
        let resolver = make_resolver(&[("name", "test")]);
        let parser = PlaceholderParser::new("<%", "%>", Some(":"));
        assert_eq!(
            parser.replace_placeholders("Hello <%name%>!", resolver).unwrap(),
            "Hello test!"
        );
    }

    #[test]
    fn default_impl_matches_with_default() {
        let r1 = make_resolver(&[("x", "y")]);
        let r2 = make_resolver(&[("x", "y")]);
        let p1 = PlaceholderParser::default();
        let p2 = PlaceholderParser::with_default();
        assert_eq!(
            p1.replace_placeholders("${x}", r1).unwrap(),
            p2.replace_placeholders("${x}", r2).unwrap()
        );
    }

    #[test]
    fn ignore_unresolvable_mode() {
        let resolver = HashMap::<String, String>::new();
        let mut parser = PlaceholderParser::with_default();
        parser.set_ignore_unresolvable(true);
        assert_eq!(
            parser.replace_placeholders("${unknown}", resolver).unwrap(),
            "${unknown}"
        );
    }
}
