//! AbstractBeanFactory — Spring 风格的抽象 BeanFactory 基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AbstractBeanFactory`。
//!
//! 提供 BeanFactory 实现的公共功能，包括 BeanPostProcessor 管理和
//! 忽略依赖类型管理。作为具体 BeanFactory 实现的基类。

use std::any::TypeId;
use std::collections::HashSet;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::bean_post_processor::BeanPostProcessor;

/// Spring 风格的抽象 BeanFactory 基类。
///
/// 对应 Spring 的 `AbstractBeanFactory`。
///
/// 管理 BeanPostProcessor 链和忽略的依赖类型。充当自定义容器实现的基类，
/// 提供需要跨多种 BeanFactory 实现复用的共享逻辑。
pub struct AbstractBeanFactory {
    /// 被忽略的依赖类型集合（这些类型不参与自动装配）。
    ignore_dependency_types: Arc<Mutex<HashSet<TypeId>>>,
    /// 已注册的 BeanPostProcessor 列表。
    bean_post_processors: Arc<Mutex<Vec<Arc<dyn BeanPostProcessor>>>>,
}

impl fmt::Debug for AbstractBeanFactory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ignored_count = self
            .ignore_dependency_types
            .lock()
            .ok()
            .map_or(0, |s| s.len());
        let processor_count = self.bean_post_processors.lock().ok().map_or(0, |v| v.len());
        f.debug_struct("AbstractBeanFactory")
            .field("ignored_dependency_count", &ignored_count)
            .field("bean_post_processor_count", &processor_count)
            .finish()
    }
}

impl AbstractBeanFactory {
    /// 创建一个新的 AbstractBeanFactory。
    pub fn new() -> Self {
        Self {
            ignore_dependency_types: Arc::new(Mutex::new(HashSet::new())),
            bean_post_processors: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 注册一个 BeanPostProcessor。
    ///
    /// 对应 Spring 的 `AbstractBeanFactory.addBeanPostProcessor(BeanPostProcessor beanPostProcessor)`。
    ///
    /// # 参数
    ///
    /// * `processor` — 要注册的后处理器
    pub fn add_bean_post_processor(&self, processor: Arc<dyn BeanPostProcessor>) {
        if let Ok(mut processors) = self.bean_post_processors.lock() {
            processors.push(processor);
        }
    }

    /// 获取所有已注册的 BeanPostProcessor。
    ///
    /// 对应 Spring 的 `AbstractBeanFactory.getBeanPostProcessors()`。
    ///
    /// # 返回
    ///
    /// 已注册的后处理器列表的副本。
    pub fn get_bean_post_processors(&self) -> Vec<Arc<dyn BeanPostProcessor>> {
        self.bean_post_processors
            .lock()
            .map(|processors| processors.clone())
            .unwrap_or_default()
    }

    /// 获取已注册的 BeanPostProcessor 数量。
    ///
    /// 对应 Spring 的 `AbstractBeanFactory.getBeanPostProcessorCount()`。
    ///
    /// # 返回
    ///
    /// 后处理器数量。
    pub fn bean_post_processor_count(&self) -> usize {
        self.bean_post_processors
            .lock()
            .map(|processors| processors.len())
            .unwrap_or(0)
    }

    /// 将指定类型加入忽略依赖类型集合。
    ///
    /// 被标记为忽略的类型在自动装配时将不会被注入。
    ///
    /// 对应 Spring 的 `AbstractBeanFactory.ignoreDependencyType(Class<?> type)`。
    ///
    /// # 参数
    ///
    /// * `type_id` — 要忽略的类型 ID
    pub fn ignore_dependency_type(&self, type_id: TypeId) {
        if let Ok(mut types) = self.ignore_dependency_types.lock() {
            types.insert(type_id);
        }
    }

    /// 获取所有被忽略的依赖类型。
    ///
    /// # 返回
    ///
    /// 被忽略的类型 ID 集合的副本。
    pub fn get_ignored_dependency_types(&self) -> HashSet<TypeId> {
        self.ignore_dependency_types
            .lock()
            .map(|types| types.clone())
            .unwrap_or_default()
    }

    /// 检查指定类型是否被忽略。
    ///
    /// # 参数
    ///
    /// * `type_id` — 要检查的类型 ID
    ///
    /// # 返回
    ///
    /// 如果该类型被忽略，返回 `true`。
    pub fn is_ignored_dependency_type(&self, type_id: &TypeId) -> bool {
        self.ignore_dependency_types
            .lock()
            .map(|types| types.contains(type_id))
            .unwrap_or(false)
    }

    /// 重置所有状态。
    pub fn clear(&self) {
        if let Ok(mut types) = self.ignore_dependency_types.lock() {
            types.clear();
        }
        if let Ok(mut processors) = self.bean_post_processors.lock() {
            processors.clear();
        }
    }
}

impl Default for AbstractBeanFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProcessor;
    impl BeanPostProcessor for TestProcessor {}

    #[test]
    fn test_new_abstract_bean_factory() {
        let factory = AbstractBeanFactory::new();
        assert_eq!(factory.bean_post_processor_count(), 0);
        assert!(factory.get_ignored_dependency_types().is_empty());
    }

    #[test]
    fn test_ignore_dependency_type() {
        let factory = AbstractBeanFactory::new();
        let type_id = TypeId::of::<String>();
        factory.ignore_dependency_type(type_id);
        assert!(factory.is_ignored_dependency_type(&type_id));
    }

    #[test]
    fn test_add_bean_post_processor() {
        let factory = AbstractBeanFactory::new();
        factory.add_bean_post_processor(Arc::new(TestProcessor));
        assert_eq!(factory.bean_post_processor_count(), 1);
        assert_eq!(factory.get_bean_post_processors().len(), 1);
    }

    #[test]
    fn test_clear() {
        let factory = AbstractBeanFactory::new();
        factory.ignore_dependency_type(TypeId::of::<String>());
        factory.add_bean_post_processor(Arc::new(TestProcessor));
        factory.clear();
        assert!(factory.get_ignored_dependency_types().is_empty());
        assert_eq!(factory.bean_post_processor_count(), 0);
    }
}
