//! PropertyResolver — Spring 风格的属性解析器 trait。
//!
//! 对应 Java 类：`org.springframework.core.env.PropertyResolver`。
//!
//! 提供属性查询与占位符解析的统一接口。

/// Spring 风格的属性解析器 trait。
///
/// 对应 Spring 的 `PropertyResolver`。
///
/// `Environment` 继承此接口，提供按名查询属性、查询 profile 以及
/// 占位符解析能力。
pub trait PropertyResolver {
    /// 是否包含指定属性。
    ///
    /// 对应 Spring 的 `containsProperty(String key)`。
    fn contains_property(&self, key: &str) -> bool;

    /// 获取指定属性值。
    ///
    /// 对应 Spring 的 `getProperty(String key)`。
    fn get_property(&self, key: &str) -> Option<String>;

    /// 获取指定属性值，缺失时返回默认值。
    ///
    /// 对应 Spring 的 `getProperty(String key, String defaultValue)`。
    fn get_property_or(&self, key: &str, default_value: &str) -> String {
        self.get_property(key)
            .unwrap_or_else(|| default_value.to_owned())
    }

    /// 获取必需属性，缺失时返回错误。
    ///
    /// 对应 Spring 的 `getRequiredProperty(String key)`。
    fn get_required_property(&self, key: &str) -> Result<String, MissingRequiredPropertyError> {
        self.get_property(key)
            .ok_or_else(|| MissingRequiredPropertyError::new(key.to_owned()))
    }

    /// 解析字符串中的 `${...}` 占位符。
    ///
    /// 对应 Spring 的 `resolvePlaceholders(String text)`。
    /// 遇到无法解析的占位符时原样保留。
    fn resolve_placeholders(&self, text: &str) -> String;

    /// 解析字符串中的 `${...}` 占位符，遇到无法解析的占位符时返回错误。
    ///
    /// 对应 Spring 的 `resolveRequiredPlaceholders(String text)`。
    fn resolve_required_placeholders(
        &self,
        text: &str,
    ) -> Result<String, UnresolvedPlaceholderError>;
}

/// 缺失必需属性错误。
#[derive(Debug, Clone)]
pub struct MissingRequiredPropertyError {
    /// 缺失的属性键。
    pub key: String,
}

impl std::fmt::Display for MissingRequiredPropertyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Required property '{}' is not defined", self.key)
    }
}

impl std::error::Error for MissingRequiredPropertyError {}

impl MissingRequiredPropertyError {
    /// 创建错误。
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

/// 占位符无法解析错误（用于 `resolve_required_placeholders`）。
#[derive(Debug, Clone)]
pub struct UnresolvedPlaceholderError {
    /// 原始输入。
    pub input: String,
}

impl std::fmt::Display for UnresolvedPlaceholderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Could not resolve placeholders in value \"{}\"",
            self.input
        )
    }
}

impl std::error::Error for UnresolvedPlaceholderError {}

impl UnresolvedPlaceholderError {
    /// 创建错误。
    pub fn new(input: String) -> Self {
        Self { input }
    }
}
