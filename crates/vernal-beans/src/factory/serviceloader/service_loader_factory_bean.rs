//! ServiceLoaderFactoryBean — 对应 Java 类：org.springframework.beans.factory.serviceloader.ServiceLoaderFactoryBean。
//!
//! 对应 Spring beans.factory.serviceloader 包。
//!
//! 在 Spring 中，`ServiceLoaderFactoryBean` 是最通用的 ServiceLoader 工厂 Bean。
//! 它可以返回 `ServiceLoader` 实例本身（而非单个服务或服务列表），
//! 允许调用方控制加载过程。支持自定义类加载器和服务类型配置。
//!
//! ## 与 ServiceFactoryBean / ServiceListFactoryBean 的区别
//!
//! - `ServiceFactoryBean` — 返回第一个实现
//! - `ServiceListFactoryBean` — 返回所有实现的列表
//! - `ServiceLoaderFactoryBean` — 返回 ServiceLoader 实例本身

use std::any::Any;
use std::sync::Arc;

/// ServiceLoaderFactoryBean — Spring 风格的 ServiceLoader 工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.serviceloader.ServiceLoaderFactoryBean`。
///
/// 返回 ServiceLoader 实例，允许调用方控制服务加载过程。
/// 支持自定义类加载器、服务类型和服务接口配置。
///
/// ## Java 对比
///
/// | Java | Rust |
/// |------|------|
/// | `ServiceLoaderFactoryBean` | `ServiceLoaderFactoryBean` |
/// | `getObject()` 返回 `ServiceLoader<?>` | `get_loader()` |
/// | `setServiceType(Class)` | `set_service_type(name)` |
/// | `setBeanClassLoader(ClassLoader)` | `set_class_loader(name)` |
#[derive(Debug)]
pub struct ServiceLoaderFactoryBean {
    /// 服务接口类型名
    service_type: String,
    /// 类加载器名称
    class_loader: Option<String>,
    /// 已注册的服务实现
    implementations: Vec<String>,
    /// 是否已初始化
    initialized: bool,
    /// 是否为单例
    singleton: bool,
    /// 服务加载器配置属性
    config_properties: std::collections::HashMap<String, String>,
}

impl ServiceLoaderFactoryBean {
    /// 创建新的 ServiceLoaderFactoryBean。
    ///
    /// # 参数
    /// - `service_type` — 服务接口类型名
    pub fn new(service_type: impl Into<String>) -> Self {
        Self {
            service_type: service_type.into(),
            class_loader: None,
            implementations: Vec::new(),
            initialized: false,
            singleton: true,
            config_properties: std::collections::HashMap::new(),
        }
    }

    /// 设置服务接口类型。
    pub fn set_service_type(&mut self, service_type: impl Into<String>) {
        self.service_type = service_type.into();
    }

    /// 获取服务接口类型名。
    pub fn service_type(&self) -> &str {
        &self.service_type
    }

    /// 设置类加载器名称。
    ///
    /// 对应 Java 的 `setBeanClassLoader(ClassLoader)`。
    pub fn set_class_loader(&mut self, class_loader: impl Into<String>) {
        self.class_loader = Some(class_loader.into());
    }

    /// 获取类加载器名称。
    pub fn class_loader(&self) -> Option<&str> {
        self.class_loader.as_deref()
    }

    /// 注册服务实现。
    pub fn register_implementation(&mut self, impl_class: impl Into<String>) {
        self.implementations.push(impl_class.into());
    }

    /// 获取已注册的实现列表。
    pub fn implementations(&self) -> &[String] {
        &self.implementations
    }

    /// 获取实现数量。
    pub fn implementation_count(&self) -> usize {
        self.implementations.len()
    }

    /// 获取 ServiceLoader（返回所有实现）。
    ///
    /// 对应 Java 的 `getObject()`。
    pub fn get_loader(&self) -> &[String] {
        &self.implementations
    }

    /// 设置是否为单例。
    pub fn set_singleton(&mut self, singleton: bool) {
        self.singleton = singleton;
    }

    /// 是否为单例。
    pub fn is_singleton(&self) -> bool {
        self.singleton
    }

    /// 设置配置属性。
    pub fn set_config_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.config_properties.insert(key.into(), value.into());
    }

    /// 获取配置属性。
    pub fn config_property(&self, key: &str) -> Option<&str> {
        self.config_properties.get(key).map(|s| s.as_str())
    }

    /// 获取配置属性数量。
    pub fn config_property_count(&self) -> usize {
        self.config_properties.len()
    }

    /// 初始化工厂 Bean。
    ///
    /// 对应 Spring 的 `InitializingBean.afterPropertiesSet()`。
    pub fn after_properties_set(&mut self) -> Result<(), String> {
        if self.service_type.is_empty() {
            return Err("serviceType must not be empty".to_string());
        }
        self.initialized = true;
        Ok(())
    }

    /// 是否已初始化。
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// 清空所有注册的实现。
    pub fn clear(&mut self) {
        self.implementations.clear();
        self.config_properties.clear();
    }
}

impl Default for ServiceLoaderFactoryBean {
    fn default() -> Self {
        Self::new("java.lang.Object")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_factory_stores_service_type() {
        let factory = ServiceLoaderFactoryBean::new("com.example.Service");
        assert_eq!(factory.service_type(), "com.example.Service");
    }

    #[test]
    fn register_and_get_implementations() {
        let mut factory = ServiceLoaderFactoryBean::new("Service");
        factory.register_implementation("Impl1");
        factory.register_implementation("Impl2");

        assert_eq!(factory.implementation_count(), 2);
        let loader = factory.get_loader();
        assert_eq!(loader[0], "Impl1");
        assert_eq!(loader[1], "Impl2");
    }

    #[test]
    fn after_properties_set_validates() {
        let mut factory = ServiceLoaderFactoryBean::new("Service");
        assert!(factory.after_properties_set().is_ok());
        assert!(factory.is_initialized());
    }

    #[test]
    fn after_properties_set_fails_on_empty_type() {
        let mut factory = ServiceLoaderFactoryBean::new("");
        assert!(factory.after_properties_set().is_err());
    }

    #[test]
    fn set_service_type() {
        let mut factory = ServiceLoaderFactoryBean::new("Old");
        factory.set_service_type("New");
        assert_eq!(factory.service_type(), "New");
    }

    #[test]
    fn class_loader() {
        let mut factory = ServiceLoaderFactoryBean::new("Service");
        assert!(factory.class_loader().is_none());
        factory.set_class_loader("myClassLoader");
        assert_eq!(factory.class_loader(), Some("myClassLoader"));
    }

    #[test]
    fn singleton_default_true() {
        let factory = ServiceLoaderFactoryBean::new("Service");
        assert!(factory.is_singleton());
    }

    #[test]
    fn set_singleton() {
        let mut factory = ServiceLoaderFactoryBean::new("Service");
        factory.set_singleton(false);
        assert!(!factory.is_singleton());
    }

    #[test]
    fn config_properties() {
        let mut factory = ServiceLoaderFactoryBean::new("Service");
        factory.set_config_property("timeout", "30");
        factory.set_config_property("retry", "3");

        assert_eq!(factory.config_property("timeout"), Some("30"));
        assert_eq!(factory.config_property("retry"), Some("3"));
        assert_eq!(factory.config_property("missing"), None);
        assert_eq!(factory.config_property_count(), 2);
    }

    #[test]
    fn clear_removes_everything() {
        let mut factory = ServiceLoaderFactoryBean::new("Service");
        factory.register_implementation("impl");
        factory.set_config_property("k", "v");

        factory.clear();
        assert_eq!(factory.implementation_count(), 0);
        assert_eq!(factory.config_property_count(), 0);
    }

    #[test]
    fn default_service_type() {
        let factory = ServiceLoaderFactoryBean::default();
        assert_eq!(factory.service_type(), "java.lang.Object");
    }
}
