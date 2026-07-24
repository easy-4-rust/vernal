//! 可变组件注册表建造器。

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

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

    /// 原子注册一组组件定义。
    ///
    /// 方法会先校验批次内部以及批次与现有注册表之间的全部组件标识。只有所有
    /// 定义都不冲突时才会修改建造器，适合 Hutool-Rust、Sa-Token-Rust 等消费方
    /// Bridge 一次安装相互依赖的组件集合。
    ///
    /// # Errors
    ///
    /// 任一组件标识已经存在，或同一批次中出现重复标识时返回
    /// [`DefinitionError::DuplicateDefinition`]；失败时建造器保持原状。
    pub fn register_all(
        &mut self,
        definitions: impl IntoIterator<Item = ComponentDefinition>,
    ) -> Result<&mut Self, DefinitionError> {
        let definitions = definitions.into_iter().collect::<Vec<_>>();
        let mut batch_keys = HashSet::with_capacity(definitions.len());

        // 先完成全批次预检，避免前几个定义已经写入、后续定义才发现冲突。
        for definition in &definitions {
            let key = definition.key();
            if self.key_indices.contains_key(key) || !batch_keys.insert(key.clone()) {
                return Err(DefinitionError::DuplicateDefinition { key: key.clone() });
            }
        }

        // 预检成功后再一次性提交，注册顺序仍与调用方提供的迭代顺序一致。
        for definition in definitions {
            let index = self.definitions.len();
            self.key_indices.insert(definition.key().clone(), index);
            self.definitions.push(Arc::new(definition));
        }
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
