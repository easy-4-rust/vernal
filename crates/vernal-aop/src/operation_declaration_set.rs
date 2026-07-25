//! 操作声明集合对象。

use std::collections::HashMap;

use crate::{Operation, OperationMetadataConflictError};

/// 在调用计划编译前校验并去重稳定操作声明。
///
/// 集合使用 [`Operation`] 的组件名与方法名作为键，同时在值中保留完整声明元数据。
/// 完全相同的重复声明会合并；同一身份但标签或限定符不同会立即返回结构化错误。
/// 该对象只在启动期短暂存在，不进入公开 API 或运行期热路径。
pub(crate) struct OperationDeclarationSet {
    declarations: HashMap<Operation, Operation>,
}

impl OperationDeclarationSet {
    /// 收集、校验并去重操作声明。
    ///
    /// # Errors
    ///
    /// 同一稳定身份出现不同声明元数据时返回
    /// [`OperationMetadataConflictError`]。
    pub(crate) fn collect(
        operations: impl IntoIterator<Item = Operation>,
    ) -> Result<Self, OperationMetadataConflictError> {
        let mut declarations = HashMap::<Operation, Operation>::new();

        for operation in operations {
            if let Some(existing) = declarations.get(&operation) {
                if !existing.same_declaration(&operation) {
                    return Err(OperationMetadataConflictError::new(
                        existing.clone(),
                        existing.metadata().clone(),
                        operation.metadata().clone(),
                    ));
                }
                continue;
            }

            // Key 与 value 暂时各持有一份廉价 Arc 克隆：key 服务身份去重，value
            // 保留权威声明元数据并在校验完成后交给计划建造器。
            declarations.insert(operation.clone(), operation);
        }

        Ok(Self { declarations })
    }

    /// 消费集合并返回全部已经去重的权威声明。
    pub(crate) fn into_operations(self) -> impl Iterator<Item = Operation> {
        self.declarations.into_values()
    }
}
