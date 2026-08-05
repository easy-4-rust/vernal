//! AbstractBeanDefinitionReader — Spring 风格 Bean 定义读取器抽象基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AbstractBeanDefinitionReader`。
//!
//! 在 Spring 中，此抽象类是所有 Bean 定义读取器的基类，
//! 提供了资源加载器、类加载器等通用功能。
//! XML 读取器、Properties 读取器等都继承此类。
//!
//! ## 设计说明
//!
//! Spring 的 `AbstractBeanDefinitionReader` 持有 `BeanDefinitionRegistry` 引用，
//! 在 vernal 中通过记录注册表状态来实现等价语义。

use std::collections::HashMap;
use std::sync::Mutex;

/// 抽象 Bean 定义读取器基类。
///
/// 对应 Spring 的 `AbstractBeanDefinitionReader`。
///
/// 提供 Bean 定义读取的基础设施：
/// - 资源加载器名称管理
/// - Bean 类计数跟踪
/// - 资源读取次数记录
/// - 注册表大小跟踪
pub struct AbstractBeanDefinitionReader {
    /// 资源加载器名称
    resource_loader_name: String,
    /// 已加载的 Bean 类数量
    bean_class_count: usize,
    /// 已加载的 Bean 定义数量
    bean_definition_count: usize,
    /// 资源读取次数（resource_name -> count）
    resource_count: Mutex<HashMap<String, usize>>,
    /// 注册表大小
    registry_size: Mutex<usize>,
    /// 错误数量
    error_count: Mutex<usize>,
}

impl AbstractBeanDefinitionReader {
    /// 创建新的抽象 Bean 定义读取器。
    ///
    /// # 参数
    /// - `resource_loader_name` — 资源加载器名称
    pub fn new(resource_loader_name: impl Into<String>) -> Self {
        Self {
            resource_loader_name: resource_loader_name.into(),
            bean_class_count: 0,
            bean_definition_count: 0,
            resource_count: Mutex::new(HashMap::new()),
            registry_size: Mutex::new(0),
            error_count: Mutex::new(0),
        }
    }

    /// 获取资源加载器名称。
    pub fn resource_loader_name(&self) -> &str {
        &self.resource_loader_name
    }

    /// 递增 Bean 类计数。
    pub fn increment_bean_class_count(&mut self) {
        self.bean_class_count += 1;
    }

    /// 获取已加载的 Bean 类数量。
    pub fn get_bean_class_count(&self) -> usize {
        self.bean_class_count
    }

    /// 重置 Bean 类计数。
    pub fn reset_bean_class_count(&mut self) {
        self.bean_class_count = 0;
    }

    /// 递增 Bean 定义计数。
    pub fn increment_bean_definition_count(&mut self) {
        self.bean_definition_count += 1;
    }

    /// 获取已加载的 Bean 定义数量。
    pub fn get_bean_definition_count(&self) -> usize {
        self.bean_definition_count
    }

    /// 记录资源读取次数。
    ///
    /// 对应 Spring 的资源加载跟踪。
    pub fn record_resource(&self, resource: String, count: usize) {
        self.resource_count.lock().unwrap().insert(resource, count);
    }

    /// 获取指定资源的读取次数。
    pub fn resource_count(&self, resource: &str) -> Option<usize> {
        self.resource_count.lock().unwrap().get(resource).copied()
    }

    /// 获取已记录的资源数量。
    pub fn recorded_resource_count(&self) -> usize {
        self.resource_count.lock().unwrap().len()
    }

    /// 设置注册表大小。
    pub fn set_registry_size(&self, size: usize) {
        *self.registry_size.lock().unwrap() = size;
    }

    /// 获取注册表大小。
    pub fn registry_size(&self) -> usize {
        *self.registry_size.lock().unwrap()
    }

    /// 递增错误计数。
    pub fn increment_error_count(&self) {
        *self.error_count.lock().unwrap() += 1;
    }

    /// 获取错误数量。
    pub fn error_count(&self) -> usize {
        *self.error_count.lock().unwrap()
    }

    /// 是否有错误。
    pub fn has_errors(&self) -> bool {
        *self.error_count.lock().unwrap() > 0
    }
}

impl Default for AbstractBeanDefinitionReader {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_reader_with_resource_loader() {
        let reader = AbstractBeanDefinitionReader::new("classpath");
        assert_eq!(reader.resource_loader_name(), "classpath");
        assert_eq!(reader.get_bean_class_count(), 0);
        assert_eq!(reader.get_bean_definition_count(), 0);
    }

    #[test]
    fn increment_bean_class_count() {
        let mut reader = AbstractBeanDefinitionReader::new("test");
        reader.increment_bean_class_count();
        reader.increment_bean_class_count();
        assert_eq!(reader.get_bean_class_count(), 2);
    }

    #[test]
    fn reset_bean_class_count() {
        let mut reader = AbstractBeanDefinitionReader::new("test");
        reader.increment_bean_class_count();
        reader.increment_bean_class_count();
        reader.reset_bean_class_count();
        assert_eq!(reader.get_bean_class_count(), 0);
    }

    #[test]
    fn resource_count_tracking() {
        let reader = AbstractBeanDefinitionReader::new("test");
        reader.record_resource("config.xml".to_string(), 3);
        reader.record_resource("app.xml".to_string(), 1);

        assert_eq!(reader.resource_count("config.xml"), Some(3));
        assert_eq!(reader.resource_count("app.xml"), Some(1));
        assert_eq!(reader.resource_count("missing.xml"), None);
        assert_eq!(reader.recorded_resource_count(), 2);
    }

    #[test]
    fn registry_size_tracking() {
        let reader = AbstractBeanDefinitionReader::new("test");
        reader.set_registry_size(10);
        assert_eq!(reader.registry_size(), 10);
    }

    #[test]
    fn error_count_tracking() {
        let reader = AbstractBeanDefinitionReader::new("test");
        assert!(!reader.has_errors());
        assert_eq!(reader.error_count(), 0);

        reader.increment_error_count();
        reader.increment_error_count();
        assert!(reader.has_errors());
        assert_eq!(reader.error_count(), 2);
    }
}
