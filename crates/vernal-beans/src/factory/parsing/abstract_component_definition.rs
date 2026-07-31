//! AbstractComponentDefinition — 对应 Spring AbstractComponentDefinition。
//!
//! 抽象组件定义基类。提供 ComponentDefinition trait 的默认实现和
//! 通用字段访问器。BeanComponentDefinition 和 CompositeComponentDefinition
//! 都继承自此基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.AbstractComponentDefinition`。

use std::fmt;

/// 抽象组件定义基类。
///
/// 对应 Spring 的 `AbstractComponentDefinition`。
///
/// 提供组件定义的通用字段和默认实现。子类型（如
/// [`super::bean_component_definition::BeanComponentDefinition`]）
/// 可以覆写默认行为。
///
/// ## Spring 语义
///
/// 在 Spring 中，`AbstractComponentDefinition` 实现了
/// `ComponentDefinition` 接口，提供了：
/// - 组件名称（主名称 + 别名）
/// - 内部 Bean 定义列表
/// - Bean 引用列表
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::abstract_component_definition::AbstractComponentDefinition;
///
/// let def = AbstractComponentDefinition::new("myComponent");
/// assert_eq!(def.name(), "myComponent");
/// assert!(def.description().is_none());
/// ```
#[derive(Debug, Clone)]
pub struct AbstractComponentDefinition {
    /// 组件主名称。
    name: String,
    /// 组件别名列表。
    aliases: Vec<String>,
    /// 描述信息。
    description: Option<String>,
    /// 内部 Bean 定义名称列表。
    inner_bean_names: Vec<String>,
    /// Bean 引用名称列表。
    bean_references: Vec<String>,
}

impl AbstractComponentDefinition {
    /// 创建一个新的抽象组件定义。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            aliases: Vec::new(),
            description: None,
            inner_bean_names: Vec::new(),
            bean_references: Vec::new(),
        }
    }

    /// 设置描述信息（链式构建）。
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// 添加一个别名。
    pub fn add_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into());
    }

    /// 添加内部 Bean 名称。
    pub fn add_inner_bean_name(&mut self, name: impl Into<String>) {
        self.inner_bean_names.push(name.into());
    }

    /// 添加 Bean 引用。
    pub fn add_bean_reference(&mut self, name: impl Into<String>) {
        self.bean_references.push(name.into());
    }

    /// 返回组件主名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 返回所有别名。
    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    /// 返回描述信息。
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// 返回内部 Bean 名称列表。
    pub fn inner_bean_names(&self) -> &[String] {
        &self.inner_bean_names
    }

    /// 返回 Bean 引用列表。
    pub fn bean_references(&self) -> &[String] {
        &self.bean_references
    }

    /// 是否为 singleton（默认 true）。
    pub fn is_singleton(&self) -> bool {
        true
    }
}

impl fmt::Display for AbstractComponentDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ComponentDefinition '{}'", self.name)?;
        if !self.aliases.is_empty() {
            write!(f, " (aliases: {})", self.aliases.join(", "))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_and_accessors() {
        let def = AbstractComponentDefinition::new("myService");
        assert_eq!(def.name(), "myService");
        assert!(def.description().is_none());
        assert!(def.aliases().is_empty());
        assert!(def.is_singleton());
    }

    #[test]
    fn test_with_description() {
        let def = AbstractComponentDefinition::new("svc").with_description("My Service");
        assert_eq!(def.description(), Some("My Service"));
    }

    #[test]
    fn test_aliases_and_references() {
        let mut def = AbstractComponentDefinition::new("bean1");
        def.add_alias("alias1");
        def.add_alias("alias2");
        def.add_inner_bean_name("inner1");
        def.add_bean_reference("ref1");

        assert_eq!(def.aliases().len(), 2);
        assert_eq!(def.inner_bean_names(), &["inner1"]);
        assert_eq!(def.bean_references(), &["ref1"]);
    }

    #[test]
    fn test_display() {
        let mut def = AbstractComponentDefinition::new("myBean");
        def.add_alias("altName");
        let display = format!("{}", def);
        assert!(display.contains("myBean"));
        assert!(display.contains("altName"));
    }
}
