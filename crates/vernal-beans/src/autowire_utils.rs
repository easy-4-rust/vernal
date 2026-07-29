//! AutowireUtils — Spring 风格的自动装配工具函数。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AutowireUtils`。
//!
//! 提供自动装配相关的静态工具方法。

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use crate::autowire::Autowire;
use crate::bean_definition::BeanDefinition;
use crate::bean_factory::BeanFactory;
use crate::component_key::ComponentKey;

/// Spring 风格的自动装配工具函数集。
///
/// 对应 Spring 的 `AutowireUtils`。
///
/// 提供静态方法用于判断自动装配模式匹配、确定候选 Bean 等。
pub struct AutowireUtils;

impl AutowireUtils {
    /// 判断给定的自动装配模式是否匹配目标 Bean 定义。
    ///
    /// 对应 Spring 的 `isAutowireModeMatch` 概念。
    ///
    /// # 参数
    ///
    /// * `autowire_mode` — 自动装配模式
    /// * `definition` — Bean 定义
    ///
    /// # 返回
    ///
    /// - `true` — 模式匹配，可以对该 Bean 应用指定的自动装配模式
    /// - `false` — 模式不匹配
    pub fn is_autowire_mode_match(
        autowire_mode: Autowire,
        definition: &dyn BeanDefinition,
    ) -> bool {
        match autowire_mode {
            Autowire::No => false,
            Autowire::ByName | Autowire::ByType | Autowire::Constructor => {
                // 只要 Bean 定义允许 autowire 并且不是 abstract 的，就认为匹配
                definition.is_autowire_candidate() && !definition.is_abstract()
            }
        }
    }

    /// 确定自动装配候选 Bean。
    ///
    /// 对应 Spring 的 `determineAutowireCandidates` 概念。
    ///
    /// 从工厂中所有可用的 Bean 中，筛选出符合指定类型且是 autowire candidate 的 Bean。
    ///
    /// # 参数
    ///
    /// * `factory` — BeanFactory 引用
    /// * `required_type_id` — 所需类型的 TypeId
    /// * `bean_names` — 所有 Bean 的名称列表
    /// * `is_autowire_candidate_fn` — 用于判断 Bean 是否是 autowire candidate 的函数
    /// * `bean_key_fn` — 用于从名称创建 ComponentKey 的函数
    ///
    /// # 返回
    ///
    /// 候选 Bean 的映射（名称 -> 实例）
    pub fn determine_autowire_candidates(
        factory: &dyn BeanFactory,
        required_type_id: TypeId,
        bean_names: &[String],
        is_autowire_candidate_fn: &dyn Fn(&str) -> bool,
        bean_key_fn: &dyn Fn(&str) -> ComponentKey,
    ) -> Result<
        HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let mut candidates = HashMap::new();

        for bean_name in bean_names {
            // 检查 Bean 是否是 autowire candidate
            if !is_autowire_candidate_fn(bean_name) {
                continue;
            }

            let bean_key = bean_key_fn(bean_name);
            if factory.is_type_match(&bean_key, required_type_id) {
                // 尝试获取 Bean 实例
                match factory.get_bean_by_key(&bean_key) {
                    Ok(instance) => {
                        candidates.insert(bean_name.clone(), instance);
                    }
                    Err(_) => {
                        // 跳过获取失败的 Bean
                        continue;
                    }
                }
            }
        }

        Ok(candidates)
    }

    /// 判断给定方法是否适合作为工厂方法。
    ///
    /// 对应 Spring 的 `isFactoryMethod` 概念。
    ///
    /// # 参数
    ///
    /// * `method_name` — 方法名
    ///
    /// # 返回
    ///
    /// - `true` — 如果方法名看起来像工厂方法
    pub fn is_factory_method(method_name: &str) -> bool {
        // 简单启发式：非静态方法或非 getter/setter
        !method_name.starts_with("get")
            && !method_name.starts_with("set")
            && !method_name.starts_with("is")
            && !method_name.starts_with("has")
    }

    /// 判断自动装配是否可用（即定义不是 abstract 且是 autowire candidate）。
    ///
    /// # 参数
    ///
    /// * `definition` — Bean 定义
    ///
    /// # 返回
    ///
    /// - `true` — 可以对该 Bean 应用自动装配
    pub fn is_autowire_applicable(definition: &dyn BeanDefinition) -> bool {
        definition.is_autowire_candidate() && !definition.is_abstract()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_autowire_mode_match_no() {
        let bd = crate::abstract_bean_definition::AbstractBeanDefinition::new();
        assert!(!AutowireUtils::is_autowire_mode_match(Autowire::No, &bd));
    }

    #[test]
    fn test_is_autowire_mode_match_by_name() {
        let bd = crate::abstract_bean_definition::AbstractBeanDefinition::new();
        assert!(AutowireUtils::is_autowire_mode_match(Autowire::ByName, &bd));
    }

    #[test]
    fn test_is_autowire_applicable() {
        let bd = crate::abstract_bean_definition::AbstractBeanDefinition::new();
        assert!(AutowireUtils::is_autowire_applicable(&bd));
    }

    #[test]
    fn test_is_factory_method() {
        assert!(!AutowireUtils::is_factory_method("getBean"));
        assert!(!AutowireUtils::is_factory_method("setName"));
        assert!(AutowireUtils::is_factory_method("createInstance"));
        assert!(AutowireUtils::is_factory_method("buildFactory"));
    }
}
