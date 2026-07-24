//! 注册表诊断摘要对象。

use serde::Serialize;

/// 已冻结组件注册表的计数摘要。
///
/// 所有计数都在生成快照时从不可变定义和绑定计算，不读取组件实例，也不触发
/// 工厂执行。`declared_dependency_count` 表示定义声明的选择器数量；一个
/// `all<dyn Trait>` 选择器即使映射到多个目标，也只计为一项声明。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct RegistrySummary {
    #[serde(rename = "definition_count")]
    definitions: usize,
    #[serde(rename = "singleton_count")]
    singletons: usize,
    #[serde(rename = "transient_count")]
    transients: usize,
    #[serde(rename = "declared_dependency_count")]
    declared_dependencies: usize,
    #[serde(rename = "trait_binding_count")]
    trait_bindings: usize,
}

impl RegistrySummary {
    /// 创建一份完整注册表计数摘要。
    pub(crate) const fn new(
        definition_count: usize,
        singleton_count: usize,
        transient_count: usize,
        declared_dependency_count: usize,
        trait_binding_count: usize,
    ) -> Self {
        Self {
            definitions: definition_count,
            singletons: singleton_count,
            transients: transient_count,
            declared_dependencies: declared_dependency_count,
            trait_bindings: trait_binding_count,
        }
    }

    /// 返回组件定义数量。
    #[must_use]
    pub const fn definition_count(&self) -> usize {
        self.definitions
    }

    /// 返回 Singleton 定义数量。
    #[must_use]
    pub const fn singleton_count(&self) -> usize {
        self.singletons
    }

    /// 返回 Transient 定义数量。
    #[must_use]
    pub const fn transient_count(&self) -> usize {
        self.transients
    }

    /// 返回组件定义显式声明的依赖选择器总数。
    #[must_use]
    pub const fn declared_dependency_count(&self) -> usize {
        self.declared_dependencies
    }

    /// 返回 Trait Binding 数量。
    #[must_use]
    pub const fn trait_binding_count(&self) -> usize {
        self.trait_bindings
    }
}
