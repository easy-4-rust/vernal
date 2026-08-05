//! AutowiredArguments — Spring 风格的自动装配参数。
//!
//! 对应 Java 类：`org.springframework.beans.factory.aot.AutowiredArguments`。
//!
//! 表示 AOT 阶段解析的自动装配参数。

use std::any::{Any, TypeId};
use std::sync::Arc;

/// Spring 风格的 `AutowiredArguments`。
///
/// 对应 Spring 的 `AutowiredArguments`。
///
/// 表示 AOT 编译时已解析的自动装配参数。
#[derive(Clone)]
pub struct AutowiredArguments {
    arguments: Vec<Arc<dyn Any + Send + Sync>>,
    type_ids: Vec<TypeId>,
}

impl AutowiredArguments {
    /// 执行empty操作。
    pub fn empty() -> Self {
        Self {
            arguments: Vec::new(),
            type_ids: Vec::new(),
        }
    }

    /// 转换为arguments。
    pub fn from_arguments(args: Vec<Arc<dyn Any + Send + Sync>>) -> Self {
        let type_ids = args.iter().map(|a| (**a).type_id()).collect();
        Self {
            arguments: args,
            type_ids,
        }
    }

    /// 获取条目数量。
    pub fn count(&self) -> usize {
        self.arguments.len()
    }

    /// 判断是否empty。
    pub fn is_empty(&self) -> bool {
        self.arguments.is_empty()
    }

    /// 获取指定条目。
    pub fn get(&self, index: usize) -> Option<Arc<dyn Any + Send + Sync>> {
        self.arguments.get(index).map(Arc::clone)
    }

    /// 获取类型idat。
    pub fn type_id_at(&self, index: usize) -> Option<TypeId> {
        self.type_ids.get(index).copied()
    }
}

impl Default for AutowiredArguments {
    fn default() -> Self {
        Self::empty()
    }
}
