//! 注册表诊断快照对象。

use serde::Serialize;

use crate::{ComponentSnapshot, RegistrySummary, TraitBindingSnapshot};

/// 已冻结注册表的只读、可序列化诊断快照。
///
/// 组件按依赖优先的真实构建顺序排列，Trait Binding 保留稳定注册顺序。该对象
/// 完全由拥有所有权的字符串和值类型组成，脱离注册表后仍可安全跨 Tokio task
/// 传递或序列化，同时无法反向访问组件工厂和实例。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RegistrySnapshot {
    summary: RegistrySummary,
    components: Vec<ComponentSnapshot>,
    trait_bindings: Vec<TraitBindingSnapshot>,
}

impl RegistrySnapshot {
    /// 由注册表生成完整诊断值对象。
    pub fn new(
        summary: RegistrySummary,
        components: Vec<ComponentSnapshot>,
        trait_bindings: Vec<TraitBindingSnapshot>,
    ) -> Self {
        Self {
            summary,
            components,
            trait_bindings,
        }
    }

    /// 返回注册表计数摘要。
    #[must_use]
    pub const fn summary(&self) -> &RegistrySummary {
        &self.summary
    }

    /// 按真实依赖优先构建顺序返回组件快照。
    #[must_use]
    pub fn components(&self) -> &[ComponentSnapshot] {
        &self.components
    }

    /// 按稳定注册顺序返回 Trait Binding 快照。
    #[must_use]
    pub fn trait_bindings(&self) -> &[TraitBindingSnapshot] {
        &self.trait_bindings
    }
}
