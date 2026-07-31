//! QualifierEntry — 对应 Spring QualifierEntry。
//!
//! 限定符条目。在 Bean 定义解析过程中，记录 Bean 的限定符信息。
//! 限定符用于在自动装配时区分同类型的多个 Bean 候选。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.QualifierEntry`。

use std::fmt;

use super::location::Location;

/// 限定符条目。
///
/// 对应 Spring 的 `QualifierEntry`。
///
/// 在解析 `@Qualifier` 注解或 XML 的 `<qualifier>` 元素时创建。
/// 限定符用于在有多个同类型 Bean 候选时进行精确匹配。
///
/// ## XML 示例
///
/// ```xml
/// <qualifier type="org.springframework.beans.factory.annotation.Qualifier"
///            value="mainDataSource"/>
/// ```
///
/// ## Rust 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::qualifier_entry::QualifierEntry;
///
/// let entry = QualifierEntry::new("mainDataSource");
/// assert_eq!(entry.qualifier_value(), "mainDataSource");
///
/// let typed = QualifierEntry::with_type(
///     "com.example.annotations.Main", "primary"
/// );
/// assert_eq!(typed.qualifier_type(), Some("com.example.annotations.Main"));
/// assert_eq!(typed.qualifier_value(), "primary");
/// ```
#[derive(Debug, Clone)]
pub struct QualifierEntry {
    /// 限定符类型名（通常是注解的全限定类名）。
    qualifier_type: Option<String>,
    /// 限定符值。
    qualifier_value: String,
    /// 来源位置。
    location: Option<Location>,
}

impl QualifierEntry {
    /// 创建一个新的限定符条目（仅值，无类型）。
    pub fn new(qualifier_value: impl Into<String>) -> Self {
        Self {
            qualifier_type: None,
            qualifier_value: qualifier_value.into(),
            location: None,
        }
    }

    /// 创建一个带类型的限定符条目。
    pub fn with_type(
        qualifier_type: impl Into<String>,
        qualifier_value: impl Into<String>,
    ) -> Self {
        Self {
            qualifier_type: Some(qualifier_type.into()),
            qualifier_value: qualifier_value.into(),
            location: None,
        }
    }

    /// 设置来源位置（链式构建）。
    #[must_use]
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    /// 返回限定符类型名。
    pub fn qualifier_type(&self) -> Option<&str> {
        self.qualifier_type.as_deref()
    }

    /// 返回限定符值。
    pub fn qualifier_value(&self) -> &str {
        &self.qualifier_value
    }

    /// 返回来源位置。
    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }
}

impl fmt::Display for QualifierEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ty) = &self.qualifier_type {
            write!(f, "Qualifier(type={}, value={})", ty, self.qualifier_value)
        } else {
            write!(f, "Qualifier(value={})", self.qualifier_value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let entry = QualifierEntry::new("main");
        assert_eq!(entry.qualifier_value(), "main");
        assert!(entry.qualifier_type().is_none());
    }

    #[test]
    fn test_with_type() {
        let entry = QualifierEntry::with_type("com.example.Qualifier", "primary");
        assert_eq!(entry.qualifier_type(), Some("com.example.Qualifier"));
        assert_eq!(entry.qualifier_value(), "primary");
    }

    #[test]
    fn test_with_location() {
        let loc = Location::new("beans.xml", 5, 1);
        let entry = QualifierEntry::new("main").with_location(loc);
        assert!(entry.location().is_some());
        assert_eq!(entry.location().unwrap().resource(), "beans.xml");
    }

    #[test]
    fn test_display() {
        let entry = QualifierEntry::with_type("MyQualifier", "primary");
        let display = format!("{}", entry);
        assert!(display.contains("MyQualifier"));
        assert!(display.contains("primary"));
    }

    #[test]
    fn test_display_value_only() {
        let entry = QualifierEntry::new("main");
        let display = format!("{}", entry);
        assert!(display.contains("main"));
        assert!(!display.contains("type="));
    }
}
