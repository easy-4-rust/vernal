//! 组件定义错误对象。

use std::{error::Error, fmt};

use crate::{ComponentKey, TraitKey};

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
    /// 同一 Trait 与目标组件组合被重复绑定。
    DuplicateTraitBinding {
        /// 冲突的 Trait 选择键。
        key: TraitKey,
        /// 被重复绑定的具体组件。
        target: ComponentKey,
    },
    /// 同一 Trait qualifier 被绑定到多个不同目标。
    DuplicateQualifiedTraitBinding {
        /// 无法再保持唯一选择的 Trait 键。
        key: TraitKey,
    },
    /// 同一 Trait 声明了多个 Primary 实现。
    MultiplePrimaryTraitBindings {
        /// 存在多个首选实现的 Trait 类型名。
        trait_name: &'static str,
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
            Self::DuplicateTraitBinding { key, target } => {
                write!(formatter, "duplicate trait binding: {key} -> {target}")
            }
            Self::DuplicateQualifiedTraitBinding { key } => {
                write!(formatter, "duplicate qualified trait binding: {key}")
            }
            Self::MultiplePrimaryTraitBindings { trait_name } => {
                write!(formatter, "multiple primary trait bindings: {trait_name}")
            }
        }
    }
}

impl Error for DefinitionError {}
