//! 不可变组件注册表对象。

use std::sync::Arc;

use crate::{
    BuildPlan, ComponentDefinition, ComponentSnapshot, Container, RegistrySnapshot,
    RegistrySummary, TraitBinding, TraitBindingSnapshot,
};

/// 保存已校验组件定义与构建计划的不可变注册表。
///
/// 注册表可以被多个容器共享，但实例缓存不在这里。每次调用 [`Registry::container`]
/// 都会获得相互隔离的作用域状态，适合多应用实例和并行测试。
#[derive(Clone, Debug)]
pub struct Registry {
    definitions: Arc<[Arc<ComponentDefinition>]>,
    bindings: Arc<[Arc<TraitBinding>]>,
    ordered_indices: Arc<[usize]>,
    plan: BuildPlan,
}

impl Registry {
    /// 由 [`crate::RegistryBuilder`] 在完成图校验后创建。
    pub(crate) fn new(
        definitions: Vec<Arc<ComponentDefinition>>,
        bindings: Vec<Arc<TraitBinding>>,
        ordered_indices: Vec<usize>,
        plan: BuildPlan,
    ) -> Self {
        Self {
            definitions: definitions.into(),
            bindings: bindings.into(),
            ordered_indices: ordered_indices.into(),
            plan,
        }
    }

    /// 按稳定注册顺序返回全部定义。
    #[must_use]
    pub fn definitions(&self) -> &[Arc<ComponentDefinition>] {
        &self.definitions
    }

    /// 按稳定注册顺序返回全部 Trait Binding。
    #[must_use]
    pub fn bindings(&self) -> &[Arc<TraitBinding>] {
        &self.bindings
    }

    /// 返回经过校验的依赖优先构建计划。
    #[must_use]
    pub fn plan(&self) -> &BuildPlan {
        &self.plan
    }

    /// 生成不包含工厂和实例的只读诊断快照。
    ///
    /// 组件使用已经校验的 `ordered_indices` 排列，不重复运行拓扑算法；Trait
    /// Binding 只复制选择键、目标和 Primary 标记，不暴露 upcast 闭包。
    #[must_use]
    pub fn snapshot(&self) -> RegistrySnapshot {
        let singleton_count = self
            .definitions
            .iter()
            .filter(|definition| definition.scope().is_singleton())
            .count();
        let transient_count = self
            .definitions
            .iter()
            .filter(|definition| definition.scope().is_transient())
            .count();
        let custom_scope_count = self.definitions.len() - singleton_count - transient_count;
        let declared_dependency_count = self
            .definitions
            .iter()
            .map(|definition| definition.dependencies().len())
            .sum();

        // `ordered_indices` 是 GraphPlanner 已验证的唯一权威构建顺序。诊断层只做
        // 值对象投影，不重新选择 Trait 实现或重新计算依赖图。
        let components = self
            .ordered_indices
            .iter()
            .enumerate()
            .map(|(build_order, index)| {
                let definition = &self.definitions[*index];
                ComponentSnapshot::new(
                    definition.key().to_string(),
                    definition.key().type_name().to_owned(),
                    definition
                        .key()
                        .qualifier()
                        .map(|qualifier| qualifier.as_str().to_owned()),
                    definition.scope().as_str().to_owned(),
                    build_order,
                    definition
                        .dependencies()
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                )
            })
            .collect();
        let trait_bindings = self
            .bindings
            .iter()
            .map(|binding| {
                TraitBindingSnapshot::new(
                    binding.key().to_string(),
                    binding.key().type_name().to_owned(),
                    binding
                        .key()
                        .qualifier()
                        .map(|qualifier| qualifier.as_str().to_owned()),
                    binding.target().to_string(),
                    binding.is_primary(),
                )
            })
            .collect();
        let summary = RegistrySummary::new(
            self.definitions.len(),
            singleton_count,
            transient_count,
            custom_scope_count,
            declared_dependency_count,
            self.bindings.len(),
        );

        RegistrySnapshot::new(summary, components, trait_bindings)
    }

    /// 创建拥有独立实例缓存的容器。
    #[must_use]
    pub fn container(&self) -> Container {
        Container::new(self.clone())
    }

    /// 返回定义数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// 返回注册表是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// 返回依赖优先的定义索引，供容器预热使用。
    pub(crate) fn ordered_indices(&self) -> &[usize] {
        &self.ordered_indices
    }
}
