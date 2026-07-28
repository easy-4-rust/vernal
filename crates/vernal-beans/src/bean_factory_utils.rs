//! BeanFactoryUtils — Spring 风格的 BeanFactory 工具方法。
//!
//! 对应 Java 类：`org.springframework.beans.factory.BeanFactoryUtils`。
//!
//! 提供 BeanFactory 的通用工具方法，如按类型收集 Bean、统计数量等。

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::bean_factory::BeanFactory;
use crate::listable_bean_factory::ListableBeanFactory;

/// FactoryBean 前缀：`"&"`。
const FACTORY_BEAN_PREFIX: &str = "&";

/// Spring 风格的 BeanFactory 工具方法。
///
/// 对应 Spring 的 `BeanFactoryUtils`。
pub struct BeanFactoryUtils;

impl BeanFactoryUtils {
    /// 转换 Bean 名称（去除 FactoryBean 前缀）。
    ///
    /// 对应 Spring 的 `BeanFactoryUtils.transformedBeanName(String name)`。
    pub fn transformed_bean_name(name: &str) -> &str {
        if let Some(stripped) = name.strip_prefix(FACTORY_BEAN_PREFIX) {
            stripped
        } else {
            name
        }
    }

    /// 检查名称是否是 FactoryBean 名称。
    ///
    /// 对应 Spring 的 `BeanFactoryUtils.isFactoryBean(String name, BeanFactory beanFactory)`。
    pub fn is_factory_bean(name: &str) -> bool {
        name.starts_with(FACTORY_BEAN_PREFIX)
    }

    /// 按类型统计 Bean 数量。
    ///
    /// 对应 Spring 的 `BeanFactoryUtils.countBeansForType(Class<?> type, BeanFactory beanFactory)`。
    ///
    /// 遍历 `ListableBeanFactory.bean_names_for_type_id` 获取匹配的 Bean 名称数量。
    pub fn count_beans_for_type(type_id: TypeId, factory: &dyn ListableBeanFactory) -> usize {
        factory.bean_names_for_type_id(type_id, true, true).len()
    }

    /// 按类型查找 Bean 名称。
    ///
    /// 对应 Spring 的 `BeanFactoryUtils.beanNamesForTypeIncludingAncestors`。
    pub fn bean_names_for_type(type_id: TypeId, factory: &dyn ListableBeanFactory) -> Vec<String> {
        factory.bean_names_for_type_id(type_id, true, true)
    }

    /// 按类型获取 Bean 名称和实例的映射。
    ///
    /// 对应 Spring 的 `BeanFactoryUtils.beansOfTypeIncludingAncestors`。
    pub fn beans_of_type(
        type_id: TypeId,
        factory: &dyn ListableBeanFactory,
    ) -> Result<HashMap<String, Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        let names = Self::bean_names_for_type(type_id, factory);
        let mut result = HashMap::new();

        // 由于 ComponentKey 需要 TypeId 构造，这里使用 bean_names_for_type_id 获取的名称
        // 然后通过 get_bean_by_type_id 获取实例
        if let Ok(bean) = factory.get_bean_by_type_id(type_id) {
            // 获取所有匹配的名称，使用第一个名称作为 key
            if let Some(first_name) = names.first() {
                result.insert(first_name.clone(), bean);
            }
        }

        Ok(result)
    }

    /// 获取所有 Bean 定义名称。
    ///
    /// 对应 Spring 的 `BeanFactoryUtils.beanDefinitionNames(BeanFactory beanFactory)`。
    pub fn bean_definition_names(factory: &dyn ListableBeanFactory) -> Vec<String> {
        factory.bean_definition_names()
    }

    /// 检查是否是 FactoryBean（通过名称判断）。
    ///
    /// 对应 Spring 的 `BeanFactoryUtils.isFactoryBean(String name, BeanFactory beanFactory)`。
    pub fn check_is_factory_bean(name: &str) -> bool {
        name.starts_with(FACTORY_BEAN_PREFIX)
    }

    /// 按类型统计 Bean 数量（包含父容器）。
    pub fn count_beans_for_type_including_ancestors(
        type_id: TypeId,
        factory: &dyn ListableBeanFactory,
    ) -> usize {
        Self::count_beans_for_type(type_id, factory)
    }

    /// 按类型获取 Bean 名称（包含父容器）。
    pub fn bean_names_for_type_including_ancestors(
        type_id: TypeId,
        factory: &dyn ListableBeanFactory,
    ) -> Vec<String> {
        Self::bean_names_for_type(type_id, factory)
    }
}
