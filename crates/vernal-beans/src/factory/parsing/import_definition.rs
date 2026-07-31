//! ImportDefinition — 对应 Spring beans.factory.parsing.ImportDefinition。
//!
//! 导入定义。记录一个被导入的资源路径。在 XML 配置中，
//! `<import resource="classpath:other.xml"/>` 元素解析后会创建
//! 一个 ImportDefinition 实例。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.ImportDefinition`。

use std::fmt;

use super::location::Location;

/// 导入定义。
///
/// 对应 Spring 的 `ImportDefinition`。
///
/// 描述一个被导入的配置资源。在解析 `<import>` 元素时创建，
/// 通过 [`super::reader_event_listener::ReaderEventListener`]
/// 的 `import_registered` 回调通知监听器。
///
/// ## XML 示例
///
/// ```xml
/// <import resource="classpath:services.xml"/>
/// <import resource="file:///etc/app/datasource.xml"/>
/// ```
///
/// ## Rust 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::import_definition::ImportDefinition;
///
/// let import = ImportDefinition::new("classpath:services.xml");
/// assert_eq!(import.imported_resource(), "classpath:services.xml");
/// assert!(import.source_location().is_none());
/// ```
#[derive(Debug, Clone)]
pub struct ImportDefinition {
    /// 被导入的资源路径。
    imported_resource: String,
    /// 源位置信息。
    source_location: Option<Location>,
}

impl ImportDefinition {
    /// 创建一个新的导入定义。
    pub fn new(imported_resource: impl Into<String>) -> Self {
        Self {
            imported_resource: imported_resource.into(),
            source_location: None,
        }
    }

    /// 创建一个带位置信息的导入定义。
    pub fn with_location(
        imported_resource: impl Into<String>,
        location: Location,
    ) -> Self {
        Self {
            imported_resource: imported_resource.into(),
            source_location: Some(location),
        }
    }

    /// 返回被导入的资源路径。
    pub fn imported_resource(&self) -> &str {
        &self.imported_resource
    }

    /// 返回源位置信息（如果有）。
    pub fn source_location(&self) -> Option<&Location> {
        self.source_location.as_ref()
    }
}

impl fmt::Display for ImportDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Import '{}'", self.imported_resource)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_and_accessors() {
        let import = ImportDefinition::new("classpath:beans.xml");
        assert_eq!(import.imported_resource(), "classpath:beans.xml");
        assert!(import.source_location().is_none());
    }

    #[test]
    fn test_with_location() {
        let loc = Location::new("main.xml", 3, 1);
        let import = ImportDefinition::with_location("other.xml", loc);
        assert!(import.source_location().is_some());
        assert_eq!(import.source_location().unwrap().line_number(), 3);
    }

    #[test]
    fn test_display() {
        let import = ImportDefinition::new("file:///etc/app/cfg.xml");
        let display = format!("{}", import);
        assert!(display.contains("file:///etc/app/cfg.xml"));
    }
}
