//! 可变组件注册表建造器。

use std::{collections::HashMap, sync::Arc};

use crate::{
    BuildPlan, ComponentDefinition, ComponentKey, DefinitionError, GraphError, Registry,
    graph_planner::GraphPlanner,
};

/// 收集组件定义并在构建时完成全量图校验。
///
/// 建造器只服务于启动阶段，不参与运行时解析。注册顺序会保留下来，作为互不
/// 依赖节点之间的稳定排序依据，使测试、日志和生命周期执行顺序可重复。
#[derive(Debug, Default)]
pub struct RegistryBuilder {
    definitions: Vec<Arc<ComponentDefinition>>,
    key_indices: HashMap<ComponentKey, usize>,
}

impl RegistryBuilder {
    /// 创建空注册表建造器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个组件定义。
    ///
    /// # Errors
    ///
    /// 当同一组件标识已经注册时返回 [`DefinitionError::DuplicateDefinition`]。
    pub fn register(
        &mut self,
        definition: ComponentDefinition,
    ) -> Result<&mut Self, DefinitionError> {
        if self.key_indices.contains_key(definition.key()) {
            return Err(DefinitionError::DuplicateDefinition {
                key: definition.key().clone(),
            });
        }

        let index = self.definitions.len();
        self.key_indices.insert(definition.key().clone(), index);
        self.definitions.push(Arc::new(definition));
        Ok(self)
    }

    /// 校验全部依赖并冻结为不可变注册表。
    ///
    /// # Errors
    ///
    /// 缺少依赖、候选不唯一或存在依赖环时返回 [`GraphError`]。
    pub fn build(self) -> Result<Registry, GraphError> {
        let ordered_indices = GraphPlanner::plan(&self.definitions)?;
        let keys = ordered_indices
            .iter()
            .map(|index| self.definitions[*index].key().clone())
            .collect::<Vec<_>>();

        Ok(Registry::new(
            self.definitions,
            ordered_indices,
            BuildPlan::new(keys),
        ))
    }

    /// 返回当前已注册定义数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// 返回当前是否尚未注册任何定义。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}
