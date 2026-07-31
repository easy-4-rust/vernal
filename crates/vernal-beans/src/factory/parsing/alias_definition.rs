//! AliasDefinition — 对应 Spring beans.factory.parsing.AliasDefinition。
//!
//! Bean 别名定义。记录一个 Bean 名称的别名映射关系。在 XML 配置中，
//! `<alias name="beanName" alias="aliasName"/>` 元素解析后会创建
//! 一个 AliasDefinition 实例。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.AliasDefinition`。

use std::fmt;

use super::location::Location;

/// Bean 别名定义。
///
/// 对应 Spring 的 `AliasDefinition`。
///
/// 描述一个 Bean 名称的别名映射。在解析 `<alias>` 元素时创建，
/// 通过 [`super::reader_event_listener::ReaderEventListener`]
/// 的 `alias_registered` 回调通知监听器。
///
/// ## XML 示例
///
/// ```xml
/// <alias name="dataSource" alias="dataSourceAlias"/>
/// ```
///
/// ## Rust 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::alias_definition::AliasDefinition;
/// use vernal_beans::factory::parsing::location::Location;
///
/// let alias = AliasDefinition::new("dataSource", "dataSourceAlias");
/// assert_eq!(alias.bean_name(), "dataSource");
/// assert_eq!(alias.alias(), "dataSourceAlias");
///
/// let alias_with_loc = AliasDefinition::with_location(
///     "dataSource", "dsAlias", Location::new("beans.xml", 5, 1),
/// );
/// assert!(alias_with_loc.location().is_some());
/// ```
#[derive(Debug, Clone)]
pub struct AliasDefinition {
    /// Bean 名称。
    bean_name: String,
    /// 别名。
    alias: String,
    /// 源位置信息。
    location: Option<Location>,
}

impl AliasDefinition {
    /// 创建一个新的别名定义。
    pub fn new(bean_name: impl Into<String>, alias: impl Into<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
            alias: alias.into(),
            location: None,
        }
    }

    /// 创建一个带位置信息的别名定义。
    pub fn with_location(
        bean_name: impl Into<String>,
        alias: impl Into<String>,
        location: Location,
    ) -> Self {
        Self {
            bean_name: bean_name.into(),
            alias: alias.into(),
            location: Some(location),
        }
    }

    /// 返回 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 返回别名。
    pub fn alias(&self) -> &str {
        &self.alias
    }

    /// 返回源位置信息（如果有）。
    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }
}

impl fmt::Display for AliasDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Alias '{}' -> '{}'", self.alias, self.bean_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_and_accessors() {
        let alias = AliasDefinition::new("dataSource", "ds");
        assert_eq!(alias.bean_name(), "dataSource");
        assert_eq!(alias.alias(), "ds");
        assert!(alias.location().is_none());
    }

    #[test]
    fn test_with_location() {
        let loc = Location::new("beans.xml", 10, 3);
        let alias = AliasDefinition::with_location("myBean", "myAlias", loc);
        assert!(alias.location().is_some());
        assert_eq!(alias.location().unwrap().resource(), "beans.xml");
    }

    #[test]
    fn test_display() {
        let alias = AliasDefinition::new("realBean", "aliasBean");
        let display = format!("{}", alias);
        assert!(display.contains("aliasBean"));
        assert!(display.contains("realBean"));
    }
}
