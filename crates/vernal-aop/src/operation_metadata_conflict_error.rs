//! 重复操作声明元数据冲突错误对象。

use std::{error::Error, fmt, sync::Arc};

use crate::{Operation, OperationMetadata};

/// 同一稳定操作身份被声明为两组不同元数据时的结构化错误。
///
/// 目录键只由组件名和方法名组成，使 Web/RPC Adapter 在运行期不必重建声明标签；
/// 因而构建阶段必须拒绝同一身份的歧义元数据，不能依赖 `HashMap` 覆盖顺序选择其中
/// 一份。调用方可以通过访问器取得两份开发期元数据进行诊断。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationMetadataConflictError {
    operation: Operation,
    existing: Arc<OperationMetadata>,
    duplicate: Arc<OperationMetadata>,
}

impl OperationMetadataConflictError {
    /// 创建操作声明冲突错误。
    #[must_use]
    pub fn new(
        operation: Operation,
        existing: OperationMetadata,
        duplicate: OperationMetadata,
    ) -> Self {
        Self {
            operation,
            existing: Arc::new(existing),
            duplicate: Arc::new(duplicate),
        }
    }

    /// 返回发生冲突的稳定操作身份。
    #[must_use]
    pub const fn operation(&self) -> &Operation {
        &self.operation
    }

    /// 返回先出现的声明元数据。
    #[must_use]
    pub fn existing(&self) -> &OperationMetadata {
        self.existing.as_ref()
    }

    /// 返回后出现的冲突元数据。
    #[must_use]
    pub fn duplicate(&self) -> &OperationMetadata {
        self.duplicate.as_ref()
    }
}

impl fmt::Display for OperationMetadataConflictError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "conflicting metadata for operation {}",
            self.operation
        )
    }
}

impl Error for OperationMetadataConflictError {}
