//! ConstructorArgumentEntry — 对应 Spring ConstructorArgumentEntry。
//!
//! 构造器参数条目。在 Bean 定义解析过程中，记录一个构造器参数的
//! 索引、值类型和引用信息。用于构建 Bean 实例时传入正确的参数。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.ConstructorArgumentEntry`。

use std::fmt;

use super::location::Location;

/// 构造器参数条目。
///
/// 对应 Spring 的 `ConstructorArgumentEntry`。
///
/// 在解析 `<constructor-arg>` 元素时创建，记录参数的索引位置、
/// 类型名称和值信息。配合 [`super::parse_state::ParseState`]
/// 追踪当前解析的构造器参数。
///
/// ## XML 示例
///
/// ```xml
/// <constructor-arg index="0" type="java.lang.String" value="hello"/>
/// <constructor-arg ref="dataSource"/>
/// ```
///
/// ## Rust 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::constructor_argument_entry::ConstructorArgumentEntry;
///
/// let entry = ConstructorArgumentEntry::new(0)
///     .with_type_name("String")
///     .with_value("hello");
///
/// assert_eq!(entry.index(), 0);
/// assert_eq!(entry.type_name(), Some("String"));
/// assert_eq!(entry.value(), Some("hello"));
/// ```
#[derive(Debug, Clone)]
pub struct ConstructorArgumentEntry {
    /// 参数索引。
    index: usize,
    /// 参数类型名。
    type_name: Option<String>,
    /// 参数值（字面值或 Bean 引用名）。
    value: Option<String>,
    /// 是否为 Bean 引用。
    is_reference: bool,
    /// 来源位置。
    location: Option<Location>,
}

impl ConstructorArgumentEntry {
    /// 创建一个新的构造器参数条目。
    pub fn new(index: usize) -> Self {
        Self {
            index,
            type_name: None,
            value: None,
            is_reference: false,
            location: None,
        }
    }

    /// 设置参数类型名（链式构建）。
    #[must_use]
    pub fn with_type_name(mut self, type_name: impl Into<String>) -> Self {
        self.type_name = Some(type_name.into());
        self
    }

    /// 设置参数值（链式构建）。
    #[must_use]
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// 设置参数为 Bean 引用（链式构建）。
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

    /// 返回参数索引。
    pub fn index(&self) -> usize {
        self.index
    }

    /// 返回参数类型名。
    pub fn type_name(&self) -> Option<&str> {
        self.type_name.as_deref()
    }

    /// 返回参数值。
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

impl fmt::Display for ConstructorArgumentEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "constructor-arg[{}]", self.index)?;
        if let Some(ty) = &self.type_name {
            write!(f, " type={}", ty)?;
        }
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
        let entry = ConstructorArgumentEntry::new(0);
        assert_eq!(entry.index(), 0);
        assert!(entry.type_name().is_none());
        assert!(entry.value().is_none());
        assert!(!entry.is_reference());
    }

    #[test]
    fn test_with_type_and_value() {
        let entry = ConstructorArgumentEntry::new(1)
            .with_type_name("java.lang.String")
            .with_value("hello");
        assert_eq!(entry.type_name(), Some("java.lang.String"));
        assert_eq!(entry.value(), Some("hello"));
        assert!(!entry.is_reference());
    }

    #[test]
    fn test_with_reference() {
        let entry = ConstructorArgumentEntry::new(0).with_reference("dataSource");
        assert_eq!(entry.value(), Some("dataSource"));
        assert!(entry.is_reference());
    }

    #[test]
    fn test_display() {
        let entry = ConstructorArgumentEntry::new(2)
            .with_type_name("int")
            .with_value("42");
        let display = format!("{}", entry);
        assert!(display.contains("[2]"));
        assert!(display.contains("type=int"));
        assert!(display.contains("value=42"));
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn display_reference() {
        let entry = ConstructorArgumentEntry::new(0).with_reference("dataSource");
        let display = format!("{}", entry);
        assert!(display.contains("ref=dataSource"));
    }

    #[test]
    fn display_no_type_no_value() {
        let entry = ConstructorArgumentEntry::new(0);
        let display = format!("{}", entry);
        assert_eq!(display, "constructor-arg[0]");
    }

    #[test]
    fn display_with_type_no_value() {
        let entry = ConstructorArgumentEntry::new(1).with_type_name("String");
        let display = format!("{}", entry);
        assert!(display.contains("type=String"));
        assert!(!display.contains("value="));
    }

    #[test]
    fn with_location() {
        let entry =
            ConstructorArgumentEntry::new(0).with_location(Location::from_resource("test.xml"));
        assert!(entry.location().is_some());
    }

    #[test]
    fn location_none_by_default() {
        let entry = ConstructorArgumentEntry::new(0);
        assert!(entry.location().is_none());
    }

    #[test]
    fn clone_entry() {
        let entry = ConstructorArgumentEntry::new(0)
            .with_type_name("String")
            .with_value("hello");
        let cloned = entry.clone();
        assert_eq!(cloned.index(), 0);
        assert_eq!(cloned.type_name(), Some("String"));
        assert_eq!(cloned.value(), Some("hello"));
    }

    #[test]
    fn debug_format() {
        let entry = ConstructorArgumentEntry::new(0).with_value("test");
        let debug = format!("{:?}", entry);
        assert!(debug.contains("ConstructorArgumentEntry"));
    }
}
