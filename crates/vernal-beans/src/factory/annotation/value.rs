//! Value — Spring 风格 @Value 注解标记。
//!
//! 对应 Java 注解：`org.springframework.beans.factory.annotation.Value`。
//!
//! 标记一个字段或方法参数为值注入点。
//!
//! 支持三种形式：
//! - 字面量值：`@Value("hello")`
//! - 属性占位符：`@Value("${app.name}")`
//! - SpEL 表达式：`@Value("#{bean.property}")`

/// Spring 风格的 @Value 注解标记。
///
/// 对应 Spring 的 `@Value`。
///
/// 标记一个字段或方法参数为值注入点。
pub struct Value {
    value: String,
}

impl Value {
    /// 创建 @Value 注解。
    pub fn new(value: String) -> Self {
        Self { value }
    }

    /// 获取注解值表达式。
    pub fn value(&self) -> &str {
        &self.value
    }

    /// 值表达式长度。
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// 值表达式是否为空。
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// 是否为属性占位符（以 `${` 开头）。
    pub fn is_placeholder(&self) -> bool {
        self.value.starts_with("${") && self.value.ends_with('}')
    }

    /// 是否为 SpEL 表达式（以 `#{` 开头）。
    pub fn is_spel(&self) -> bool {
        self.value.starts_with("#{") && self.value.ends_with('}')
    }

    /// 是否为字面量值（非占位符、非 SpEL）。
    pub fn is_literal(&self) -> bool {
        !self.is_placeholder() && !self.is_spel()
    }

    /// 提取占位符的键名（去除 `${` 和 `}`）。
    ///
    /// 如果不是占位符，返回 `None`。
    pub fn placeholder_key(&self) -> Option<&str> {
        if self.is_placeholder() {
            Some(&self.value[2..self.value.len() - 1])
        } else {
            None
        }
    }

    /// 提取 SpEL 表达式内容（去除 `#{` 和 `}`）。
    ///
    /// 如果不是 SpEL，返回 `None`。
    pub fn spel_expression(&self) -> Option<&str> {
        if self.is_spel() {
            Some(&self.value[2..self.value.len() - 1])
        } else {
            None
        }
    }

    /// 创建一个属性占位符 @Value 注解。
    pub fn placeholder(key: impl Into<String>) -> Self {
        let key = key.into();
        Self {
            value: format!("${{{}}}", key),
        }
    }

    /// 创建一个 SpEL 表达式 @Value 注解。
    pub fn spel(expression: impl Into<String>) -> Self {
        let expression = expression.into();
        Self {
            value: format!("#{{{}}}", expression),
        }
    }

    /// 获取原始值表达式。
    pub fn raw_value(&self) -> &str {
        &self.value
    }

    /// 判断值是否包含嵌套占位符（如 `${a.${b}}`）。
    pub fn has_nested_placeholder(&self) -> bool {
        if !self.is_placeholder() {
            return false;
        }
        let inner = &self.value[2..self.value.len() - 1];
        inner.contains("${")
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::new(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_value() {
        let v = Value::new("hello".to_string());
        assert!(v.is_literal());
        assert!(!v.is_placeholder());
        assert!(!v.is_spel());
    }

    #[test]
    fn placeholder_value() {
        let v = Value::new("${app.name}".to_string());
        assert!(v.is_placeholder());
        assert!(!v.is_literal());
        assert_eq!(v.placeholder_key(), Some("app.name"));
    }

    #[test]
    fn spel_expression() {
        let v = Value::new("#{bean.property}".to_string());
        assert!(v.is_spel());
        assert!(!v.is_literal());
        assert_eq!(v.spel_expression(), Some("bean.property"));
    }

    #[test]
    fn empty_value() {
        let v = Value::default();
        assert!(v.is_empty());
        assert!(v.is_literal());
    }

    #[test]
    fn placeholder_factory() {
        let v = Value::placeholder("app.name");
        assert!(v.is_placeholder());
        assert_eq!(v.placeholder_key(), Some("app.name"));
        assert_eq!(v.value(), "${app.name}");
    }

    #[test]
    fn spel_factory() {
        let v = Value::spel("bean.property");
        assert!(v.is_spel());
        assert_eq!(v.spel_expression(), Some("bean.property"));
        assert_eq!(v.value(), "#{bean.property}");
    }

    #[test]
    fn raw_value_returns_original() {
        let v = Value::new("hello".to_string());
        assert_eq!(v.raw_value(), "hello");
    }

    #[test]
    fn has_nested_placeholder() {
        let v1 = Value::new("${a.${b}}".to_string());
        assert!(v1.has_nested_placeholder());

        let v2 = Value::new("${simple}".to_string());
        assert!(!v2.has_nested_placeholder());

        let v3 = Value::new("literal".to_string());
        assert!(!v3.has_nested_placeholder());
    }
}
