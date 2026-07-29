//! TypedStringValue — Spring 风格的带类型标记的字符串值。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.TypedStringValue`。
//!
//! 封装一个字符串值及其可选的目标类型名称，用于 Bean 定义中的属性值和
//! 构造参数值。容器在解析时会尝试将字符串转换为目标类型。

/// Spring 风格的带类型标记的字符串值。
///
/// 对应 Spring 的 `TypedStringValue`。
///
/// 用于在 Bean 定义中表示一个字符串值，该值可能需要在解析时转换为
/// 特定的目标类型（例如，从配置文件读取的字符串值需要转换为数字或布尔值）。
///
/// ## 示例
///
/// ```rust,ignore
/// use vernal_beans::typed_string_value::TypedStringValue;
///
/// let val = TypedStringValue::new("42");
/// assert_eq!(val.value(), "42");
/// assert!(val.get_target_type_name().is_none());
///
/// let typed = TypedStringValue::with_target_type("true", "bool");
/// assert_eq!(typed.get_target_type_name(), Some("bool"));
/// ```
#[derive(Debug, Clone)]
pub struct TypedStringValue {
    /// 原始字符串值。
    value: String,
    /// 可选的目标类型名称。
    target_type_name: Option<String>,
}

impl TypedStringValue {
    /// 创建一个新的 TypedStringValue，不指定目标类型。
    ///
    /// # 参数
    ///
    /// * `value` — 原始字符串值
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            target_type_name: None,
        }
    }

    /// 创建一个带有目标类型的 TypedStringValue。
    ///
    /// # 参数
    ///
    /// * `value` — 原始字符串值
    /// * `type_name` — 目标类型名称（如 `"i32"`, `"bool"`, `"std::path::PathBuf"`）
    pub fn with_target_type(value: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            target_type_name: Some(type_name.into()),
        }
    }

    /// 获取原始的字符串值。
    pub fn value(&self) -> &str {
        &self.value
    }

    /// 设置目标类型名称。
    ///
    /// # 参数
    ///
    /// * `type_name` — 目标类型的完整 Rust 名称
    pub fn set_target_type_name(&mut self, type_name: impl Into<String>) {
        self.target_type_name = Some(type_name.into());
    }

    /// 获取目标类型名称。
    ///
    /// 返回 `Some(name)` 表示字符串值应在解析时转换为指定类型。
    /// 返回 `None` 表示没有指定目标类型。
    pub fn get_target_type_name(&self) -> Option<&str> {
        self.target_type_name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_typed_string_value() {
        let val = TypedStringValue::new("hello");
        assert_eq!(val.value(), "hello");
        assert!(val.get_target_type_name().is_none());
    }

    #[test]
    fn test_with_target_type() {
        let val = TypedStringValue::with_target_type("42", "i32");
        assert_eq!(val.value(), "42");
        assert_eq!(val.get_target_type_name(), Some("i32"));
    }

    #[test]
    fn test_set_target_type_name() {
        let mut val = TypedStringValue::new("true");
        assert!(val.get_target_type_name().is_none());
        val.set_target_type_name("bool");
        assert_eq!(val.get_target_type_name(), Some("bool"));
    }

    #[test]
    fn test_clone() {
        let val = TypedStringValue::with_target_type("3.14", "f64");
        let cloned = val.clone();
        assert_eq!(cloned.value(), "3.14");
        assert_eq!(cloned.get_target_type_name(), Some("f64"));
    }
}
