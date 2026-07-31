//! PropertyPathFactoryBean — 对应 Spring `org.springframework.beans.factory.config.PropertyPathFactoryBean`。
//!
//! 属性路径工厂 Bean，用于通过属性路径获取 Bean 的属性值。

use std::any::Any;
use std::sync::Arc;

/// 属性路径工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.PropertyPathFactoryBean`。
///
/// 用于通过属性路径获取 Bean 的属性值。
/// 例如：`myBean.nestedBean.property`。
///
/// ## 使用场景
///
/// - 获取嵌套 Bean 的属性值
/// - 通过属性路径注入属性
/// - 链式属性访问
#[derive(Debug)]
pub struct PropertyPathFactoryBean {
    /// 目标 Bean 名称。
    target_bean_name: String,
    /// 属性路径。
    property_path: String,
}

impl PropertyPathFactoryBean {
    /// 创建新的 PropertyPathFactoryBean。
    pub fn new(
        target_bean_name: impl Into<String>,
        property_path: impl Into<String>,
    ) -> Self {
        Self {
            target_bean_name: target_bean_name.into(),
            property_path: property_path.into(),
        }
    }

    /// 获取目标 Bean 名称。
    pub fn target_bean_name(&self) -> &str {
        &self.target_bean_name
    }

    /// 获取属性路径。
    pub fn property_path(&self) -> &str {
        &self.property_path
    }

    /// 获取完整的属性路径（Bean 名称 + 属性路径）。
    pub fn full_path(&self) -> String {
        format!("{}.{}", self.target_bean_name, self.property_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_path_factory_bean_new() {
        let bean = PropertyPathFactoryBean::new("myBean", "name");
        assert_eq!(bean.target_bean_name(), "myBean");
        assert_eq!(bean.property_path(), "name");
        assert_eq!(bean.full_path(), "myBean.name");
    }

    #[test]
    fn test_property_path_factory_bean_nested() {
        let bean = PropertyPathFactoryBean::new("orderService", "customer.address.city");
        assert_eq!(bean.target_bean_name(), "orderService");
        assert_eq!(bean.property_path(), "customer.address.city");
        assert_eq!(bean.full_path(), "orderService.customer.address.city");
    }

    #[test]
    fn test_property_path_factory_bean_empty_path() {
        let bean = PropertyPathFactoryBean::new("bean1", "");
        assert_eq!(bean.property_path(), "");
        assert_eq!(bean.full_path(), "bean1.");
    }
}
