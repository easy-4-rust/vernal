//! PropertyEntry — 对应 Spring PropertyEntry。
//!
//! 属性条目。在 Bean 定义解析过程中，记录一个属性注入的名称、值
//! 和类型信息。用于在构建 Bean 实例时注入属性值。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.PropertyEntry`。

use std::fmt;

use super::location::Location;

/// 属性注入条目。
///
/// 对应 Spring 的 `PropertyEntry`。
///
/// 在解析 `<property>` 元素时创建，记录属性名称、值和来源信息。
/// 配合 [`super::parse_state::ParseState`] 追踪当前解析的属性。
///
/// ## XML 示例
///
/// ```xml
/// <property name="timeout" value="3000"/>
/// <property name="dataSource" ref="myDataSource"/>
/// ```
///
/// ## Rust 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::property_entry::PropertyEntry;
///
/// let entry = PropertyEntry::new("timeout")
///     .with_value("3000");
/// assert_eq!(entry.property_name(), "timeout");
/// assert_eq!(entry.value(), Some("3000"));
/// assert!(!entry.is_reference());
/// ```
#[derive(Debug, Clone)]
pub struct PropertyEntry {
    /// 属性名称。
    property_name: String,
    /// 属性值（字面值或 Bean 引用名）。
    value: Option<String>,
    /// 是否为 Bean 引用。
    is_reference: bool,
    /// 来源位置。
    location: Option<Location>,
}

impl PropertyEntry {
    /// 创建一个新的属性条目。
    pub fn new(property_name: impl Into<String>) -> Self {
        Self {
            property_name: property_name.into(),
            value: None,
            is_reference: false,
            location: None,
        }
    }

    /// 设置字面值（链式构建）。
    #[must_use]
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self.is_reference = false;
        self
    }

    /// 设置 Bean 引用（链式构建）。
    #[must_use]
    pub fn with_reference(mut self, bean_name: impl Into<String>) -> Self {
        self.value = Some(bean_name.into());
        self.is_reference = true;
        self
    }

    /// 设置来源位置（链式构建）。
    #[must_use]
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    /// 返回属性名称。
    pub fn property_name(&self) -> &str {
        &self.property_name
    }

    /// 返回属性值。
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    /// 是否为 Bean 引用。
    pub fn is_reference(&self) -> bool {
        self.is_reference
    }

    /// 返回来源位置。
    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }
}

impl fmt::Display for PropertyEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "property '{}'", self.property_name)?;
        if let Some(val) = &self.value {
            if self.is_reference {
                write!(f, " ref={}", val)?;
            } else {
                write!(f, " value={}", val)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let entry = PropertyEntry::new("name");
        assert_eq!(entry.property_name(), "name");
        assert!(entry.value().is_none());
        assert!(!entry.is_reference());
    }

    #[test]
    fn test_with_value() {
        let entry = PropertyEntry::new("timeout").with_value("5000");
        assert_eq!(entry.value(), Some("5000"));
        assert!(!entry.is_reference());
    }

    #[test]
    fn test_with_reference() {
        let entry = PropertyEntry::new("dataSource").with_reference("myDataSource");
        assert_eq!(entry.value(), Some("myDataSource"));
        assert!(entry.is_reference());
    }

    #[test]
    fn test_display_with_value() {
        let entry = PropertyEntry::new("timeout").with_value("3000");
        let display = format!("{}", entry);
        assert!(display.contains("timeout"));
        assert!(display.contains("value=3000"));
    }

    #[test]
    fn test_display_with_reference() {
        let entry = PropertyEntry::new("ds").with_reference("dataSource");
        let display = format!("{}", entry);
        assert!(display.contains("ref=dataSource"));
    }
}
