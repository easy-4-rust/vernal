//! ServiceFactoryBean — 对应 Java 类：org.springframework.beans.factory.serviceloader.ServiceFactoryBean。
//!
//! 对应 Spring beans.factory.serviceloader 包。
//!
//! 在 Spring 中，`ServiceFactoryBean` 是一个 `FactoryBean`，
//! 它使用 Java 的 `ServiceLoader` 机制来加载单个服务实现。
//! 当容器请求该 Bean 时，它返回 ServiceLoader 发现的第一个实现。
//!
//! ## 使用场景
//!
//! - SPI（Service Provider Interface）模式
//! - 插件架构中的服务发现
//! - 需要通过接口获取唯一实现

/// ServiceFactoryBean — Spring 风格的服务工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.serviceloader.ServiceFactoryBean`。
///
/// 使用服务注册表加载单个服务实现。
/// 当请求 Bean 时返回第一个注册的实现。
///
/// ## Java 对比
///
/// | Java | Rust |
/// |------|------|
/// | `ServiceFactoryBean` | `ServiceFactoryBean` |
/// | `ServiceLoader.load(Class)` | `service_implementations()` |
/// | `getObject()` 返回第一个实现 | `get_object()` |
#[derive(Debug)]
pub struct ServiceFactoryBean {
    /// 服务接口类型名
    service_type: String,
    /// 已注册的服务实现类名
    implementations: Vec<String>,
    /// 是否已初始化
    initialized: bool,
}

impl ServiceFactoryBean {
    /// 创建新的 ServiceFactoryBean。
    ///
    /// # 参数
    /// - `service_type` — 服务接口类型名
    pub fn new(service_type: impl Into<String>) -> Self {
        Self {
            service_type: service_type.into(),
            implementations: Vec::new(),
            initialized: false,
        }
    }

    /// 注册服务实现。
    ///
    /// 对应 Java 的 `META-INF/services/` 文件中的实现类。
    pub fn register_implementation(&mut self, impl_class: impl Into<String>) {
        self.implementations.push(impl_class.into());
    }

    /// 获取服务接口类型名。
    ///
    /// 对应 Java 的 `getObjectType()`。
    pub fn service_type(&self) -> &str {
        &self.service_type
    }

    /// 获取第一个服务实现。
    ///
    /// 对应 Java 的 `getObject()`。
    ///
    /// # 返回
    /// - `Some(impl_name)` — 第一个注册的实现类名
    /// - `None` — 无实现
    pub fn get_object(&self) -> Option<&str> {
        self.implementations.first().map(|s| s.as_str())
    }

    /// 获取所有已注册的实现。
    pub fn implementations(&self) -> &[String] {
        &self.implementations
    }

    /// 获取实现数量。
    pub fn implementation_count(&self) -> usize {
        self.implementations.len()
    }

    /// 初始化工厂 Bean。
    ///
    /// 对应 Spring 的 `InitializingBean.afterPropertiesSet()`。
    pub fn after_properties_set(&mut self) {
        self.initialized = true;
    }

    /// 是否已初始化。
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// 是否为单例（默认 true）。
    ///
    /// 对应 `FactoryBean.isSingleton()`。
    pub fn is_singleton(&self) -> bool {
        true
    }

    /// 清空所有注册的实现。
    pub fn clear(&mut self) {
        self.implementations.clear();
    }
}

impl Default for ServiceFactoryBean {
    fn default() -> Self {
        Self::new("java.lang.Object")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_factory_stores_service_type() {
        let factory = ServiceFactoryBean::new("com.example.Logger");
        assert_eq!(factory.service_type(), "com.example.Logger");
    }

    #[test]
    fn register_and_get_first_implementation() {
        let mut factory = ServiceFactoryBean::new("Logger");
        factory.register_implementation("ConsoleLogger");
        factory.register_implementation("FileLogger");

        assert_eq!(factory.get_object(), Some("ConsoleLogger"));
        assert_eq!(factory.implementation_count(), 2);
    }

    #[test]
    fn empty_factory_returns_none() {
        let factory = ServiceFactoryBean::new("Logger");
        assert!(factory.get_object().is_none());
    }

    #[test]
    fn after_properties_set_initializes() {
        let mut factory = ServiceFactoryBean::new("Logger");
        assert!(!factory.is_initialized());
        factory.after_properties_set();
        assert!(factory.is_initialized());
    }

    #[test]
    fn is_singleton_always_true() {
        let factory = ServiceFactoryBean::new("Logger");
        assert!(factory.is_singleton());
    }

    #[test]
    fn implementations_list() {
        let mut factory = ServiceFactoryBean::new("Logger");
        factory.register_implementation("A");
        factory.register_implementation("B");
        factory.register_implementation("C");

        let impls = factory.implementations();
        assert_eq!(impls.len(), 3);
        assert_eq!(impls[0], "A");
        assert_eq!(impls[1], "B");
        assert_eq!(impls[2], "C");
    }

    #[test]
    fn clear_removes_implementations() {
        let mut factory = ServiceFactoryBean::new("Logger");
        factory.register_implementation("impl");
        factory.clear();
        assert_eq!(factory.implementation_count(), 0);
    }

    #[test]
    fn default_service_type() {
        let factory = ServiceFactoryBean::default();
        assert_eq!(factory.service_type(), "java.lang.Object");
    }
}
