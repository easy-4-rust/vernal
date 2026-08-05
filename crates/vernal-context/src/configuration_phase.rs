//! 配置阶段枚举，对标 Spring 7.0 `ConfigurationCondition.ConfigurationPhase`。

use serde::Serialize;

/// 条件模块的评估阶段，对标 Spring 7.0
/// `org.springframework.context.annotation.ConfigurationCondition.ConfigurationPhase`。
///
/// Spring 的 `ConfigurationCondition#getConfigurationPhase()` 返回此枚举，让条件
/// 在 `@Configuration` 类解析阶段或注册 bean 阶段分别评估。vernal 的
/// [`crate::ConditionalComponentModule`] 已包含 `phase` 字段来承载这个语义，
/// 但本枚举让调用方在不持有模块实例的情况下也能按阶段比较。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigurationPhase {
    /// 解析 `@Configuration` 类时评估条件（PARSE_CONFIGURATION）。
    ///
    /// 对标 Spring `PARSE_CONFIGURATION`：条件不命中时整个 `@Configuration`
    /// 类不会被加入依赖图。vernal 中 [`crate::ConditionalComponentModule`] 在
    /// `VernalApplicationBuilder::register_conditional` 阶段评估条件，等同于
    /// Spring 的 PARSE_CONFIGURATION 时机。
    ParseConfiguration,
    /// 注册普通（非 `@Configuration`）bean 时评估条件（REGISTER_BEAN）。
    ///
    /// 对标 Spring `REGISTER_BEAN`：条件不会阻止 `@Configuration` 类加入依赖图。
    /// 在 vernal 当前架构中，`ConditionalComponentModule` 同样在 builder 阶段评估；
    /// 调用方可在构造时把 `Phase` 设置为 `RegisterBean` 来模拟"先加入 `@Configuration`
    /// 再评估条件"的 Spring 流程。
    RegisterBean,
}

impl ConfigurationPhase {
    /// 返回适合稳定序列化和监控标签的阶段名称。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ParseConfiguration => "parse_configuration",
            Self::RegisterBean => "register_bean",
        }
    }
}
