//! Location — 对应 Spring beans.factory.parsing.Location。
//!
//! 表示 Bean 定义源文件中的位置信息（行号、列号），用于错误报告和
//! 调试。在 XML/Properties 等配置文件解析过程中，当发现 Bean 定义
//! 存在问题时，Location 能精确指出问题发生的位置。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.Location`。

use std::fmt;

/// 源文件中的位置信息。
///
/// 对应 Spring 的 `Location`。
///
/// 记录解析过程中遇到的位置（行号、列号），配合 [`super::Problem`]
/// 提供精确的错误定位。Rust 版本中，行号和列号从 1 开始计数；值为
/// 0 表示位置未知。
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::location::Location;
///
/// let loc = Location::new("config.xml", 10, 5);
/// assert_eq!(loc.resource(), "config.xml");
/// assert_eq!(loc.line_number(), 10);
/// assert_eq!(loc.column_number(), 5);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    /// 资源名称（文件路径或 URI）。
    resource: String,
    /// 行号（从 1 开始；0 表示未知）。
    line_number: usize,
    /// 列号（从 1 开始；0 表示未知）。
    column_number: usize,
}

impl Location {
    /// 未知位置的单例标识。
    pub const UNKNOWN: Location = Location {
        resource: String::new(),
        line_number: 0,
        column_number: 0,
    };

    /// 创建一个新的位置信息。
    ///
    /// # 参数
    /// - `resource` — 资源名称或文件路径
    /// - `line_number` — 行号（从 1 开始）
    /// - `column_number` — 列号（从 1 开始）
    pub fn new(resource: impl Into<String>, line_number: usize, column_number: usize) -> Self {
        Self {
            resource: resource.into(),
            line_number,
            column_number,
        }
    }

    /// 创建仅包含资源信息的位置（行号/列号未知）。
    pub fn from_resource(resource: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            line_number: 0,
            column_number: 0,
        }
    }

    /// 返回资源名称。
    pub fn resource(&self) -> &str {
        &self.resource
    }

    /// 返回行号（0 表示未知）。
    pub fn line_number(&self) -> usize {
        self.line_number
    }

    /// 返回列号（0 表示未知）。
    pub fn column_number(&self) -> usize {
        self.column_number
    }

    /// 位置是否已知。
    pub fn is_known(&self) -> bool {
        self.line_number > 0
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line_number > 0 {
            write!(
                f,
                "Resource [{}] - line {}, column {}",
                self.resource, self.line_number, self.column_number
            )
        } else if !self.resource.is_empty() {
            write!(f, "Resource [{}]", self.resource)
        } else {
            write!(f, "Unknown location")
        }
    }
}

impl Default for Location {
    fn default() -> Self {
        Self::UNKNOWN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_creation_and_accessors() {
        let loc = Location::new("applicationContext.xml", 42, 15);
        assert_eq!(loc.resource(), "applicationContext.xml");
        assert_eq!(loc.line_number(), 42);
        assert_eq!(loc.column_number(), 15);
        assert!(loc.is_known());
    }

    #[test]
    fn test_location_unknown() {
        let loc = Location::UNKNOWN;
        assert!(!loc.is_known());
        assert_eq!(loc.line_number(), 0);
        assert_eq!(loc.column_number(), 0);
        assert_eq!(loc, Location::default());
    }

    #[test]
    fn test_location_from_resource() {
        let loc = Location::from_resource("beans.xml");
        assert_eq!(loc.resource(), "beans.xml");
        assert!(!loc.is_known());
    }

    #[test]
    fn test_location_display() {
        let known = Location::new("test.xml", 10, 5);
        assert_eq!(
            format!("{}", known),
            "Resource [test.xml] - line 10, column 5"
        );

        let resource_only = Location::from_resource("beans.xml");
        assert_eq!(format!("{}", resource_only), "Resource [beans.xml]");

        let unknown = Location::UNKNOWN;
        assert_eq!(format!("{}", unknown), "Unknown location");
    }
}
