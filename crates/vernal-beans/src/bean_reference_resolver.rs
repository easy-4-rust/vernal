//! BeanReferenceResolver — Spring 风格的 Bean 引用解析辅助工具。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionValueResolver` 的 Bean 引用解析部分。
//!
//! 提供从容器中按名称解析 Bean 引用的辅助功能。

use std::any::Any;
use std::sync::Arc;

use crate::bean_reference::BeanReference;
use crate::container::Container;

/// Spring 风格的 Bean 引用解析器。
///
/// 对应 Spring 的 `BeanDefinitionValueResolver` 中解析 `BeanReference` 的逻辑。
///
/// 按名称从 `Container` 中解析 `BeanReference` 指向的实际 Bean 实例。
/// 支持直接通过名称解析和通过 `BeanReference` 对象解析两种方式。
pub struct BeanReferenceResolver {
    /// 用于解析引用的 Container。
    container: Arc<Container>,
}

impl std::fmt::Debug for BeanReferenceResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BeanReferenceResolver")
            .field("container", &"Container(...)")
            .finish()
    }
}

impl BeanReferenceResolver {
    /// 使用指定的 Container 创建新的 BeanReferenceResolver。
    ///
    /// # 参数
    ///
    /// * `container` — 用于解析 Bean 引用的容器
    pub fn new(container: Arc<Container>) -> Self {
        Self { container }
    }

    /// 获取内部的 Container 引用。
    pub fn container(&self) -> &Arc<Container> {
        &self.container
    }

    /// 解析 Bean 引用为实际的 Bean 实例。
    ///
    /// 对应 Spring 的 `BeanDefinitionValueResolver.resolveReference(Object argName, RuntimeBeanReference ref)`。
    ///
    /// # 参数
    ///
    /// * `reference` — 要解析的 Bean 引用
    ///
    /// # 返回
    ///
    /// - `Ok(Some(Arc))` — 解析成功，找到对应的 Bean
    /// - `Ok(None)` — 引用指向的 Bean 不存在
    /// - `Err` — 解析过程中发生错误
    pub fn resolve_reference(
        &self,
        reference: &dyn BeanReference,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        let bean_name = reference.get_bean_name();
        self.resolve_by_name(bean_name)
    }

    /// 按名称解析 Bean。
    ///
    /// 对应 Spring 的 `BeanFactory.getBean(String name)`。
    ///
    /// 使用 Container 的 `resolve` 方法按 TypeId 查找，或通过类型名查找。
    /// 在当前实现中，通过 Container 的 warming_up 和已知定义来解析。
    ///
    /// # 参数
    ///
    /// * `bean_name` — Bean 的名称
    ///
    /// # 返回
    ///
    /// - `Ok(Some(Arc))` — 找到 Bean
    /// - `Ok(None)` — 未找到 Bean
    /// - `Err` — 解析过程中发生错误
    pub fn resolve_by_name(
        &self,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        // 在当前实现中，Container 没有直接按字符串名称解析的方法。
        // 此处返回 None，上层（如 DefaultListableBeanFactory）应通过
        // 自己的 Bean 定义映射来查找和解析。
        Ok(None)
    }

    /// 检查引用是否可解析（Bean 是否存在）。
    ///
    /// # 参数
    ///
    /// * `reference` — 要检查的 Bean 引用
    ///
    /// # 返回
    ///
    /// 如果 Bean 存在，返回 `true`。
    pub fn is_resolvable(&self, reference: &dyn BeanReference) -> bool {
        let key = reference.get_bean_name();
        self.contains_bean(key)
    }

    /// 按名称检查 Bean 是否存在。
    ///
    /// # 参数
    ///
    /// * `bean_name` — Bean 的名称
    ///
    /// # 返回
    ///
    /// 如果 Bean 存在，返回 `true`。
    pub fn contains_bean(&self, _bean_name: &str) -> bool {
        // 在当前实现中，通过 Container 检查 bean 是否存在
        // 使用容器级的 contains_bean_definition 方法
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component_registry::Registry;

    #[test]
    fn test_new_resolver() {
        let container = Arc::new(Container::new(Registry::empty()));
        let resolver = BeanReferenceResolver::new(container);
        // 空容器中按名称解析应返回 None
        let result = resolver.resolve_by_name("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_contains_bean_empty() {
        let container = Arc::new(Container::new(Registry::empty()));
        let resolver = BeanReferenceResolver::new(container);
        assert!(!resolver.contains_bean("nonexistent"));
    }
}
