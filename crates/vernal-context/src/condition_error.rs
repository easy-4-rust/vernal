//! 条件组件装配错误对象。

use std::{error::Error, fmt};

use vernal_core::BoxError;

/// 条件模块声明或构建期评估失败。
///
/// `Display` 与 `Debug` 只输出静态模块名、条件名和错误类别，不展开底层配置来源
/// 错误，避免远端配置响应、属性值或凭证进入常规日志。需要根因时可显式遍历
/// [`Error::source`]。
#[non_exhaustive]
pub enum ConditionError {
    /// 条件模块名为空、包含空白或控制字符。
    InvalidModuleName {
        /// 调用方声明的模块名。
        name: &'static str,
    },
    /// 同一应用建造器内出现重复条件模块名。
    DuplicateModule {
        /// 重复的静态模块名。
        name: &'static str,
    },
    /// 条件模块没有组件、Trait Binding 或生命周期登记。
    EmptyModule {
        /// 空模块的静态名称。
        name: &'static str,
    },
    /// 条件类型名为空、包含空白或控制字符。
    InvalidConditionName {
        /// 所属条件模块。
        module: &'static str,
        /// 条件实现返回的名称。
        condition: &'static str,
    },
    /// Profile 条件没有任何候选项。
    EmptyProfileSet,
    /// Profile 名称为空或包含空白字符。
    InvalidProfile {
        /// 非法 Profile 名称。
        profile: String,
    },
    /// Property 条件键为空或包含空白字符。
    InvalidPropertyKey {
        /// 非法属性键。
        key: String,
    },
    /// 条件读取环境或执行自定义判断时失败。
    EvaluationFailed {
        /// 所属条件模块。
        module: &'static str,
        /// 稳定条件类型名。
        condition: &'static str,
        /// 可供显式诊断链读取的底层错误。
        source: BoxError,
    },
}

impl ConditionError {
    /// 包装一次条件评估失败，同时保留脱敏的模块与条件身份。
    pub(crate) fn evaluation_failed(
        module: &'static str,
        condition: &'static str,
        source: BoxError,
    ) -> Self {
        Self::EvaluationFailed {
            module,
            condition,
            source,
        }
    }
}

impl fmt::Display for ConditionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidModuleName { .. } => {
                formatter.write_str("conditional component module name is invalid")
            }
            Self::DuplicateModule { name } => {
                write!(
                    formatter,
                    "conditional component module is duplicated: {name}"
                )
            }
            Self::EmptyModule { name } => {
                write!(formatter, "conditional component module is empty: {name}")
            }
            Self::InvalidConditionName { module, .. } => {
                write!(formatter, "condition name is invalid for module {module}")
            }
            Self::EmptyProfileSet => formatter.write_str("profile condition is empty"),
            Self::InvalidProfile { .. } => {
                formatter.write_str("profile condition contains an invalid profile")
            }
            Self::InvalidPropertyKey { .. } => {
                formatter.write_str("property condition contains an invalid key")
            }
            Self::EvaluationFailed {
                module, condition, ..
            } => write!(
                formatter,
                "conditional component module {module} failed to evaluate {condition}"
            ),
        }
    }
}

impl fmt::Debug for ConditionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidModuleName { name } => formatter
                .debug_struct("InvalidModuleName")
                .field("name", name)
                .finish(),
            Self::DuplicateModule { name } => formatter
                .debug_struct("DuplicateModule")
                .field("name", name)
                .finish(),
            Self::EmptyModule { name } => formatter
                .debug_struct("EmptyModule")
                .field("name", name)
                .finish(),
            Self::InvalidConditionName { module, condition } => formatter
                .debug_struct("InvalidConditionName")
                .field("module", module)
                .field("condition", condition)
                .finish(),
            Self::EmptyProfileSet => formatter.write_str("EmptyProfileSet"),
            Self::InvalidProfile { .. } => formatter.write_str("InvalidProfile"),
            Self::InvalidPropertyKey { .. } => formatter.write_str("InvalidPropertyKey"),
            Self::EvaluationFailed {
                module, condition, ..
            } => formatter
                .debug_struct("EvaluationFailed")
                .field("module", module)
                .field("condition", condition)
                .field("source", &"<redacted>")
                .finish(),
        }
    }
}

impl Error for ConditionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EvaluationFailed { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}
