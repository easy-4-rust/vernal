//! 不可变组件注册表对象。

use std::sync::Arc;

use crate::{BuildPlan, ComponentDefinition, Container};

/// 保存已校验组件定义与构建计划的不可变注册表。
///
/// 注册表可以被多个容器共享，但实例缓存不在这里。每次调用 [`Registry::container`]
/// 都会获得相互隔离的作用域状态，适合多应用实例和并行测试。
#[derive(Clone, Debug)]
pub struct Registry {
    definitions: Arc<[Arc<ComponentDefinition>]>,
    ordered_indices: Arc<[usize]>,
    plan: BuildPlan,
}

impl Registry {
    /// 由 [`crate::RegistryBuilder`] 在完成图校验后创建。
    pub(crate) fn new(
        definitions: Vec<Arc<ComponentDefinition>>,
        ordered_indices: Vec<usize>,
        plan: BuildPlan,
    ) -> Self {
        Self {
            definitions: definitions.into(),
            ordered_indices: ordered_indices.into(),
            plan,
        }
    }

    /// 按稳定注册顺序返回全部定义。
    #[must_use]
    pub fn definitions(&self) -> &[Arc<ComponentDefinition>] {
        &self.definitions
    }

    /// 返回经过校验的依赖优先构建计划。
    #[must_use]
    pub fn plan(&self) -> &BuildPlan {
        &self.plan
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
