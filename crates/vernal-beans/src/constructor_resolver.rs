//! ConstructorResolver — Spring 风格的构造器解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ConstructorResolver`。
//!
//! 负责解析 Bean 定义的构造参数，使用 SimpleInstantiationStrategy
//! 来创建 Bean 实例。

use std::any::Any;
use std::sync::Arc;

use crate::bean_definition::BeanDefinition;
use crate::instantiation_strategy::InstantiationStrategy;
use crate::simple_instantiation_strategy::SimpleInstantiationStrategy;

/// Spring 风格的构造器解析器。
///
/// 对应 Spring 的 `ConstructorResolver`。
///
/// 解析 Bean 定义的构造参数，并委托给 `SimpleInstantiationStrategy`
/// 来创建 Bean 实例。支持从构造参数值中提取参数列表。
#[derive(Debug)]
pub struct ConstructorResolver {
    /// 实例化策略。
    strategy: SimpleInstantiationStrategy,
}

impl ConstructorResolver {
    /// 使用指定的 SimpleInstantiationStrategy 创建新的 ConstructorResolver。
    ///
    /// # 参数
    ///
    /// * `strategy` — 用于实例化 Bean 的策略
    pub fn new(strategy: SimpleInstantiationStrategy) -> Self {
        Self { strategy }
    }

    /// 使用默认的 SimpleInstantiationStrategy 创建新的 ConstructorResolver。
    pub fn default() -> Self {
        Self {
            strategy: SimpleInstantiationStrategy::new(),
        }
    }

    /// 获取内部的 SimpleInstantiationStrategy 引用。
    pub fn strategy(&self) -> &SimpleInstantiationStrategy {
        &self.strategy
    }

    /// 解析指定 Bean 定义的构造参数。
    ///
    /// 对应 Spring 的 `ConstructorResolver.resolveConstructorArguments(RootBeanDefinition mbd, Object... args)`。
    ///
    /// 从 Bean 定义的 `ConstructorArgumentValues` 中提取参数值，
    /// 返回构造器参数的列表。如果 Bean 定义没有构造参数，返回空 Vec。
    ///
    /// # 参数
    ///
    /// * `definition` — Bean 定义，包含构造参数信息
    ///
    /// # 返回
    ///
    /// 构造参数值的列表。如果没有构造参数，返回空 Vec。
    pub fn resolve_constructor_arguments(
        &self,
        definition: &dyn BeanDefinition,
    ) -> Vec<Arc<dyn Any + Send + Sync>> {
        let mut args: Vec<Arc<dyn Any + Send + Sync>> = Vec::new();

        // 尝试通过 AbstractBeanDefinition 获取构造参数值
        let any_ref: &dyn Any = definition as &dyn Any;
        if let Some(abs_def) =
            any_ref.downcast_ref::<crate::abstract_bean_definition::AbstractBeanDefinition>()
        {
            let cv = abs_def.constructor_argument_values();
            for (_index, vh) in cv.indexed_argument_values() {
                if let Some(val) = vh.value() {
                    args.push(val.clone());
                }
            }
            for vh in cv.generic_argument_values() {
                if let Some(val) = vh.value() {
                    args.push(val.clone());
                }
            }
        }

        args
    }

    /// 使用已解析的构造参数实例化 Bean。
    ///
    /// # 参数
    ///
    /// * `definition` — Bean 定义
    /// * `bean_name` — Bean 的名称
    /// * `args` — 构造参数
    ///
    /// # 返回
    ///
    /// - `Ok(Arc)` — 实例化成功的 Bean
    /// - `Err` — 实例化失败
    pub fn instantiate_using_arguments(
        &self,
        definition: &dyn BeanDefinition,
        bean_name: &str,
        args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        self.strategy.instantiate(
            definition,
            bean_name,
            definition.factory_bean_name(),
            definition.factory_method_name(),
            args,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::root_bean_definition::RootBeanDefinition;

    #[test]
    fn test_new_resolver() {
        let resolver = ConstructorResolver::default();
        assert_eq!(resolver.strategy().constructor_count(), 0);
    }

    #[test]
    fn test_resolve_constructor_arguments_empty() {
        let resolver = ConstructorResolver::default();
        let def = RootBeanDefinition::new();
        let args = resolver.resolve_constructor_arguments(&def);
        assert!(args.is_empty());
    }
}
