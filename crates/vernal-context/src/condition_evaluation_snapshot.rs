//! 条件组件评估诊断快照对象。

use serde::Serialize;

/// 一次条件模块构建期判断的只读、可序列化、脱敏结果。
///
/// 快照只包含静态模块名、条件类型名、命中状态、组件类型标识和声明数量；不会
/// 暴露属性键、属性值、期望值或底层错误正文。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConditionEvaluationSnapshot {
    module: String,
    condition: String,
    matched: bool,
    components: Vec<String>,
    trait_binding_count: usize,
    lifecycle_count: usize,
    event_listener_count: usize,
}

impl ConditionEvaluationSnapshot {
    /// 创建已经完成脱敏的条件评估快照。
    pub(crate) fn new(
        module: &'static str,
        condition: &'static str,
        matched: bool,
        components: Vec<String>,
        trait_binding_count: usize,
        lifecycle_count: usize,
        event_listener_count: usize,
    ) -> Self {
        Self {
            module: module.to_owned(),
            condition: condition.to_owned(),
            matched,
            components,
            trait_binding_count,
            lifecycle_count,
            event_listener_count,
        }
    }

    /// 返回条件模块静态名称。
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// 返回稳定条件类型名。
    #[must_use]
    pub fn condition(&self) -> &str {
        &self.condition
    }

    /// 返回该模块是否进入最终依赖图。
    #[must_use]
    pub const fn matched(&self) -> bool {
        self.matched
    }

    /// 返回模块声明的组件类型标识。
    ///
    /// 未命中模块也保留类型标识，便于解释“为什么某组件没有进入容器”。
    #[must_use]
    pub fn components(&self) -> &[String] {
        &self.components
    }

    /// 返回模块声明的 Trait Binding 数量。
    #[must_use]
    pub const fn trait_binding_count(&self) -> usize {
        self.trait_binding_count
    }

    /// 返回模块声明的生命周期登记数量。
    #[must_use]
    pub const fn lifecycle_count(&self) -> usize {
        self.lifecycle_count
    }

    /// 返回模块声明的强类型事件监听器数量。
    #[must_use]
    pub const fn event_listener_count(&self) -> usize {
        self.event_listener_count
    }
}
