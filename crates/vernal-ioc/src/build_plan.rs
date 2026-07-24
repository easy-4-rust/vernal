//! 组件构建计划对象。

use std::sync::Arc;

use crate::ComponentKey;

/// 经过依赖图校验的确定性构建顺序。
///
/// 顺序保证依赖组件先于使用方出现；互不依赖的组件按注册顺序排列。应用上下文
/// 可直接复用该计划执行预热和生命周期阶段，而无需重复计算拓扑。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildPlan {
    keys: Arc<[ComponentKey]>,
}

impl BuildPlan {
    /// 从依赖优先的组件标识列表创建计划。
    pub(crate) fn new(keys: Vec<ComponentKey>) -> Self {
        Self { keys: keys.into() }
    }

    /// 按依赖优先顺序返回组件标识。
    #[must_use]
    pub fn keys(&self) -> &[ComponentKey] {
        &self.keys
    }

    /// 返回计划内组件数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// 返回计划是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}
