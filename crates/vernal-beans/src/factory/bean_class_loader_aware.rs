//! BeanClassLoaderAware — Spring 风格的 Bean 类加载器感知接口（factory 子模块版）。
//!
//! 对应 Java 类：`org.springframework.beans.factory.BeanClassLoaderAware`。
//!
//! 在 Spring 中，`BeanClassLoaderAware` 允许 Bean 获取加载它的类加载器。
//! 实现此接口的 Bean 会在初始化前被注入类加载器。
//!
//! ## 与根模块 `bean_class_loader_aware` 的区别
//!
//! 根模块的 `bean_class_loader_aware` 定义了 trait 本身。
//! 此模块提供了工厂级别的类加载器管理实现。

use std::any::TypeId;
use std::sync::Mutex;

/// 类加载器感知的 Bean 管理器。
///
/// 跟踪哪些 Bean 实现了 `BeanClassLoaderAware` 接口，
/// 并管理类加载器的注入。
#[derive(Debug, Default)]
pub struct BeanClassLoaderAwareManager {
    /// 已注册的类加载器感知 Bean 名称。
    aware_beans: Mutex<Vec<String>>,
    /// 默认类加载器类型 ID。
    default_class_loader: Mutex<Option<TypeId>>,
}

impl BeanClassLoaderAwareManager {
    /// 创建类加载器感知管理器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个类加载器感知的 Bean。
    pub fn register_aware_bean(&self, bean_name: impl Into<String>) {
        self.aware_beans.lock().unwrap().push(bean_name.into());
    }

    /// 设置默认类加载器类型 ID。
    pub fn set_default_class_loader(&self, type_id: TypeId) {
        *self.default_class_loader.lock().unwrap() = Some(type_id);
    }

    /// 获取默认类加载器类型 ID。
    pub fn default_class_loader(&self) -> Option<TypeId> {
        *self.default_class_loader.lock().unwrap()
    }

    /// 获取已注册的类加载器感知 Bean 数量。
    pub fn aware_bean_count(&self) -> usize {
        self.aware_beans.lock().unwrap().len()
    }

    /// 获取所有已注册的 Bean 名称。
    pub fn aware_bean_names(&self) -> Vec<String> {
        self.aware_beans.lock().unwrap().clone()
    }

    /// 检查指定 Bean 是否已注册。
    pub fn is_aware_bean(&self, bean_name: &str) -> bool {
        self.aware_beans
            .lock()
            .unwrap()
            .contains(&bean_name.to_string())
    }

    /// 清除所有注册信息。
    pub fn clear(&self) {
        self.aware_beans.lock().unwrap().clear();
        *self.default_class_loader.lock().unwrap() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_aware_beans() {
        let manager = BeanClassLoaderAwareManager::new();
        manager.register_aware_bean("serviceA");
        manager.register_aware_bean("serviceB");
        assert_eq!(manager.aware_bean_count(), 2);
        assert!(manager.is_aware_bean("serviceA"));
        assert!(manager.is_aware_bean("serviceB"));
    }

    #[test]
    fn default_class_loader() {
        let manager = BeanClassLoaderAwareManager::new();
        assert!(manager.default_class_loader().is_none());
        manager.set_default_class_loader(TypeId::of::<String>());
        assert!(manager.default_class_loader().is_some());
    }

    #[test]
    fn aware_bean_names() {
        let manager = BeanClassLoaderAwareManager::new();
        manager.register_aware_bean("bean1");
        manager.register_aware_bean("bean2");
        let names = manager.aware_bean_names();
        assert!(names.contains(&"bean1".to_string()));
        assert!(names.contains(&"bean2".to_string()));
    }

    #[test]
    fn clear_removes_all() {
        let manager = BeanClassLoaderAwareManager::new();
        manager.register_aware_bean("bean");
        manager.set_default_class_loader(TypeId::of::<i32>());
        manager.clear();
        assert_eq!(manager.aware_bean_count(), 0);
        assert!(manager.default_class_loader().is_none());
    }

    #[test]
    fn unknown_bean_not_aware() {
        let manager = BeanClassLoaderAwareManager::new();
        assert!(!manager.is_aware_bean("unknown"));
    }
}
