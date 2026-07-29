//! BeanFactoryDependencyProvider — 从 BeanFactory 提供依赖。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.DefaultListableBeanFactory`
//! 中 `resolveDependency` 的周边概念，以及依赖解析时的 `DependencyProvider`。
//!
//! 将 `BeanFactory` 适配为统一的依赖提供接口，便于注入点解析。

use std::any::{Any, TypeId};
use std::sync::Arc;

use crate::bean_factory::BeanFactory;
use crate::dependency_descriptor::DependencyDescriptor;

/// 依赖提供器 trait。
///
/// 表示「能够按依赖描述符解析依赖」的抽象。注入点（构造器参数、字段等）
/// 通过此接口向容器请求依赖实例。
pub trait DependencyProvider: Send + Sync {
    /// 按依赖描述符解析依赖。
    ///
    /// 对应 Spring 的 `AutowireCapableBeanFactory.resolveDependency(...)`。
    ///
    /// # 参数
    ///
    /// * `descriptor` — 依赖描述符
    ///
    /// # 返回
    ///
    /// - `Ok(Some(instance))` — 解析到依赖
    /// - `Ok(None)` — 可选依赖未解析到
    /// - `Err` — 解析必需依赖失败
    fn resolve_dependency(
        &self,
        descriptor: &DependencyDescriptor,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 基于 `BeanFactory` 的依赖提供器。
///
/// 对应 Spring 的依赖解析适配层。
///
/// 将一个 `BeanFactory` 包装为 `DependencyProvider`，按 `DependencyDescriptor`
/// 中的类型信息从底层工厂解析依赖实例。
pub struct BeanFactoryDependencyProvider<'a> {
    bean_factory: &'a dyn BeanFactory,
}

impl<'a> std::fmt::Debug for BeanFactoryDependencyProvider<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BeanFactoryDependencyProvider")
            .finish_non_exhaustive()
    }
}

impl<'a> BeanFactoryDependencyProvider<'a> {
    /// 创建依赖提供器。
    pub fn new(bean_factory: &'a dyn BeanFactory) -> Self {
        Self { bean_factory }
    }

    /// 获取底层 BeanFactory 引用。
    pub fn bean_factory(&self) -> &dyn BeanFactory {
        self.bean_factory
    }

    /// 按类型直接解析（便捷方法）。
    pub fn resolve_by_type_id(
        &self,
        type_id: TypeId,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        self.bean_factory.get_bean_by_type_id(type_id)
    }
}

impl<'a> DependencyProvider for BeanFactoryDependencyProvider<'a> {
    fn resolve_dependency(
        &self,
        descriptor: &DependencyDescriptor,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        let type_id = descriptor.type_id();
        match self.bean_factory.get_bean_by_type_id(type_id) {
            Ok(instance) => Ok(Some(instance)),
            Err(err) if descriptor.is_optional() => Ok(None),
            Err(err) => Err(err),
        }
    }
}

/// 拥有所有权的依赖提供器（持有 `Arc<dyn BeanFactory>`）。
pub struct OwnedBeanFactoryDependencyProvider {
    bean_factory: Arc<dyn BeanFactory>,
}

impl std::fmt::Debug for OwnedBeanFactoryDependencyProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedBeanFactoryDependencyProvider")
            .finish_non_exhaustive()
    }
}

impl OwnedBeanFactoryDependencyProvider {
    /// 创建拥有所有权的依赖提供器。
    pub fn new(bean_factory: Arc<dyn BeanFactory>) -> Self {
        Self { bean_factory }
    }

    /// 获取内部 BeanFactory 引用。
    pub fn bean_factory(&self) -> &Arc<dyn BeanFactory> {
        &self.bean_factory
    }
}

impl DependencyProvider for OwnedBeanFactoryDependencyProvider {
    fn resolve_dependency(
        &self,
        descriptor: &DependencyDescriptor,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        let type_id = descriptor.type_id();
        match self.bean_factory.get_bean_by_type_id(type_id) {
            Ok(instance) => Ok(Some(instance)),
            Err(err) if descriptor.is_optional() => Ok(None),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component_key::ComponentKey;
    use crate::object_provider::ObjectProvider;

    struct FailingFactory;

    impl BeanFactory for FailingFactory {
        fn get_bean_by_key(
            &self,
            _key: &ComponentKey,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Err("not found".into())
        }
        fn get_bean_by_type_id(
            &self,
            _type_id: TypeId,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Err("not found".into())
        }
        fn contains_bean(&self, _key: &ComponentKey) -> bool {
            false
        }
        fn is_singleton(
            &self,
            _key: &ComponentKey,
        ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            Ok(true)
        }
        fn is_prototype(
            &self,
            _key: &ComponentKey,
        ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            Ok(false)
        }
        fn get_type(
            &self,
            _key: &ComponentKey,
        ) -> Result<Option<&'static str>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(None)
        }
        fn get_aliases(&self, _key: &ComponentKey) -> Vec<ComponentKey> {
            Vec::new()
        }
        fn get_bean_provider_by_type_id(
            &self,
            _type_id: TypeId,
        ) -> Result<
            Box<dyn ObjectProvider<dyn Any + Send + Sync> + '_>,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            Err("not supported".into())
        }
        fn is_type_match(&self, _key: &ComponentKey, _type_id: TypeId) -> bool {
            false
        }
    }

    #[test]
    fn test_optional_resolves_to_none() {
        let factory = FailingFactory;
        let provider = BeanFactoryDependencyProvider::new(&factory);
        let descriptor =
            DependencyDescriptor::for_field(TypeId::of::<u32>(), std::any::type_name::<u32>())
                .with_optional(true);
        let result = provider.resolve_dependency(&descriptor).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_required_propagates_error() {
        let factory = FailingFactory;
        let provider = BeanFactoryDependencyProvider::new(&factory);
        let descriptor =
            DependencyDescriptor::for_field(TypeId::of::<u32>(), std::any::type_name::<u32>());
        let result = provider.resolve_dependency(&descriptor);
        assert!(result.is_err());
    }
}
