//! 属性解析器契约。
//!
//! 对标 Spring `org.springframework.core.env.PropertyResolver`。

/// 属性解析器契约。
///
/// 对应 Java: org.springframework.core.env.PropertyResolver
///
/// Spring 语义：属性查找 + `${key:default}` 占位符解析的顶层接口。
pub trait PropertyResolver: Send + Sync {
    /// 判断属性是否存在。
    ///
    /// 对应 Java: `PropertyResolver#containsProperty(String)`
    fn contains_property(&self, key: &str) -> bool {
        self.get_property(key).is_some()
    }

    /// 获取属性值。
    ///
    /// 对应 Java: `PropertyResolver#getProperty(String)`
    fn get_property(&self, key: &str) -> Option<String>;

    /// 获取属性值，缺失时返回默认值。
    ///
    /// 对应 Java: `PropertyResolver#getProperty(String, String)`
    fn get_property_with_default(&self, key: &str, default: &str) -> String {
        self.get_property(key)
            .unwrap_or_else(|| default.to_string())
    }

    /// 获取必填属性值。
    ///
    /// 对应 Java: `PropertyResolver#getRequiredProperty(String)`——缺失时抛
    /// `IllegalStateException`；Rust 中返回错误文本。
    fn get_required_property(&self, key: &str) -> Result<String, String> {
        self.get_property(key)
            .ok_or_else(|| format!("required property '{key}' is not defined"))
    }

    /// 解析文本中的占位符（未解析的保持原样）。
    ///
    /// 对应 Java: `PropertyResolver#resolvePlaceholders(String)`
    fn resolve_placeholders(&self, text: &str) -> String {
        self.resolve_placeholders_inner(text, true)
    }

    /// 解析文本中的占位符（未解析的报错）。
    ///
    /// 对应 Java: `PropertyResolver#resolveRequiredPlaceholders(String)`
    fn resolve_required_placeholders(&self, text: &str) -> Result<String, String> {
        let resolved = self.resolve_placeholders_inner(text, false);
        if resolved.contains("${") {
            return Err(format!("could not resolve placeholder in: {text}"));
        }
        Ok(resolved)
    }

    /// 内部占位符解析入口（由实现提供）。
    fn resolve_placeholders_inner(&self, text: &str, ignore_unresolvable: bool) -> String;
}

/// 占位符查找适配器：把 [`PropertyResolver`] 适配为
/// [`crate::util::PropertyPlaceholderHelper`] 的解析器契约。
pub struct LookupAdapter<'a> {
    resolver: &'a dyn PropertyResolver,
}

impl<'a> LookupAdapter<'a> {
    /// 创建适配器。
    #[must_use]
    pub fn new(resolver: &'a dyn PropertyResolver) -> Self {
        Self { resolver }
    }
}

impl crate::util::property_placeholder_helper::PlaceholderResolver for LookupAdapter<'_> {
    fn resolve_placeholder(&self, placeholder_name: &str) -> Option<String> {
        self.resolver.get_property(placeholder_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StaticResolver;

    impl PropertyResolver for StaticResolver {
        fn get_property(&self, key: &str) -> Option<String> {
            match key {
                "name" => Some("vernal".to_string()),
                _ => None,
            }
        }

        fn resolve_placeholders_inner(&self, text: &str, ignore_unresolvable: bool) -> String {
            let mut result = text.to_string();
            result = result.replace("${name}", "vernal");
            if !ignore_unresolvable && result.contains("${") {
                // 保持原样由上层报错
            }
            result
        }
    }

    #[test]
    fn contains_and_get() {
        // A 类（合同对齐）：对标 Spring 属性查找
        let resolver = StaticResolver;
        assert!(resolver.contains_property("name"));
        assert!(!resolver.contains_property("missing"));
        assert_eq!(resolver.get_property("name").as_deref(), Some("vernal"));
        assert_eq!(resolver.get_property_with_default("x", "d"), "d");
    }

    #[test]
    fn required_property_errors_when_missing() {
        // C 类（错误路径）：对标 Spring `IllegalStateException`
        let resolver = StaticResolver;
        assert!(resolver.get_required_property("name").is_ok());
        assert!(resolver.get_required_property("nope").is_err());
    }
}
