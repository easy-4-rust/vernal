//! BeanDefinitionVisitor — Spring 风格的 Bean 定义访问器 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionVisitor`。
//!
//! 提供访问 Bean 定义各部分（属性值、构造参数、依赖列表）的模板方法。
//! 默认实现均为空操作（no-op），子类可选择性覆盖。

use crate::abstract_bean_definition::AbstractBeanDefinition;
use crate::constructor_argument_values::ConstructorArgumentValues;
use crate::mutable_property_values::MutablePropertyValues;

/// Spring 风格的 Bean 定义访问器 trait。
///
/// 对应 Spring 的 `BeanDefinitionVisitor`。
///
/// 允许在 Bean 定义注册或解析过程中遍历/修改其内部结构。
/// 所有 visit 方法默认均为空操作。
pub trait BeanDefinitionVisitor {
    /// 访问整个 Bean 定义（AbstractBeanDefinition）。
    ///
    /// 默认行为：依次访问构造参数、属性值、依赖列表。
    fn visit_bean_definition(&mut self, definition: &AbstractBeanDefinition) {
        self.visit_constructor_argument_values(definition.constructor_argument_values());
        self.visit_property_values(definition.property_values());
        self.visit_depends_on(definition.get_depends_on());
    }

    /// 访问构造参数值集合。
    fn visit_constructor_argument_values(&mut self, _values: &ConstructorArgumentValues) {}

    /// 访问属性值集合。
    fn visit_property_values(&mut self, _values: &MutablePropertyValues) {}

    /// 访问依赖列表。
    fn visit_depends_on(&mut self, _depends_on: &[String]) {}
}
