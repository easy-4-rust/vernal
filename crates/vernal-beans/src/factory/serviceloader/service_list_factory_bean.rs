//! ServiceListFactoryBean — 对应 Java 类：org.springframework.beans.factory.serviceloader.ServiceListFactoryBean。
//!
//! 对应 Spring beans.factory.serviceloader 包。
//!
//! 在 Spring 中，`ServiceListFactoryBean` 是一个 `FactoryBean`，
//! 它使用 Java 的 `ServiceLoader` 机制来加载所有服务实现，
//! 并以 `List` 形式返回。与 `ServiceFactoryBean` 不同的是，
//! 它返回所有发现的实现，而不仅仅是第一个。
//!
//! ## 使用场景
//!
//! - 需要获取某接口的所有实现（策略模式）
//! - 插件系统中的多实现加载
//! - 事件监听器的批量注册

use std::any::Any;
use std::sync::Arc;

/// ServiceListFactoryBean — Spring 风格的服务列表工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.serviceloader.ServiceListFactoryBean`。
///
/// 使用服务注册表加载所有服务实现，以列表形式返回。
///
/// ## Java 对比
///
/// | Java | Rust |
/// |------|------|
/// | `ServiceListFactoryBean` | `ServiceListFactoryBean` |
/// | `getObject()` 返回 `List<?>` | `get_objects()` 返回 `&[String]` |
/// | `ServiceLoader.load(Class)` | `service_implementations()` |
#[derive(Debug)]
pub struct ServiceListFactoryBean {
    /// 服务接口类型名
    service_type: String,
    /// 已注册的服务实现类名
    implementations: Vec<String>,
    /// 是否已初始化
    initialized: bool,
    /// 是否暴露服务加载器上下文
    expose_class_loader: bool,
}

impl ServiceListFactoryBean {
    /// 创建新的 ServiceListFactoryBean。
    ///
    /// # 参数
    /// - `service_type` — 服务接口类型名
    pub fn new(service_type: impl Into<String>) -> Self {
        Self {
            service_type: service_type.into(),
            implementations: Vec::new(),
            initialized: false,
            expose_class_loader: false,
        }
    }

    /// 注册服务实现。
    pub fn register_implementation(&mut self, impl_class: impl Into<String>) {
        self.implementations.push(impl_class.into());
    }

    /// 批量注册服务实现。
    pub fn register_implementations(&mut self, impls: Vec<String>) {
        self.implementations.extend(impls);
    }

    /// 获取服务接口类型名。
    pub fn service_type(&self) -> &str {
        &self.service_type
    }

    /// 获取所有服务实现。
    ///
    /// 对应 Java 的 `getObject()` 返回 `List<?>`。
    pub fn get_objects(&self) -> &[String] {
        &self.implementations
    }

    /// 获取实现数量。
    pub fn implementation_count(&self) -> usize {
        self.implementations.len()
    }

    /// 按索引获取实现。
    pub fn get_implementation(&self, index: usize) -> Option<&str> {
        self.implementations.get(index).map(|s| s.as_str())
    }

    /// 检查是否包含指定实现。
    pub fn contains_implementation(&self, impl_class: &str) -> bool {
        self.implementations.iter().any(|s| s == impl_class)
    }

    /// 初始化工厂 Bean。
    pub fn after_properties_set(&mut self) {
        self.initialized = true;
    }

    /// 是否已初始化。
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// 设置是否暴露类加载器上下文。
    pub fn set_expose_class_loader(&mut self, expose: bool) {
        self.expose_class_loader = expose;
    }

    /// 是否暴露类加载器上下文。
    pub fn is_expose_class_loader(&self) -> bool {
        self.expose_class_loader
    }

    /// 清空所有注册的实现。
    pub fn clear(&mut self) {
        self.implementations.clear();
    }

    /// 是否为空（无实现）。
    pub fn is_empty(&self) -> bool {
        self.implementations.is_empty()
    }
}

impl Default for ServiceListFactoryBean {
    fn default() -> Self {
        Self::new("java.lang.Object")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_factory_is_empty() {
        let factory = ServiceListFactoryBean::new("Logger");
        assert!(factory.is_empty());
        assert_eq!(factory.implementation_count(), 0);
    }

    #[test]
    fn register_and_get_all() {
        let mut factory = ServiceListFactoryBean::new("Logger");
        factory.register_implementation("ConsoleLogger");
        factory.register_implementation("FileLogger");
        factory.register_implementation("DbLogger");

        let objects = factory.get_objects();
        assert_eq!(objects.len(), 3);
        assert_eq!(objects[0], "ConsoleLogger");
        assert_eq!(objects[2], "DbLogger");
    }

    #[test]
    fn register_implementations_batch() {
        let mut factory = ServiceListFactoryBean::new("Logger");
        factory.register_implementations(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
        ]);
        assert_eq!(factory.implementation_count(), 3);
    }

    #[test]
    fn get_by_index() {
        let mut factory = ServiceListFactoryBean::new("Logger");
        factory.register_implementation("first");
        factory.register_implementation("second");

        assert_eq!(factory.get_implementation(0), Some("first"));
        assert_eq!(factory.get_implementation(1), Some("second"));
        assert_eq!(factory.get_implementation(2), None);
    }

    #[test]
    fn contains_implementation() {
        let mut factory = ServiceListFactoryBean::new("Logger");
        factory.register_implementation("ConsoleLogger");

        assert!(factory.contains_implementation("ConsoleLogger"));
        assert!(!factory.contains_implementation("FileLogger"));
    }

    #[test]
    fn after_properties_set() {
        let mut factory = ServiceListFactoryBean::new("Logger");
        assert!(!factory.is_initialized());
        factory.after_properties_set();
        assert!(factory.is_initialized());
    }

    #[test]
    fn expose_class_loader() {
        let mut factory = ServiceListFactoryBean::new("Logger");
        assert!(!factory.is_expose_class_loader());
        factory.set_expose_class_loader(true);
        assert!(factory.is_expose_class_loader());
    }

    #[test]
    fn clear_removes_all() {
        let mut factory = ServiceListFactoryBean::new("Logger");
        factory.register_implementation("impl");
        factory.clear();
        assert!(factory.is_empty());
    }

    #[test]
    fn service_type_preserved() {
        let factory = ServiceListFactoryBean::new("com.example.Service");
        assert_eq!(factory.service_type(), "com.example.Service");
    }
}
