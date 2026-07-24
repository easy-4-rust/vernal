//! 组件定义错误对象。

use std::{error::Error, fmt};

use crate::ComponentKey;

/// 组件定义自身无效或与已有定义冲突。
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DefinitionError {
    /// 限定符为空或包含首尾空白。
    InvalidQualifier {
        /// 被拒绝的原始文本。
        value: String,
    },
    /// 同一类型与限定符组合被重复注册。
    DuplicateDefinition {
        /// 冲突的组件标识。
        key: ComponentKey,
    },
}

impl fmt::Display for DefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidQualifier { value } => {
                write!(formatter, "invalid component qualifier {value:?}")
            }
            Self::DuplicateDefinition { key } => {
                write!(formatter, "duplicate component definition: {key}")
            }
        }
    }
}

impl Error for DefinitionError {}
