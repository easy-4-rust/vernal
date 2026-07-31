//! FactoryBeanRegistrySupport — Spring 风格的 FactoryBean 注册表支持。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.FactoryBeanRegistrySupport`。
//!
//! 缓存 FactoryBean 创建的对象，避免重复创建。

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Spring 风格的 FactoryBean 注册表支持。
///
/// 对应 Spring 的 `FactoryBeanRegistrySupport`。
///
/// 维护 FactoryBean 产品对象的缓存，以及 FactoryBean 类型映射。
///
/// ## 缓存策略
///
/// - 单例 FactoryBean 的产品被缓存，后续请求返回缓存值
/// - 非单例 FactoryBean 每次都创建新产品（不缓存）
pub struct FactoryBeanRegistrySupport {
    /// FactoryBean 产品缓存（Bean 名称 -> 产品实例）。
    factory_bean_object_cache: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// FactoryBean 类型映射（TypeId -> Bean 名称）。
    factory_bean_types: Mutex<HashMap<TypeId, String>>,
    /// 已知的 FactoryBean 名称集合。
    factory_bean_names: Mutex<Vec<String>>,
}

impl FactoryBeanRegistrySupport {
    /// 创建 FactoryBean 注册表支持。
    pub fn new() -> Self {
        Self {
            factory_bean_object_cache: Mutex::new(HashMap::new()),
            factory_bean_types: Mutex::new(HashMap::new()),
            factory_bean_names: Mutex::new(Vec::new()),
        }
    }

    /// 获取 FactoryBean 创建的对象（带缓存）。
    ///
    /// 对应 Spring 的 `FactoryBeanRegistrySupport.getObjectFromFactoryBean`。
    ///
    /// # 参数
    ///
    /// - `bean_name` — FactoryBean 的名称
    /// - `factory` — 获取产品的工厂闭包（通常调用 `FactoryBean.getObject()`）
    ///
    /// # 缓存行为
    ///
    /// - 如果缓存中已有该 Bean 名称的产品，直接返回缓存值
    /// - 否则调用工厂闭包创建产品，缓存后返回
    ///
    /// # 错误
    ///
    /// 工厂闭包返回 `Err` 时，此方法传播该错误。
    pub fn get_object_from_factory_bean_with_closure(
        &self,
        bean_name: &str,
        factory: &dyn Fn() -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // 检查缓存
        {
            let cache = self.factory_bean_object_cache.lock().unwrap();
            if let Some(cached) = cache.get(bean_name) {
                return Ok(Arc::clone(cached));
            }
        }

        // 创建产品
        let object = factory()?;

        // 缓存结果
        {
            let mut cache = self.factory_bean_object_cache.lock().unwrap();
            cache.insert(bean_name.to_string(), Arc::clone(&object));
        }

        Ok(object)
    }

    /// 获取 FactoryBean 创建的对象（带缓存，通过类型擦除的 FactoryBean）。
    ///
    /// 此方法接受 `&dyn Any` 类型的 FactoryBean 引用。
    /// 由于 Rust 的 trait object 限制，实际调用需要上层
    /// 将 `Arc<dyn FactoryBean>` 转换为具体类型后调用。
    ///
    /// # 参数
    ///
    /// - `factory_bean` — FactoryBean 实例（类型擦除）
    /// - `bean_name` — Bean 名称
    pub fn get_object_from_factory_bean(
        &self,
        _factory_bean: &dyn Any,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // 检查缓存
        {
            let cache = self.factory_bean_object_cache.lock().unwrap();
            if let Some(cached) = cache.get(bean_name) {
                return Ok(Arc::clone(cached));
            }
        }

        // 无法从 &dyn Any 调用 FactoryBean.getObject()，
        // 上层应使用 get_object_from_factory_bean_with_closure 或直接管理缓存。
        Err(format!(
            "FactoryBean '{}' 无法通过类型擦除引用调用 getObject()，请使用 get_object_from_factory_bean_with_closure",
            bean_name
        )
        .into())
    }

    /// 注册 FactoryBean 的类型映射。
    ///
    /// 用于按 TypeId 查找 FactoryBean。
    pub fn register_factory_bean_type(&self, type_id: TypeId, bean_name: String) {
        let mut types = self.factory_bean_types.lock().unwrap();
        types.insert(type_id, bean_name.clone());
        let mut names = self.factory_bean_names.lock().unwrap();
        if !names.iter().any(|n| n == &bean_name) {
            names.push(bean_name);
        }
    }

    /// 获取 FactoryBean 类型对应的 Bean 名称。
    pub fn get_factory_bean_type(&self, type_id: TypeId) -> Option<String> {
        self.factory_bean_types
            .lock()
            .unwrap()
            .get(&type_id)
            .cloned()
    }

    /// 检查指定 TypeId 是否是 FactoryBean。
    pub fn is_factory_bean_by_type(&self, type_id: TypeId) -> bool {
        self.factory_bean_types
            .lock()
            .unwrap()
            .contains_key(&type_id)
    }

    /// 检查指定名称是否是 FactoryBean。
    ///
    /// 对应 Spring 的 `BeanFactory.isFactoryBean(String)`。
    pub fn is_factory_bean(&self, bean_name: &str) -> bool {
        self.factory_bean_names
            .lock()
            .unwrap()
            .iter()
            .any(|n| n == bean_name)
    }

    /// 获取所有已注册的 FactoryBean 名称。
    pub fn factory_bean_names(&self) -> Vec<String> {
        self.factory_bean_names.lock().unwrap().clone()
    }

    /// 清除产品缓存。
    pub fn clear_cache(&self) {
        self.factory_bean_object_cache.lock().unwrap().clear();
    }

    /// 清除所有数据（缓存和类型映射）。
    pub fn clear_all(&self) {
        self.factory_bean_object_cache.lock().unwrap().clear();
        self.factory_bean_types.lock().unwrap().clear();
        self.factory_bean_names.lock().unwrap().clear();
    }

    /// 从缓存中移除指定 FactoryBean 的产品。
    pub fn remove_cached_object(&self, bean_name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.factory_bean_object_cache
            .lock()
            .unwrap()
            .remove(bean_name)
    }
}

impl Default for FactoryBeanRegistrySupport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_factory_bean_object() {
        let support = FactoryBeanRegistrySupport::new();

        let result = support
            .get_object_from_factory_bean_with_closure("myFactory", &|| {
                Ok(Arc::new(42_i32) as Arc<dyn Any + Send + Sync>)
            })
            .unwrap();

        assert_eq!(*result.downcast_ref::<i32>().unwrap(), 42);

        // 第二次应返回缓存
        let result2 = support
            .get_object_from_factory_bean_with_closure("myFactory", &|| {
                Ok(Arc::new(99_i32) as Arc<dyn Any + Send + Sync>)
            })
            .unwrap();

        assert_eq!(*result2.downcast_ref::<i32>().unwrap(), 42);
    }

    #[test]
    fn factory_bean_type_registration() {
        let support = FactoryBeanRegistrySupport::new();

        support.register_factory_bean_type(TypeId::of::<i32>(), "myIntFactory".to_string());

        assert!(support.is_factory_bean_by_type(TypeId::of::<i32>()));
        assert!(support.is_factory_bean("myIntFactory"));
        assert!(!support.is_factory_bean("unknown"));

        assert_eq!(
            support.get_factory_bean_type(TypeId::of::<i32>()),
            Some("myIntFactory".to_string())
        );
    }

    #[test]
    fn clear_cache_works() {
        let support = FactoryBeanRegistrySupport::new();

        support
            .get_object_from_factory_bean_with_closure("f1", &|| {
                Ok(Arc::new("v1".to_string()) as Arc<dyn Any + Send + Sync>)
            })
            .unwrap();

        support.clear_cache();

        // 清除后应重新创建
        let result = support
            .get_object_from_factory_bean_with_closure("f1", &|| {
                Ok(Arc::new("v2".to_string()) as Arc<dyn Any + Send + Sync>)
            })
            .unwrap();

        assert_eq!(*result.downcast_ref::<String>().unwrap(), "v2");
    }
}
