//! ToStringBeanDefinitionVisitor — 将 Bean 定义访问结果收集为字符串的实现。
//!
//! 对应 Spring 的 BeanDefinitionVisitor 的具体实现（概念等价）。
//!
//! 提供 `BeanDefinitionVisitor` trait 的一个实现，将访问到的
//! Bean 定义内容收集到 `String` 中。适用于日志输出、调试信息
//! 和 Bean 定义的简单序列化。

use std::any::Any;
use std::sync::Arc;

use crate::abstract_bean_definition::AbstractBeanDefinition;
use crate::bean_definition_visitor::BeanDefinitionVisitor;
use crate::constructor_argument_values::ConstructorArgumentValues;
use crate::mutable_property_values::MutablePropertyValues;

/// 将 Bean 定义访问结果收集为字符串的访问器。
///
/// 对应 Spring 的 `BeanDefinitionVisitor` 的字符串收集实现。
///
/// 访问 Bean 定义时，将构造参数、属性值和依赖关系等信息
/// 收集到一个内部 `String` 中。适用于调试、日志和监控场景。
///
/// ## 示例
///
/// ```rust,ignore
/// use vernal_beans::bean_definition_visitor_impl::ToStringBeanDefinitionVisitor;
/// use vernal_beans::bean_definition_visitor::BeanDefinitionVisitor;
/// use vernal_beans::abstract_bean_definition::AbstractBeanDefinition;
///
/// let mut visitor = ToStringBeanDefinitionVisitor::new("myBean");
/// let def = AbstractBeanDefinition::new();
/// visitor.visit_bean_definition(&def);
/// let result = visitor.to_string();
/// println!("{}", result);
/// ```
#[derive(Debug, Clone)]
pub struct ToStringBeanDefinitionVisitor {
    /// 收集的字符串内容。
    output: String,
    /// 可选的 Bean 名称前缀。
    bean_name: Option<String>,
}

impl ToStringBeanDefinitionVisitor {
    /// 创建一个新的 ToStringBeanDefinitionVisitor。
    ///
    /// # 参数
    ///
    /// * `bean_name` — Bean 的名称，将作为输出前缀
    pub fn new(bean_name: impl Into<String>) -> Self {
        Self {
            output: String::new(),
            bean_name: Some(bean_name.into()),
        }
    }

    /// 创建一个空的 ToStringBeanDefinitionVisitor（不显示 Bean 名称）。
    pub fn empty() -> Self {
        Self {
            output: String::new(),
            bean_name: None,
        }
    }

    /// 获取收集到的字符串内容。
    pub fn to_string(&self) -> &str {
        &self.output
    }

    /// 清空收集的内容。
    pub fn clear(&mut self) {
        self.output.clear();
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> Option<&str> {
        self.bean_name.as_deref()
    }
}

impl BeanDefinitionVisitor for ToStringBeanDefinitionVisitor {
    fn visit_bean_definition(&mut self, definition: &AbstractBeanDefinition) {
        if let Some(name) = &self.bean_name {
            self.output.push_str(&format!("BeanDefinition[{}]", name));
        } else {
            self.output.push_str("BeanDefinition[anonymous]");
        }

        if let Some(class_name) = definition.get_bean_class_name() {
            self.output.push_str(&format!(" class: {}", class_name));
        }

        self.output.push('\n');

        // 访问构造参数
        let cv = definition.constructor_argument_values();
        self.visit_constructor_argument_values(cv);

        // 访问属性值
        let pv = definition.property_values();
        self.visit_property_values(pv);

        // 访问依赖
        let depends_on = definition.get_depends_on();
        self.visit_depends_on(depends_on);
    }

    fn visit_constructor_argument_values(&mut self, values: &ConstructorArgumentValues) {
        let count = values.argument_count();
        if count > 0 {
            self.output
                .push_str(&format!("  constructor arguments ({}):\n", count));

            for (index, vh) in values.indexed_argument_values() {
                let mut line = format!("    [{}]", index);
                if let Some(type_name) = vh.type_name() {
                    line.push_str(&format!(" type: {}", type_name));
                }
                if let Some(name) = vh.name() {
                    line.push_str(&format!(" name: {}", name));
                }
                self.output.push_str(&line);
                self.output.push('\n');
            }

            for vh in values.generic_argument_values() {
                let mut line = "    [generic]".to_string();
                if let Some(type_name) = vh.type_name() {
                    line.push_str(&format!(" type: {}", type_name));
                }
                if let Some(name) = vh.name() {
                    line.push_str(&format!(" name: {}", name));
                }
                self.output.push_str(&line);
                self.output.push('\n');
            }
        } else {
            self.output.push_str("  constructor arguments: none\n");
        }
    }

    fn visit_property_values(&mut self, values: &MutablePropertyValues) {
        let pv_list = values.get_property_values();
        let count = pv_list.len();
        if count > 0 {
            self.output
                .push_str(&format!("  properties ({}):\n", count));
            for pv in pv_list {
                let value_str = format_any_value(pv.value());
                self.output
                    .push_str(&format!("    {} = {}\n", pv.name(), value_str));
            }
        } else {
            self.output.push_str("  properties: none\n");
        }
    }

    fn visit_depends_on(&mut self, depends_on: &[String]) {
        if !depends_on.is_empty() {
            self.output.push_str("  depends-on: ");
            for (i, dep) in depends_on.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.output.push_str(dep);
            }
            self.output.push('\n');
        } else {
            self.output.push_str("  depends-on: none\n");
        }
    }
}

/// 辅助函数：将 Any 值格式化为字符串。
fn format_any_value(value: &Arc<dyn Any + Send + Sync>) -> String {
    if let Some(s) = value.downcast_ref::<String>() {
        format!("\"{}\"", s)
    } else if let Some(i) = value.downcast_ref::<i32>() {
        format!("{}", i)
    } else if let Some(i) = value.downcast_ref::<i64>() {
        format!("{}", i)
    } else if let Some(f) = value.downcast_ref::<f64>() {
        format!("{}", f)
    } else if let Some(b) = value.downcast_ref::<bool>() {
        format!("{}", b)
    } else {
        format!("<{:?}>", value.type_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property_value::PropertyValue;

    #[test]
    fn test_empty_visitor() {
        let mut visitor = ToStringBeanDefinitionVisitor::new("testBean");
        let def = AbstractBeanDefinition::new();
        visitor.visit_bean_definition(&def);
        let result = visitor.to_string();
        assert!(result.contains("BeanDefinition[testBean]"));
        assert!(result.contains("constructor arguments: none"));
        assert!(result.contains("properties: none"));
        assert!(result.contains("depends-on: none"));
    }

    #[test]
    fn test_with_properties() {
        let mut visitor = ToStringBeanDefinitionVisitor::new("testBean");
        let mut def = AbstractBeanDefinition::new();
        def.set_bean_class_name("test::MyService");

        let mut props = def.property_values().clone();
        props.add(PropertyValue::new("name", Arc::new("value")));
        *def.get_property_values_mut() = props;

        visitor.visit_bean_definition(&def);
        let result = visitor.to_string();
        assert!(result.contains("test::MyService"));
        assert!(result.contains("properties (1)"));
    }

    #[test]
    fn test_clear() {
        let mut visitor = ToStringBeanDefinitionVisitor::new("testBean");
        let def = AbstractBeanDefinition::new();
        visitor.visit_bean_definition(&def);
        assert!(!visitor.to_string().is_empty());
        visitor.clear();
        assert!(visitor.to_string().is_empty());
    }

    #[test]
    fn test_bean_name() {
        let visitor = ToStringBeanDefinitionVisitor::new("myService");
        assert_eq!(visitor.bean_name(), Some("myService"));

        let empty = ToStringBeanDefinitionVisitor::empty();
        assert!(empty.bean_name().is_none());
    }
}
