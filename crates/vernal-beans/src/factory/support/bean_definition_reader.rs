//! BeanDefinitionReader — Spring 风格 Bean 定义读取器接口。
//!
//! 对应 Java 接口：`org.springframework.beans.factory.support.BeanDefinitionReader`。
//!
//! 从外部资源（XML、Properties、注解等）读取 Bean 定义。

use std::any::TypeId;

/// 资源描述符。
///
/// 对应 Spring 的 `org.springframework.core.io.Resource`。
#[derive(Debug, Clone)]
pub struct Resource {
    /// pub。
    pub name: String,
    /// pub。
    pub location: String,
    /// pub。
    pub exists: bool,
}

impl Resource {
    /// 创建一个新的实例。
    pub fn new(name: String, location: String) -> Self {
        Self { name, location, exists: true }
    }

    /// 执行description操作。
    pub fn description(&self) -> String {
        format!("Resource[{}@{}]", self.name, self.location)
    }
}

/// Bean 定义读取器接口。
///
/// 对应 Spring 的 `BeanDefinitionReader`。
///
/// 从外部资源（XML、Properties、注解等）读取 Bean 定义。
pub trait BeanDefinitionReader: Send + Sync {
    /// 从资源加载 Bean 定义
    fn load_bean_definitions(&self, resource: &Resource) -> Result<usize, String>;

    /// 从多个资源批量加载
    fn load_bean_definitions_batch(&self, resources: &[Resource]) -> Result<usize, String> {
        let mut count = 0;
        for r in resources {
            count += self.load_bean_definitions(r)?;
        }
        Ok(count)
    }

    /// 获取已加载的 Bean 类数量
    fn get_bean_class_count(&self) -> usize;

    /// 获取已加载的 Bean 数量
    fn get_bean_definition_count(&self) -> usize;

    /// 获取资源加载器
    fn get_resource_loader_name(&self) -> &str;

    /// 获取类加载器
    fn get_class_loader_type_id(&self) -> Option<TypeId>;

    /// 获取 Bean 定义注册表
    fn get_registry_size(&self) -> usize;
}

/// 抽象 Bean 定义读取器基类。
///
/// 对应 Spring 的 `AbstractBeanDefinitionReader`。
pub struct AbstractBeanDefinitionReaderImpl {
    bean_class_count: std::sync::Mutex<usize>,
    bean_definition_count: std::sync::Mutex<usize>,
    registry_size: std::sync::Mutex<usize>,
    resource_loader_name: String,
    class_loader_type_id: std::sync::Mutex<Option<TypeId>>,
}

impl AbstractBeanDefinitionReaderImpl {
    /// 创建一个新的实例。
    pub fn new(resource_loader_name: String) -> Self {
        Self {
            bean_class_count: std::sync::Mutex::new(0),
            bean_definition_count: std::sync::Mutex::new(0),
            registry_size: std::sync::Mutex::new(0),
            resource_loader_name,
            class_loader_type_id: std::sync::Mutex::new(None),
        }
    }

    /// 获取incrementBean类数量。
    pub fn increment_bean_class_count(&self) {
        let mut count = self.bean_class_count.lock().unwrap();
        *count += 1;
    }

    /// 获取incrementBean定义数量。
    pub fn increment_bean_definition_count(&self) {
        let mut count = self.bean_definition_count.lock().unwrap();
        *count += 1;
    }

    /// 设置注册表size。
    pub fn set_registry_size(&self, size: usize) {
        *self.registry_size.lock().unwrap() = size;
    }

    /// 设置类加载器类型id。
    pub fn set_class_loader_type_id(&self, type_id: TypeId) {
        *self.class_loader_type_id.lock().unwrap() = Some(type_id);
    }
}

impl BeanDefinitionReader for AbstractBeanDefinitionReaderImpl {
    fn load_bean_definitions(&self, _resource: &Resource) -> Result<usize, String> {
        Ok(0)
    }

    fn get_bean_class_count(&self) -> usize {
        *self.bean_class_count.lock().unwrap()
    }

    fn get_bean_definition_count(&self) -> usize {
        *self.bean_definition_count.lock().unwrap()
    }

    fn get_resource_loader_name(&self) -> &str {
        &self.resource_loader_name
    }

    fn get_class_loader_type_id(&self) -> Option<TypeId> {
        *self.class_loader_type_id.lock().unwrap()
    }

    fn get_registry_size(&self) -> usize {
        *self.registry_size.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Resource ─────────────────────────────────────────────────────────

    #[test]
    fn resource_new() {
        let r = Resource::new("test.xml".to_string(), "/path/test.xml".to_string());
        assert_eq!(r.name, "test.xml");
        assert_eq!(r.location, "/path/test.xml");
        assert!(r.exists);
    }

    #[test]
    fn resource_description() {
        let r = Resource::new("test.xml".to_string(), "/path/test.xml".to_string());
        let desc = r.description();
        assert!(desc.contains("test.xml"));
        assert!(desc.contains("/path/test.xml"));
    }

    #[test]
    fn resource_clone() {
        let r = Resource::new("test.xml".to_string(), "/path/test.xml".to_string());
        let cloned = r.clone();
        assert_eq!(cloned.name, r.name);
        assert_eq!(cloned.location, r.location);
        assert_eq!(cloned.exists, r.exists);
    }

    #[test]
    fn resource_debug() {
        let r = Resource::new("test.xml".to_string(), "/path/test.xml".to_string());
        let debug = format!("{:?}", r);
        assert!(debug.contains("Resource"));
    }

    // ── AbstractBeanDefinitionReaderImpl ─────────────────────────────────

    #[test]
    fn new_reader() {
        let reader = AbstractBeanDefinitionReaderImpl::new("testLoader".to_string());
        assert_eq!(reader.get_resource_loader_name(), "testLoader");
        assert_eq!(reader.get_bean_class_count(), 0);
        assert_eq!(reader.get_bean_definition_count(), 0);
        assert_eq!(reader.get_registry_size(), 0);
        assert!(reader.get_class_loader_type_id().is_none());
    }

    #[test]
    fn increment_bean_class_count() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        reader.increment_bean_class_count();
        assert_eq!(reader.get_bean_class_count(), 1);
        reader.increment_bean_class_count();
        assert_eq!(reader.get_bean_class_count(), 2);
    }

    #[test]
    fn increment_bean_definition_count() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        reader.increment_bean_definition_count();
        assert_eq!(reader.get_bean_definition_count(), 1);
        reader.increment_bean_definition_count();
        reader.increment_bean_definition_count();
        assert_eq!(reader.get_bean_definition_count(), 3);
    }

    #[test]
    fn set_registry_size() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        reader.set_registry_size(10);
        assert_eq!(reader.get_registry_size(), 10);
        reader.set_registry_size(20);
        assert_eq!(reader.get_registry_size(), 20);
    }

    #[test]
    fn set_class_loader_type_id() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        assert!(reader.get_class_loader_type_id().is_none());
        reader.set_class_loader_type_id(TypeId::of::<String>());
        assert!(reader.get_class_loader_type_id().is_some());
        assert_eq!(reader.get_class_loader_type_id().unwrap(), TypeId::of::<String>());
    }

    #[test]
    fn load_bean_definitions_returns_zero() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        let resource = Resource::new("test".to_string(), "/test".to_string());
        let result = reader.load_bean_definitions(&resource).unwrap();
        assert_eq!(result, 0);
    }

    // ── load_bean_definitions_batch ──────────────────────────────────────

    #[test]
    fn load_bean_definitions_batch_empty() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        let result = reader.load_bean_definitions_batch(&[]).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn load_bean_definitions_batch_single() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        let resources = vec![
            Resource::new("a.xml".to_string(), "/a.xml".to_string()),
        ];
        let result = reader.load_bean_definitions_batch(&resources).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn load_bean_definitions_batch_multiple() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        let resources = vec![
            Resource::new("a.xml".to_string(), "/a.xml".to_string()),
            Resource::new("b.xml".to_string(), "/b.xml".to_string()),
            Resource::new("c.xml".to_string(), "/c.xml".to_string()),
        ];
        let result = reader.load_bean_definitions_batch(&resources).unwrap();
        assert_eq!(result, 0);
    }

    // ── Trait method coverage ────────────────────────────────────────────

    #[test]
    fn trait_get_resource_loader_name() {
        let reader = AbstractBeanDefinitionReaderImpl::new("myLoader".to_string());
        let trait_ref: &dyn BeanDefinitionReader = &reader;
        assert_eq!(trait_ref.get_resource_loader_name(), "myLoader");
    }

    #[test]
    fn trait_get_bean_class_count() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        reader.increment_bean_class_count();
        let trait_ref: &dyn BeanDefinitionReader = &reader;
        assert_eq!(trait_ref.get_bean_class_count(), 1);
    }

    #[test]
    fn trait_get_bean_definition_count() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        reader.increment_bean_definition_count();
        let trait_ref: &dyn BeanDefinitionReader = &reader;
        assert_eq!(trait_ref.get_bean_definition_count(), 1);
    }

    #[test]
    fn trait_get_class_loader_type_id() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        let trait_ref: &dyn BeanDefinitionReader = &reader;
        assert!(trait_ref.get_class_loader_type_id().is_none());
    }

    #[test]
    fn trait_get_registry_size() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        reader.set_registry_size(5);
        let trait_ref: &dyn BeanDefinitionReader = &reader;
        assert_eq!(trait_ref.get_registry_size(), 5);
    }

    #[test]
    fn trait_load_bean_definitions() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        let resource = Resource::new("test".to_string(), "/test".to_string());
        let trait_ref: &dyn BeanDefinitionReader = &reader;
        let result = trait_ref.load_bean_definitions(&resource).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn trait_load_bean_definitions_batch() {
        let reader = AbstractBeanDefinitionReaderImpl::new("loader".to_string());
        let resources = vec![
            Resource::new("a.xml".to_string(), "/a.xml".to_string()),
        ];
        let trait_ref: &dyn BeanDefinitionReader = &reader;
        let result = trait_ref.load_bean_definitions_batch(&resources).unwrap();
        assert_eq!(result, 0);
    }
}
