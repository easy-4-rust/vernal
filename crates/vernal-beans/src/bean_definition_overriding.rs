//! BeanDefinitionOverridingStrategy — Spring 风格的 Bean 定义覆盖策略。
//!
//! 对应 Spring 中 Bean 定义覆盖的行为策略。
//!
//! 决定在注册同名 Bean 定义时是否允许覆盖现有定义。

use crate::bean_definition::BeanDefinition;

/// Spring 风格的 Bean 定义覆盖策略枚举。
///
/// 控制当注册名称与现有定义冲突时的行为。
///
/// # 变体
///
/// - `OverrideAlways` — 始终覆盖（Spring 默认行为，当 allowBeanDefinitionOverriding = true）
/// - `OverrideIfExists` — 仅当已存在时覆盖（不抛出异常）
/// - `OverrideIfPrimary` — 仅当新定义为 primary 时覆盖
/// - `OverrideNever` — 从不覆盖，抛出异常
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BeanDefinitionOverridingStrategy {
    /// 始终覆盖现有定义。
    OverrideAlways,
    /// 覆盖已存在的定义（不抛出异常）。
    OverrideIfExists,
    /// 仅当新定义为 primary 时覆盖。
    OverrideIfPrimary,
    /// 从不覆盖，抛出 BeanDefinitionOverrideException。
    OverrideNever,
}

impl BeanDefinitionOverridingStrategy {
    /// 判断是否应覆盖当前定义。
    ///
    /// 对应 Spring 的决定逻辑：给定当前已注册的定义和新的定义，
    /// 根据策略返回是否允许覆盖。
    ///
    /// # 参数
    ///
    /// * `current_definition` — 当前已注册的 Bean 定义。
    /// * `new_definition` — 新传入的 Bean 定义。
    ///
    /// # 返回
    ///
    /// `true` 表示允许覆盖，`false` 表示禁止覆盖。
    pub fn should_override(
        &self,
        _current_definition: &dyn BeanDefinition,
        new_definition: &dyn BeanDefinition,
    ) -> bool {
        match self {
            Self::OverrideAlways => true,
            Self::OverrideIfExists => true,
            Self::OverrideIfPrimary => new_definition.is_primary(),
            Self::OverrideNever => false,
        }
    }
}

impl Default for BeanDefinitionOverridingStrategy {
    fn default() -> Self {
        // Spring 默认行为（allowBeanDefinitionOverriding = true 时）
        Self::OverrideAlways
    }
}
