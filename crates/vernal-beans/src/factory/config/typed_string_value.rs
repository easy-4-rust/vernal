//! TypedStringValue — 对应 Spring `org.springframework.beans.factory.config.TypedStringValue`。
//!
//! 带类型的字符串值，用于 Bean 属性注入。

use std::fmt;

/// 带类型的字符串值。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.TypedStringValue`。
///
/// 表示一个带有目标类型的字符串值，用于 Bean 属性注入时的类型转换。
///
/// ## 使用场景
///
/// - XML 配置中的 `<value type="java.lang.Integer">42</value>`
/// - 注解中的字符串值需要转换为目标类型
/// - 类型安全的属性注入
#[derive(Debug, Clone)]
pub struct TypedStringValue {
    /// 字符串值。
    value: String,
    /// 目标类型名（可选）。
    target_type_name: Option<String>,
    /// 源类型（可选）。
    source_type_name: Option<String>,
    /// 是否已指定目标类型。
    specified_target_type: bool,
}

impl TypedStringValue {
    /// 创建新的带类型字符串值（仅字符串）。
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            target_type_name: None,
            source_type_name: None,
            specified_target_type: false,
        }
    }

    /// 创建带目标类型的字符串值。
    pub fn with_target_type(
        value: impl Into<String>,
        target_type_name: impl Into<String>,
    ) -> Self {
        Self {
            value: value.into(),
            target_type_name: Some(target_type_name.into()),
            source_type_name: None,
            specified_target_type: true,
        }
    }

    /// 获取字符串值。
    pub fn value(&self) -> &str {
        &self.value
    }

    /// 获取目标类型名。
    pub fn target_type_name(&self) -> Option<&str> {
        self.target_type_name.as_deref()
    }

    /// 获取源类型名。
    pub fn source_type_name(&self) -> Option<&str> {
        self.source_type_name.as_deref()
    }

    /// 设置源类型名。
    pub fn set_source_type_name(&mut self, source_type_name: impl Into<String>) {
        self.source_type_name = Some(source_type_name.into());
    }

    /// 是否指定了目标类型。
    pub fn is_specified_target_type(&self) -> bool {
        self.specified_target_type
    }

    /// 设置目标类型名。
    pub fn set_target_type_name(&mut self, target_type_name: impl Into<String>) {
        self.target_type_name = Some(target_type_name.into());
        self.specified_target_type = true;
    }
}

impl fmt::Display for TypedStringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.target_type_name {
            Some(type_name) => write!(f, "[{}]{}", type_name, self.value),
            None => write!(f, "{}", self.value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_string_value_new() {
        let tsv = TypedStringValue::new("42");
        assert_eq!(tsv.value(), "42");
        assert!(tsv.target_type_name().is_none());
        assert!(!tsv.is_specified_target_type());
    }

    #[test]
    fn test_typed_string_value_with_target_type() {
        let tsv = TypedStringValue::with_target_type("42", "java.lang.Integer");
        assert_eq!(tsv.value(), "42");
        assert_eq!(tsv.target_type_name(), Some("java.lang.Integer"));
        assert!(tsv.is_specified_target_type());
        assert_eq!(format!("{}", tsv), "[java.lang.Integer]42");
    }

    #[test]
    fn test_typed_string_value_set_source_type() {
        let mut tsv = TypedStringValue::new("hello");
        assert!(tsv.source_type_name().is_none());

        tsv.set_source_type_name("java.lang.String");
        assert_eq!(tsv.source_type_name(), Some("java.lang.String"));
    }
}
