//! CustomAutowireConfigurer — Spring 风格的自定义装配校验器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.CustomAutowireConfigurer`。
//!
//! 允许注册自定义的自动装配候选规则（如基于限定符注解的过滤）。
//! 配合 `AutowireCandidateResolver` 使用，决定某个 BeanDefinition 是否符合自定义装配条件。

use crate::bean_definition::BeanDefinition;

/// 自定义装配配置器 trait。
///
/// 对应 Spring 的 `CustomAutowireConfigurer`。
///
/// 实现者通过 `is_custom_autowire_candidate()` 返回额外的装配候选判定。
/// 默认实现返回 `false`，表示不做额外判定（即不强制将任何 Bean 视为"自定义装配候选"）。
pub trait CustomAutowireConfigurer: Send + Sync {
    /// 判断给定 BeanDefinition 是否是自定义装配候选。
    ///
    /// 对应 Spring 的 `CustomAutowireConfigurer.customCandidateTypes` 匹配逻辑。
    ///
    /// # 参数
    ///
    /// - `definition` — 待判定的 Bean 定义
    ///
    /// # 返回
    ///
    /// - `true` — 该 Bean 满足自定义装配规则，应被视为候选
    /// - `false` — 不满足
    fn is_custom_autowire_candidate(&self, _definition: &dyn BeanDefinition) -> bool {
        false
    }
}

/// 默认的空实现，所有判定都返回 `false`。
///
/// 可作为未配置自定义规则时的占位实例。
#[derive(Debug, Clone, Default)]
pub struct NoopCustomAutowireConfigurer;

impl CustomAutowireConfigurer for NoopCustomAutowireConfigurer {}
