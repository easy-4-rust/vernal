//! ListableBeanFactory — Spring 风格的可列举 BeanFactory 接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.ListableBeanFactory`。
//!
//! 扩展 `BeanFactory`，提供列举容器内所有 Bean 的能力。

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::bean_factory::BeanFactory;

/// Spring 风格的可列举 BeanFactory 接口。
///
/// 对应 Spring 的 `ListableBeanFactory`。
///
/// 提供枚举容器内所有 Bean 的能力。
pub trait ListableBeanFactory: BeanFactory {
    /// 检查容器是否包含指定 Bean 定义。
    fn contains_bean_definition(&self, bean_name: &str) -> bool;

    /// 获取 Bean 定义总数。
    fn bean_definition_count(&self) -> usize;

    /// 获取所有 Bean 定义名称。
    fn bean_definition_names(&self) -> Vec<String>;

    /// 按类型获取所有 Bean 名称（使用 TypeId）。
    fn bean_names_for_type_id(
        &self,
        type_id: std::any::TypeId,
        include_non_singletons: bool,
        allow_eager_init: bool,
    ) -> Vec<String>;

    /// 按类型获取所有 Bean 实例（使用 TypeId）。
    fn beans_of_type_id(
        &self,
        type_id: std::any::TypeId,
        include_non_singletons: bool,
        allow_eager_init: bool,
    ) -> Result<HashMap<String, Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>;

    /// 返回已注册的 BeanPostProcessor 数量。
    fn bean_post_processor_count(&self) -> usize;

    /// 返回是否包含非 singleton Bean。
    fn contains_non_singleton_bean(&self) -> bool;

    /// 返回是否包含 singleton Bean。
    fn contains_singleton_bean(&self) -> bool;

    /// 返回 Bean 名称的迭代器。
    fn bean_names_iterator(&self) -> Box<dyn Iterator<Item = String> + '_>;
}
