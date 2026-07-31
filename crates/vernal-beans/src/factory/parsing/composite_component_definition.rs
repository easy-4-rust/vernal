//! CompositeComponentDefinition — 对应 Spring CompositeComponentDefinition。
//!
//! 组合组件定义，包含多个子组件定义。当一个 Bean 配置元素（如
//! `<beans>`）包含多个子组件时，使用 CompositeComponentDefinition
//! 将它们组合在一起。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.CompositeComponentDefinition`。

use std::fmt;

use super::abstract_component_definition::AbstractComponentDefinition;
use super::bean_component_definition::BeanComponentDefinition;

/// 组合组件定义。
///
/// 对应 Spring 的 `CompositeComponentDefinition`。
///
/// 包含多个子组件定义，用于表示一个配置元素（如 `<beans>` 根元素）
/// 下的所有 Bean 定义的聚合。在解析完成后，每个子组件定义被逐一
/// 注册到容器中。
///
/// ## Spring 语义
///
/// Spring 中，当 `<beans>` 元素被解析时，所有内部 `<bean>` 元素
/// 被收集到一个 `CompositeComponentDefinition` 中，然后通过
/// `ReaderEventListener` 逐个通知。
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::composite_component_definition::CompositeComponentDefinition;
/// use vernal_beans::factory::parsing::bean_component_definition::BeanComponentDefinition;
///
/// let mut composite = CompositeComponentDefinition::new("beans");
/// composite.add_nested_component(BeanComponentDefinition::new("bean1", "Class1"));
/// composite.add_nested_component(BeanComponentDefinition::new("bean2", "Class2"));
/// assert_eq!(composite.nested_component_count(), 2);
/// ```
#[derive(Debug)]
pub struct CompositeComponentDefinition {
    /// 基类组件定义。
    base: AbstractComponentDefinition,
    /// 嵌套的 Bean 组件定义。
    nested_components: Vec<BeanComponentDefinition>,
}

impl CompositeComponentDefinition {
    /// 创建一个新的组合组件定义。
    ///
    /// # 参数
    /// - `name` — 组合定义名称（通常为 "beans" 或父元素名称）
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: AbstractComponentDefinition::new(name),
            nested_components: Vec::new(),
        }
    }

    /// 添加一个嵌套的 Bean 组件定义。
    pub fn add_nested_component(&mut self, component: BeanComponentDefinition) {
        self.nested_components.push(component);
    }

    /// 返回嵌套组件数量。
    pub fn nested_component_count(&self) -> usize {
        self.nested_components.len()
    }

    /// 返回所有嵌套组件。
    pub fn nested_components(&self) -> &[BeanComponentDefinition] {
        &self.nested_components
    }

    /// 返回组合定义名称。
    pub fn name(&self) -> &str {
        self.base.name()
    }

    /// 返回底层的抽象组件定义。
    pub fn base(&self) -> &AbstractComponentDefinition {
        &self.base
    }

    /// 是否为空（没有嵌套组件）。
    pub fn is_empty(&self) -> bool {
        self.nested_components.is_empty()
    }
}

impl fmt::Display for CompositeComponentDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CompositeComponentDefinition '{}' with {} nested components",
            self.name(),
            self.nested_component_count()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_composite() {
        let composite = CompositeComponentDefinition::new("beans");
        assert_eq!(composite.name(), "beans");
        assert!(composite.is_empty());
        assert_eq!(composite.nested_component_count(), 0);
    }

    #[test]
    fn test_add_nested_components() {
        let mut composite = CompositeComponentDefinition::new("root");
        composite.add_nested_component(BeanComponentDefinition::new("bean1", "Class1"));
        composite.add_nested_component(BeanComponentDefinition::new("bean2", "Class2"));
        composite.add_nested_component(BeanComponentDefinition::new("bean3", "Class3"));

        assert_eq!(composite.nested_component_count(), 3);
        assert!(!composite.is_empty());

        let components = composite.nested_components();
        assert_eq!(components[0].bean_name(), "bean1");
        assert_eq!(components[1].bean_name(), "bean2");
        assert_eq!(components[2].bean_name(), "bean3");
    }

    #[test]
    fn test_display() {
        let mut composite = CompositeComponentDefinition::new("app-context");
        composite.add_nested_component(BeanComponentDefinition::new("svc", "Svc"));
        let display = format!("{}", composite);
        assert!(display.contains("app-context"));
        assert!(display.contains("1 nested components"));
    }
}
