//! ServiceLocatorFactoryBean — 对应 Spring `org.springframework.beans.factory.config.ServiceLocatorFactoryBean`。
//!
//! 服务定位器工厂 Bean。

use std::collections::HashMap;

/// 服务定位器工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.ServiceLocatorFactoryBean`。
///
/// 用于创建服务定位器接口的实现。
/// 服务定位器接口定义了获取服务的方法，工厂会自动实现这些方法。
///
/// ## 使用场景
///
/// - 服务查找模式
/// - 依赖查找替代依赖注入
/// - 动态服务获取
#[derive(Debug)]
pub struct ServiceLocatorFactoryBean {
    /// 服务定位器接口名称。
    service_locator_interface: String,
    /// 服务映射（方法名 -> Bean 名称）。
    service_mapping: HashMap<String, String>,
}

impl ServiceLocatorFactoryBean {
    /// 创建新的 ServiceLocatorFactoryBean。
    pub fn new(service_locator_interface: impl Into<String>) -> Self {
        Self {
            service_locator_interface: service_locator_interface.into(),
            service_mapping: HashMap::new(),
        }
    }

    /// 获取服务定位器接口名称。
    pub fn service_locator_interface(&self) -> &str {
        &self.service_locator_interface
    }

    /// 添加服务映射。
    ///
    /// # 参数
    ///
    /// - `method_name` — 服务定位器接口中的方法名
    /// - `bean_name` — 对应的 Bean 名称
    pub fn add_service_mapping(
        &mut self,
        method_name: impl Into<String>,
        bean_name: impl Into<String>,
    ) {
        self.service_mapping
            .insert(method_name.into(), bean_name.into());
    }

    /// 获取服务映射。
    pub fn service_mapping(&self) -> &HashMap<String, String> {
        &self.service_mapping
    }

    /// 根据方法名获取 Bean 名称。
    pub fn get_bean_name(&self, method_name: &str) -> Option<&str> {
        self.service_mapping.get(method_name).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_locator_factory_bean_new() {
        let factory = ServiceLocatorFactoryBean::new("com.example.ServiceLocator");
        assert_eq!(
            factory.service_locator_interface(),
            "com.example.ServiceLocator"
        );
        assert!(factory.service_mapping().is_empty());
    }

    #[test]
    fn test_service_locator_factory_bean_add_mapping() {
        let mut factory = ServiceLocatorFactoryBean::new("com.example.ServiceLocator");
        factory.add_service_mapping("getUserService", "userService");
        factory.add_service_mapping("getOrderService", "orderService");

        assert_eq!(factory.get_bean_name("getUserService"), Some("userService"));
        assert_eq!(
            factory.get_bean_name("getOrderService"),
            Some("orderService")
        );
        assert_eq!(factory.get_bean_name("getUnknown"), None);
    }

    #[test]
    fn test_service_locator_factory_bean_mapping_count() {
        let mut factory = ServiceLocatorFactoryBean::new("com.example.ServiceLocator");
        factory.add_service_mapping("method1", "bean1");
        factory.add_service_mapping("method2", "bean2");
        factory.add_service_mapping("method3", "bean3");

        assert_eq!(factory.service_mapping().len(), 3);
    }
}
